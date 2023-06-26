mod randomcode;

use base64::Engine;
use byteorder::{ReadBytesExt, WriteBytesExt};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use hmac::Mac;
use prost::Message;
use sha3::Sha3_256;

#[derive(clap::Parser)]
struct Args {
    #[clap(long, default_value = "[::]:9898")]
    bind_addr: std::net::SocketAddr,

    #[clap(long, default_value = "1000")]
    max_users: u64,

    #[clap(long, default_value = "false")]
    use_x_real_ip: bool,

    #[clap(long)]
    ticket_key: String,
}

static CLIENT_VERSION_REQUIREMENT: once_cell::sync::Lazy<semver::VersionReq> =
    once_cell::sync::Lazy::new(|| semver::VersionReq::parse("*").unwrap());

async fn handle_request(
    server_state: std::sync::Arc<ServerState>,
    remote_ip: std::net::IpAddr,
    mut request: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, anyhow::Error> {
    // Browsers cannot set headers on Websocket requests, so this prevents browsers from trivially opening up nettai connections.
    let client_version = if let Some(client_version) = request.headers().get("X-Nettai-Version") {
        semver::Version::parse(client_version.to_str()?)?
    } else {
        return Ok(hyper::Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body(hyper::StatusCode::BAD_REQUEST.canonical_reason().unwrap().into())?);
    };

    if !CLIENT_VERSION_REQUIREMENT.matches(&client_version) {
        return Ok(hyper::Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body(hyper::StatusCode::BAD_REQUEST.canonical_reason().unwrap().into())?);
    }

    let mut current_user_id = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split_once(' '))
        .filter(|(k, _)| *k == "Nettai-Ticket")
        .and_then(|(_, v)| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(v).ok())
        .and_then(|v| bincode::deserialize::<Ticket>(&v).ok())
        .filter(|ticket| {
            let mut hmac = hmac::Hmac::<Sha3_256>::new_from_slice(&server_state.ticket_key).unwrap();
            hmac.update(&ticket.user_id);
            hmac.finalize().into_bytes().as_slice() == ticket.sig
        })
        .map(|ticket| ticket.user_id);

    if !hyper_tungstenite::is_upgrade_request(&request) {
        return Ok(hyper::Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body(hyper::StatusCode::BAD_REQUEST.canonical_reason().unwrap().into())?);
    }

    let (response, websocket) = hyper_tungstenite::upgrade(
        &mut request,
        Some(tungstenite::protocol::WebSocketConfig {
            max_message_size: Some(4 * 1024 * 1024),
            max_frame_size: Some(1 * 1024 * 1024),
            ..Default::default()
        }),
    )?;

    tokio::spawn(async move {
        if let Err(e) = {
            let server_state = server_state.clone();
            let current_user_id = &mut current_user_id;
            (|| async move {
                let websocket = websocket.await?;

                let (tx, rx) = websocket.split();
                let user_state: std::sync::Arc<UserState> = std::sync::Arc::new(UserState {
                    tx: Sender(tokio::sync::Mutex::new(tx)),
                    latencies: tokio::sync::Mutex::new(std::collections::VecDeque::new()),
                    ip: remote_ip,
                });

                let cancellation_token = tokio_util::sync::CancellationToken::new();
                let drop_guard = cancellation_token.clone().drop_guard();

                let user_id = {
                    let mut users = server_state.users.lock().await;
                    let entry = ServerStateUserEntry {
                        user: user_state.clone(),
                        _drop_guard: drop_guard,
                    };

                    if let Some(user_id) = current_user_id.as_ref() {
                        users.insert(user_id.clone(), entry);
                        user_id.clone()
                    } else {
                        loop {
                            let user_id = randomcode::generate().into_bytes();
                            match users.entry(user_id.clone()) {
                                std::collections::hash_map::Entry::Occupied(_) => {
                                    continue;
                                }
                                std::collections::hash_map::Entry::Vacant(e) => {
                                    e.insert(entry);
                                    break user_id;
                                }
                            }
                        }
                    }
                };
                *current_user_id = Some(user_id.clone());

                handle_connection(
                    server_state.clone(),
                    user_id.as_slice(),
                    user_state.clone(),
                    cancellation_token,
                    Receiver(rx),
                )
                .await?;

                Ok::<_, anyhow::Error>(())
            })()
            .await
        } {
            log::error!("connection error for {}: {}", remote_ip, e);
        }

        if let Some(current_user_id) = current_user_id {
            server_state.users.lock().await.remove(&current_user_id);
        }
    });

    Ok(response)
}

struct Sender(
    tokio::sync::Mutex<
        futures_util::stream::SplitSink<
            hyper_tungstenite::WebSocketStream<hyper::upgrade::Upgraded>,
            tungstenite::Message,
        >,
    >,
);

impl Sender {
    async fn send(&self, message: tungstenite::Message) -> Result<(), tungstenite::Error> {
        self.0.lock().await.send(message).await
    }

    async fn send_binary(&self, buf: Vec<u8>) -> Result<(), tungstenite::Error> {
        self.0.lock().await.send(tungstenite::Message::Binary(buf)).await
    }

    async fn send_message(&self, msg: &impl prost::Message) -> Result<(), tungstenite::Error> {
        self.send_binary(msg.encode_to_vec()).await
    }
}

struct Receiver(futures_util::stream::SplitStream<hyper_tungstenite::WebSocketStream<hyper::upgrade::Upgraded>>);

impl Receiver {
    async fn recv(&mut self) -> Option<Result<tungstenite::Message, tungstenite::Error>> {
        self.0.next().await
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Ticket {
    user_id: Vec<u8>,
    sig: Vec<u8>,
}

async fn handle_connection(
    server_state: std::sync::Arc<ServerState>,
    current_user_id: &[u8],
    user_state: std::sync::Arc<UserState>,
    cancellation_token: tokio_util::sync::CancellationToken,
    mut rx: Receiver,
) -> Result<(), anyhow::Error> {
    // Send Hello.
    let ticket = {
        let mut hmac = hmac::Hmac::<sha3::Sha3_256>::new_from_slice(&server_state.ticket_key)?;
        hmac.update(current_user_id);
        Ticket {
            user_id: current_user_id.to_vec(),
            sig: hmac.finalize().into_bytes().to_vec(),
        }
    };

    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        user_state.tx.send_message(&nettai_client::protocol::Packet {
            which: Some(nettai_client::protocol::packet::Which::Hello(
                nettai_client::protocol::packet::Hello {
                    user_id: current_user_id.to_vec(),
                    ticket: bincode::serialize(&ticket).unwrap(),
                },
            )),
        }),
    )
    .await??;

    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            msg = tokio::time::timeout(std::time::Duration::from_secs(60), rx.recv()) => {
                let msg = if let Some(msg) = msg? {
                    msg
                } else {
                    // Stream was closed.
                    return Ok(());
                }?;

                match msg {
                    tungstenite::Message::Binary(buf) => {
                        match nettai_client::protocol::Packet::decode(&mut bytes::Bytes::from(buf))?
                            .which
                            .ok_or_else(|| anyhow::anyhow!("unknown packet"))?
                        {
                            msg => {
                                return Err(anyhow::format_err!("unknown packet: {:?}", msg));
                            }
                        }
                    }

                    tungstenite::Message::Pong(buf) => {
                        let unix_ts_ms = buf.as_slice().read_u64::<byteorder::LittleEndian>()?;
                        let ts = std::time::UNIX_EPOCH + std::time::Duration::from_millis(unix_ts_ms);
                        let now = std::time::SystemTime::now();

                        // Record time.
                        let mut latencies = user_state.latencies.lock().await;
                        const MAX_LATENCIES: usize = 9;
                        while latencies.len() >= MAX_LATENCIES {
                            latencies.pop_front();
                        }
                        latencies.push_back(now.duration_since(ts)?);
                    }

                    _ => {
                        return Err(anyhow::anyhow!("cannot handle this message: {}", msg));
                    }
                }
            }

            _ = ping_interval.tick() => {
                let now = std::time::SystemTime::now();
                let unix_ts_ms = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
                let mut buf = vec![];
                buf.write_u64::<byteorder::LittleEndian>(unix_ts_ms).unwrap();
                tokio::time::timeout(std::time::Duration::from_secs(60), user_state.tx.send(tungstenite::Message::Ping(buf))).await??;
            }

            _ = cancellation_token.cancelled() => {
                return Ok(());
            }
        }
    }
}

struct UserState {
    tx: Sender,
    latencies: tokio::sync::Mutex<std::collections::VecDeque<std::time::Duration>>,
    ip: std::net::IpAddr,
}

struct ServerStateUserEntry {
    user: std::sync::Arc<UserState>,
    _drop_guard: tokio_util::sync::DropGuard,
}

struct ServerState {
    ticket_key: Vec<u8>,
    users: tokio::sync::Mutex<std::collections::HashMap<Vec<u8>, ServerStateUserEntry>>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::Builder::from_default_env()
        .filter(Some("nettai_server"), log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    let ticket_key = args.ticket_key.into_bytes();

    assert!(ticket_key.len() >= 32, "ticket key must be at least 32 bytes long");

    let server_state = std::sync::Arc::new(ServerState {
        ticket_key,
        users: tokio::sync::Mutex::new(std::collections::HashMap::new()),
    });

    let incoming = hyper::server::conn::AddrIncoming::bind(&args.bind_addr)?;
    log::info!("listening on: {}", incoming.local_addr());

    hyper::Server::builder(incoming)
        .serve(hyper::service::make_service_fn(
            move |stream: &hyper::server::conn::AddrStream| {
                let server_state = server_state.clone();
                let raw_remote_ip = stream.remote_addr().ip();
                async move {
                    Ok::<_, std::convert::Infallible>(hyper::service::service_fn(move |request| {
                        let server_state = server_state.clone();
                        async move {
                            let remote_ip = if args.use_x_real_ip {
                                request
                                    .headers()
                                    .get("X-Real-IP")
                                    .ok_or_else(|| anyhow::anyhow!("missing X-Real-IP header"))?
                                    .to_str()?
                                    .parse()?
                            } else {
                                raw_remote_ip
                            };
                            handle_request(server_state, remote_ip, request).await
                        }
                    }))
                }
            },
        ))
        .await?;

    Ok(())
}
