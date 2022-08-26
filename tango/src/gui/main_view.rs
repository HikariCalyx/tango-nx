use fluent_templates::Loader;

use crate::{gui, i18n, input, net, session};

pub enum State {
    Session(session::Session),
    Start(Start),
}

enum ConnectionFailure {}

enum ConnectionTask {
    InProgress {
        state: ConnectionState,
        cancellation_token: tokio_util::sync::CancellationToken,
    },
    InLobby(Lobby),
    Failed(anyhow::Error),
}

enum ConnectionState {
    Starting,
    Signaling,
    Waiting,
}

struct Lobby {
    dc: datachannel_wrapper::DataChannel,
    peer_conn: datachannel_wrapper::PeerConnection,
}

pub struct Start {
    link_code: String,
    connection_task: std::sync::Arc<tokio::sync::Mutex<Option<ConnectionTask>>>,
    show_save_select: Option<gui::save_select_window::State>,
}

impl Start {
    pub fn new() -> Self {
        Self {
            link_code: String::new(),
            connection_task: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            show_save_select: None,
        }
    }
}

pub struct MainView {
    session_view: gui::session_view::SessionView,
    save_select_window: gui::save_select_window::SaveSelectWindow,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            session_view: gui::session_view::SessionView::new(),
            save_select_window: gui::save_select_window::SaveSelectWindow::new(),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        handle: tokio::runtime::Handle,
        input_state: &input::State,
        state: &mut gui::State,
    ) {
        match &mut state.main_view {
            State::Session(session) => {
                self.session_view.show(
                    ctx,
                    input_state,
                    &state.config.input_mapping,
                    session,
                    &state.config.video_filter,
                    state.config.max_scale,
                );
            }
            State::Start(start) => {
                self.save_select_window.show(
                    ctx,
                    &mut start.show_save_select,
                    &state.config.language,
                    &state.config.saves_path,
                    state.saves_list.clone(),
                    state.audio_binder.clone(),
                    state.emu_tps_counter.clone(),
                );

                egui::TopBottomPanel::top("main-top-panel")
                    .frame(egui::Frame {
                        inner_margin: egui::style::Margin::symmetric(8.0, 2.0),
                        rounding: egui::Rounding::none(),
                        fill: ctx.style().visuals.window_fill(),
                        ..Default::default()
                    })
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .selectable_label(state.show_settings.is_some(), "⚙️")
                                        .on_hover_text_at_pointer(
                                            i18n::LOCALES
                                                .lookup(&state.config.language, "settings")
                                                .unwrap(),
                                        )
                                        .clicked()
                                    {
                                        state.show_settings = if state.show_settings.is_none() {
                                            Some(gui::settings_window::State::new())
                                        } else {
                                            None
                                        };
                                    }
                                },
                            );
                        });
                    });
                egui::TopBottomPanel::bottom("main-bottom-panel")
                    .frame(egui::Frame {
                        inner_margin: egui::style::Margin::symmetric(8.0, 2.0),
                        rounding: egui::Rounding::none(),
                        fill: ctx.style().visuals.window_fill(),
                        ..Default::default()
                    })
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let submit = |start: &Start| {
                                        if !start.link_code.is_empty() {
                                            let cancellation_token =
                                                tokio_util::sync::CancellationToken::new();
                                            let connection_task = start.connection_task.clone();

                                            *connection_task.blocking_lock() =
                                                Some(ConnectionTask::InProgress {
                                                    state: ConnectionState::Starting,
                                                    cancellation_token: cancellation_token
                                                        .clone(),
                                                });

                                            let matchmaking_addr =
                                                state.config.matchmaking_endpoint.clone();
                                            let link_code = start.link_code.clone();

                                            handle.spawn(async move {
                                                log::info!("spawning connection task");
                                                if let Err(e) = {
                                                    let connection_task = connection_task.clone();

                                                    tokio::select!{
                                                        r = {
                                                            let connection_task = connection_task.clone();
                                                            let cancellation_token = cancellation_token.clone();
                                                            (move || async move {
                                                                *connection_task.lock().await =
                                                                    Some(ConnectionTask::InProgress {
                                                                        state: ConnectionState::Signaling,
                                                                        cancellation_token:
                                                                            cancellation_token.clone(),
                                                                    });
                                                                const OPEN_TIMEOUT: std::time::Duration =
                                                                    std::time::Duration::from_secs(30);
                                                                let pending_conn = tokio::time::timeout(
                                                                    OPEN_TIMEOUT,
                                                                    net::signaling::open(
                                                                        &matchmaking_addr,
                                                                        &link_code,
                                                                    ),
                                                                )
                                                                .await??;

                                                                *connection_task.lock().await =
                                                                    Some(ConnectionTask::InProgress {
                                                                        state: ConnectionState::Waiting,
                                                                        cancellation_token:
                                                                            cancellation_token.clone(),
                                                                    });

                                                                let (mut dc, peer_conn) = pending_conn.connect().await?;
                                                                net::negotiate(&mut dc).await?;

                                                                *connection_task.lock().await =
                                                                    Some(ConnectionTask::InLobby(Lobby{
                                                                        dc,
                                                                        peer_conn,
                                                                    }));

                                                                Ok(())
                                                            })(
                                                            )
                                                        }
                                                        => { r }
                                                        _ = cancellation_token.cancelled() => {
                                                            *connection_task.lock().await = None;
                                                            log::info!("connection task cancelled");
                                                            return;
                                                        }
                                                    }
                                                } {
                                                    log::info!("connection task failed: {:?}", e);
                                                    *connection_task.lock().await =
                                                        Some(ConnectionTask::Failed(e));
                                                }
                                            });
                                        }
                                    };

                                    let cancellation_token = if let Some(connection_task) = &*start.connection_task.blocking_lock() {
                                        match connection_task {
                                            ConnectionTask::InProgress { state: _, cancellation_token } => Some(cancellation_token.clone()),
                                            ConnectionTask::InLobby(_) => None,
                                            ConnectionTask::Failed(_) => None,
                                        }
                                    } else {
                                        None
                                    };

                                    if let Some(cancellation_token) = &cancellation_token {
                                        if ui.button(
                                            format!(
                                                "⏹️ {}",
                                                i18n::LOCALES
                                                    .lookup(&state.config.language, "start.stop")
                                                    .unwrap()
                                                )
                                            ).clicked() {
                                            cancellation_token.cancel();
                                        }
                                    } else {
                                        if ui
                                            .button(if start.link_code.is_empty() {
                                                format!(
                                                    "▶️ {}",
                                                    i18n::LOCALES
                                                        .lookup(&state.config.language, "start.play")
                                                        .unwrap()
                                                )
                                            } else {
                                                format!(
                                                    "🥊 {}",
                                                    i18n::LOCALES
                                                        .lookup(&state.config.language, "start.fight")
                                                        .unwrap()
                                                )
                                            })
                                            .clicked()
                                        {
                                            submit(start);
                                        }
                                    }

                                    let input_resp = ui.add(
                                        egui::TextEdit::singleline(&mut start.link_code)
                                            .interactive(cancellation_token.is_none())
                                            .hint_text(
                                                i18n::LOCALES
                                                    .lookup(
                                                        &state.config.language,
                                                        "start.link-code",
                                                    )
                                                    .unwrap(),
                                            )
                                            .desired_width(f32::INFINITY),
                                    );
                                    start.link_code = start
                                        .link_code
                                        .to_lowercase()
                                        .chars()
                                        .filter(|c| {
                                            "abcdefghijklmnopqrstuvwxyz0123456789-"
                                                .chars()
                                                .any(|c2| c2 == *c)
                                        })
                                        .take(40)
                                        .collect::<String>()
                                        .trim_start_matches("-")
                                        .to_string();

                                    if let Some(last) = start.link_code.chars().last() {
                                        if last == '-' {
                                            start.link_code = start
                                                .link_code
                                                .chars()
                                                .rev()
                                                .skip_while(|c| *c == '-')
                                                .collect::<Vec<_>>()
                                                .into_iter()
                                                .rev()
                                                .collect::<String>()
                                                + "-";
                                        }
                                    }

                                    if input_resp.lost_focus()
                                        && ctx.input().key_pressed(egui::Key::Enter)
                                    {
                                        submit(start);
                                    }
                                },
                            );
                        });
                    });
                egui::CentralPanel::default().show(ctx, |ui| {});
            }
        }
    }
}
