mod msg;

pub struct Offsets {
    chip_data: u32,
    chip_names_pointer: u32,
    chip_descriptions_pointer: u32,
    chip_icon_palette_pointer: u32,
    element_icon_palette_pointer: u32,
    element_icons_pointer: u32,
}

#[rustfmt::skip]
pub static AREE_00: Offsets = Offsets {
    chip_data:                      0x08007d70,
    chip_names_pointer:             0x080145f4,
    chip_descriptions_pointer:      0x08016104,
    element_icons_pointer:          0x0801a688,
    element_icon_palette_pointer:   0x08005a1c,
    chip_icon_palette_pointer:      0x08015ebc,
};

#[rustfmt::skip]
pub static AREJ_00: Offsets = Offsets {
    chip_data:                      0x08007d3c,
    chip_names_pointer:             0x08014578,
    chip_descriptions_pointer:      0x08016088,
    element_icons_pointer:          0x0801a5a4,
    element_icon_palette_pointer:   0x08005a0c,
    chip_icon_palette_pointer:      0x08015e40,
};

pub struct Assets {
    offsets: &'static Offsets,
    msg_parser: msg::Parser,
    mapper: crate::rom::MemoryMapper,
    chip_icon_palette: [image::Rgba<u8>; 16],
    element_icon_palette: [image::Rgba<u8>; 16],
}

struct Chip<'a> {
    id: usize,
    assets: &'a Assets,
}

#[repr(packed, C)]
#[derive(bytemuck::AnyBitPattern, Clone, Copy)]
struct RawChip {
    codes: [u8; 5],
    element: u8,
    _unk_06: [u8; 6],
    attack_power: u16,
    _unk_0e: [u8; 2],
    icon_ptr: u32,
    image_ptr: u32,
    palette_ptr: u32,
}
const _: () = assert!(std::mem::size_of::<RawChip>() == 0x1c);

impl<'a> Chip<'a> {
    fn raw(&'a self) -> RawChip {
        bytemuck::pod_read_unaligned(
            &self.assets.mapper.get(self.assets.offsets.chip_data)[self.id * std::mem::size_of::<RawChip>()..]
                [..std::mem::size_of::<RawChip>()],
        )
    }
}

impl<'a> crate::rom::Chip for Chip<'a> {
    fn name(&self) -> Option<String> {
        let region = self.assets.mapper.get(bytemuck::pod_read_unaligned::<u32>(
            &self.assets.mapper.get(self.assets.offsets.chip_names_pointer)[..std::mem::size_of::<u32>()],
        ));
        let entry = crate::msg::get_entry(&region, self.id)?;

        Some(
            self.assets
                .msg_parser
                .parse(entry)
                .ok()?
                .into_iter()
                .flat_map(|part| {
                    match part {
                        crate::msg::Chunk::Text(s) => s,
                        _ => "".to_string(),
                    }
                    .chars()
                    .collect::<Vec<_>>()
                })
                .collect::<String>(),
        )
    }

    fn description(&self) -> Option<String> {
        let region = self.assets.mapper.get(bytemuck::pod_read_unaligned::<u32>(
            &self.assets.mapper.get(self.assets.offsets.chip_descriptions_pointer)[..std::mem::size_of::<u32>()],
        ));
        let entry = crate::msg::get_entry(&region, self.id)?;

        Some(
            self.assets
                .msg_parser
                .parse(entry)
                .ok()?
                .into_iter()
                .flat_map(|part| {
                    match part {
                        crate::msg::Chunk::Text(s) => s,
                        _ => "".to_string(),
                    }
                    .chars()
                    .collect::<Vec<_>>()
                })
                .collect::<String>(),
        )
    }

    fn icon(&self) -> image::RgbaImage {
        let raw = self.raw();
        crate::rom::apply_palette(
            crate::rom::read_merged_tiles(&self.assets.mapper.get(raw.icon_ptr)[..crate::rom::TILE_BYTES * 4], 2)
                .unwrap(),
            &self.assets.chip_icon_palette,
        )
    }

    fn image(&self) -> image::RgbaImage {
        let raw = self.raw();
        crate::rom::apply_palette(
            crate::rom::read_merged_tiles(
                &self.assets.mapper.get(raw.image_ptr)[..crate::rom::TILE_BYTES * 8 * 7],
                8,
            )
            .unwrap(),
            &crate::rom::read_palette(&self.assets.mapper.get(raw.palette_ptr)[..crate::rom::PALETTE_BYTES]),
        )
    }

    fn codes(&self) -> Vec<char> {
        let raw = self.raw();
        raw.codes
            .iter()
            .cloned()
            .filter(|code| *code != 0xff)
            .map(|code| b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[code as usize] as char)
            .collect()
    }

    fn element(&self) -> usize {
        let raw = self.raw();
        raw.element as usize
    }

    fn class(&self) -> crate::rom::ChipClass {
        crate::rom::ChipClass::Standard
    }

    fn dark(&self) -> bool {
        false
    }

    fn mb(&self) -> u8 {
        0
    }

    fn attack_power(&self) -> u32 {
        let raw = self.raw();
        raw.attack_power as u32
    }

    fn library_sort_order(&self) -> Option<usize> {
        Some(self.id)
    }
}

impl Assets {
    pub fn new(offsets: &'static Offsets, charset: &[String], rom: Vec<u8>, wram: Vec<u8>) -> Self {
        let mapper = crate::rom::MemoryMapper::new(rom, wram);
        let chip_icon_palette = crate::rom::read_palette(
            &mapper.get(bytemuck::pod_read_unaligned::<u32>(
                &mapper.get(offsets.chip_icon_palette_pointer)[..std::mem::size_of::<u32>()],
            ))[..crate::rom::PALETTE_BYTES],
        );
        let element_icon_palette = crate::rom::read_palette(
            &mapper.get(bytemuck::pod_read_unaligned::<u32>(
                &mapper.get(offsets.element_icon_palette_pointer)[..std::mem::size_of::<u32>()],
            ))[..crate::rom::PALETTE_BYTES],
        );

        Self {
            offsets,
            msg_parser: msg::parser(charset),
            mapper,
            chip_icon_palette,
            element_icon_palette,
        }
    }
}

impl crate::rom::Assets for Assets {
    fn chip<'a>(&'a self, id: usize) -> Option<Box<dyn crate::rom::Chip + 'a>> {
        if id >= self.num_chips() {
            return None;
        }
        Some(Box::new(Chip { id, assets: self }))
    }

    fn num_chips(&self) -> usize {
        super::NUM_CHIPS
    }

    fn chips_have_mb(&self) -> bool {
        false
    }

    fn element_icon(&self, id: usize) -> Option<image::RgbaImage> {
        if id >= 5 {
            return None;
        }

        let buf = self.mapper.get(bytemuck::pod_read_unaligned::<u32>(
            &self.mapper.get(self.offsets.element_icons_pointer)[..std::mem::size_of::<u32>()],
        ));
        Some(crate::rom::apply_palette(
            crate::rom::read_merged_tiles(&buf[id * crate::rom::TILE_BYTES * 4..][..crate::rom::TILE_BYTES * 4], 2)
                .unwrap(),
            &self.element_icon_palette,
        ))
    }
}

#[rustfmt::skip]
pub const EN_CHARSET: &[&str] = &[" ", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "[Lv.]", "[11]", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "-", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "[.]", "[×]", "[=]", "[:]", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "!", "‼", "?", "“", "„", "#", "♭", "$", "%", "&", "'", "(", ")", "~", "^", "\"", "∧", "∨", "<", ">", ",", "。", ".", "・", "/", "\\\\", "_", "「", "」", "\\[", "\\]", "[bracket1]", "[bracket2]", "⊂", "⊃", "∩", "[raindrop]", "↑", "→", "↓", "←", "∀", "α", "β", "@", "★", "♥", "♪", "℃", "♂", "♀", "＿", "｜", "￣", ":", ";", "…", "¥", "+", "×", "÷", "=", "※", "*", "○", "●", "◎", "□", "■", "◇", "◆", "△", "▲", "▽", "▼", "▶", "◀", "☛", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "�", "¼", "[infinity1]", "[infinity2]"];

#[rustfmt::skip]
pub const JA_CHARSET: &[&str] = &[" ", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "ア", "イ", "ウ", "エ", "オ", "カ", "キ", "ク", "ケ", "コ", "サ", "シ", "ス", "セ", "ソ", "タ", "チ", "ツ", "テ", "ト", "ナ", "ニ", "ヌ", "ネ", "ノ", "ハ", "ヒ", "フ", "ヘ", "ホ", "マ", "ミ", "ム", "メ", "モ", "ヤ", "ユ", "ヨ", "ラ", "リ", "ル", "レ", "ロ", "ワ", "ヰ", "ヱ", "ヲ", "ン", "ガ", "ギ", "グ", "ゲ", "ゴ", "ザ", "ジ", "ズ", "ゼ", "ゾ", "ダ", "ヂ", "ヅ", "デ", "ド", "バ", "ビ", "ブ", "ベ", "ボ", "パ", "ピ", "プ", "ペ", "ポ", "ァ", "ィ", "ゥ", "ェ", "ォ", "ッ", "ャ", "ュ", "ョ", "ヴ", "ー", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "あ", "い", "う", "え", "お", "か", "き", "く", "け", "こ", "さ", "し", "す", "せ", "そ", "た", "ち", "つ", "て", "と", "な", "に", "ぬ", "ね", "の", "は", "ひ", "ふ", "へ", "ほ", "ま", "み", "む", "め", "も", "や", "ゆ", "よ", "ら", "り", "る", "れ", "ろ", "わ", "ゐ", "ゑ", "を", "ん", "が", "ぎ", "ぐ", "げ", "ご", "ざ", "じ", "ず", "ぜ", "ぞ", "だ", "ぢ", "づ", "で", "ど", "ば", "び", "ぶ", "べ", "ぼ", "ぱ", "ぴ", "ぷ", "ぺ", "ぽ", "ぁ", "ぃ", "ぅ", "ぇ", "ぉ", "っ", "ゃ", "ゅ", "ょ", "!", "‼", "?", "“", "„", "#", "♭", "$", "%", "&", "'", "(", ")", "~", "^", "\"", "∧", "∨", "<", ">", "、", "。", ".", "・", "/", "\\\\", "_", "「", "」", "\\[", "\\]", "[bracket1]", "[bracket2]", "⊂", "⊃", "∩", "[raindrop]", "↑", "→", "↓", "←", "∀", "α", "β", "@", "★", "♥", "♪", "℃", "♂", "♀", "＿", "｜", "￣", ":", ";", "…", "¥", "+", "×", "÷", "=", "※", "*", "○", "●", "◎", "□", "■", "◇", "◆", "△", "▲", "▽", "▼", "▶", "◀", "☛", "止", "彩", "起", "父", "博", "士", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十", "百", "千", "万", "億", "上", "下", "左", "右", "手", "足", "日", "目", "月", "顔", "頭", "人", "入", "出", "山", "口", "光", "電", "気", "話", "広", "雨", "名", "前", "学", "校", "保", "健", "室", "世", "界", "体", "育", "館", "信", "号", "機", "器", "大", "小", "中", "自", "分", "間", "開", "閉", "問", "聞", "門", "熱", "斗", "要", "住", "道", "行", "街", "屋", "水", "見", "家", "教", "走", "先", "生", "長", "今", "事", "点", "女", "子", "言", "会", "来", "¼", "[infinity1]", "[infinity2]", "思", "時", "円", "知", "毎", "年", "火", "朝", "計", "画", "休", "曜", "帰", "回", "外", "多", "考", "正", "死", "値", "合", "戦", "争", "秋", "原", "町", "天", "用", "金", "男", "作", "数", "方", "社", "攻", "撃", "力", "同", "武", "何", "発", "少", "度", "以", "楽", "早", "暮", "面", "組", "後", "文", "字", "本", "階", "岩", "才", "者", "立", "官", "庁", "ヶ", "連", "射", "国", "局", "耳", "土", "炎", "伊", "集", "院", "各", "科", "省", "祐", "朗", "枚", "永", "川", "花", "兄", "茶", "音", "属", "性", "持", "勝", "赤", "充", "池", "停", "丁", "舎", "地", "所", "明", "切", "急", "木", "無", "高", "駅", "店", "闘", "絵", "球", "研", "究", "香"];
