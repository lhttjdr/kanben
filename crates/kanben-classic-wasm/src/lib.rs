//! 原版排版器文本處理的瀏覽器接口。頁面上原版的全局函數換成這裏的同名實現。
//!
//! 環境值（`調距`、`行尾空白長`、`系統`）每次由頁面傳入，寫作 `鍵=值;鍵=值`。

use kanben_classic::marks;
use kanben_classic::text::{self, Env, 抽字};
use wasm_bindgen::prelude::*;

fn env(spec: &str) -> Env {
    let mut e = Env::default();
    for kv in spec.split(';') {
        let Some((k, v)) = kv.split_once('=') else {
            continue;
        };
        let f = v.parse::<f64>().ok();
        match (k, f) {
            ("正文", Some(f)) => e.正文 = f,
            ("夾注", Some(f)) => e.夾注 = f,
            ("注中注", Some(f)) => e.注中注 = f,
            ("注中小字", Some(f)) => e.注中小字 = f,
            ("敘", Some(f)) => e.敘 = f,
            ("跋", Some(f)) => e.跋 = f,
            ("行尾空白長", Some(f)) => e.行尾空白長 = f,
            ("系統", _) => e.系統 = v.to_string(),
            _ => {}
        }
    }
    e
}

/// `toHTML`
#[wasm_bindgen(js_name = toHTML)]
pub fn to_html(src: &str, env_spec: &str) -> String {
    text::to_html(src, &env(env_spec))
}

/// `寫眉批`：返回 `正文\u{0}天頭HTML`。
#[wasm_bindgen(js_name = headNotes)]
pub fn head_notes(src: &str) -> String {
    let (body, notes) = text::寫眉批(src);
    format!("{body}\u{0}{}", text::天頭(&notes))
}

/// `點行間`
#[wasm_bindgen(js_name = markBody)]
pub fn mark_body(html: &str) -> String {
    marks::點行間(html)
}

/// `點字下`
#[wasm_bindgen(js_name = markNote)]
pub fn mark_note(html: &str, env_spec: &str) -> String {
    marks::點字下(html, &env(env_spec))
}

/// `行間標點`；`clr` 空串表示不著色。
#[wasm_bindgen(js_name = markInter)]
pub fn mark_inter(html: &str, clr: &str) -> String {
    marks::行間標點(html, (!clr.is_empty()).then_some(clr))
}

/// `punctuateVH(類, true, 文)`：返回 HTML；抽出的拼字、補遺字由 [`restore`] 放回。
#[wasm_bindgen]
pub struct Punct {
    html: String,
    state: 抽字,
}

#[wasm_bindgen]
impl Punct {
    #[wasm_bindgen(getter)]
    pub fn html(&self) -> String {
        self.html.clone()
    }

    /// `punctuateVH(類, false, 容器號)` 的後半：佔位字換回原字。
    pub fn restore(&self, html: &str) -> String {
        text::punctuate_vh_2_restore(html, &self.state)
    }

    /// 有沒有要放回的字（原版沒有時直接返回，不動容器）。
    #[wasm_bindgen(getter, js_name = hasPending)]
    pub fn has_pending(&self) -> bool {
        self.state.拼字.is_some() || self.state.補遺字.is_some()
    }
}

#[wasm_bindgen(js_name = punctuate)]
pub fn punctuate(cls: &str, src: &str) -> Punct {
    let (html, state) = text::punctuate_vh_1(cls, src);
    Punct { html, state }
}

/// `punctuateVH(類, false, …)` 的前半：夾注元素內容的直排標點換回橫排。
#[wasm_bindgen(js_name = unverticalize)]
pub fn unverticalize(inner: &str) -> String {
    text::punctuate_vh_2_item(inner)
}

/// `構字`
#[wasm_bindgen(js_name = composeChar)]
pub fn compose_char(ids: &str) -> String {
    text::構字(ids)
}

use kanben_classic::layout::{self, 夾注排法};

/// 一條雙行夾注的排法（見 [`layout::夾注排法`]）。`kind` 為 `ig`、`sht`、`long`。
#[wasm_bindgen]
pub struct NotePlan {
    plan: 夾注排法,
}

#[wasm_bindgen]
impl NotePlan {
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        match self.plan {
            夾注排法::不分 => "ig",
            夾注排法::本行 { .. } => "sht",
            夾注排法::跨行 { .. } => "long",
        }
        .into()
    }

    fn field(&self, f: impl Fn(&夾注排法) -> Option<&String>) -> String {
        f(&self.plan).cloned().unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> String {
        self.field(|p| match p {
            夾注排法::本行 { height } => Some(height),
            _ => None,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> String {
        self.field(|p| match p {
            夾注排法::跨行 { width, .. } => Some(width),
            _ => None,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn clip(&self) -> String {
        self.field(|p| match p {
            夾注排法::跨行 { clip, .. } => Some(clip),
            _ => None,
        })
    }

    #[wasm_bindgen(getter, js_name = headHeight)]
    pub fn head_height(&self) -> String {
        self.field(|p| match p {
            夾注排法::跨行 { head_height, .. } => Some(head_height),
            _ => None,
        })
    }

    #[wasm_bindgen(getter, js_name = tailClip)]
    pub fn tail_clip(&self) -> String {
        self.field(|p| match p {
            夾注排法::跨行 { tail_clip, .. } => Some(tail_clip),
            _ => None,
        })
    }

    #[wasm_bindgen(getter, js_name = afterHeight)]
    pub fn after_height(&self) -> String {
        self.field(|p| match p {
            夾注排法::跨行 { after_height, .. } => Some(after_height),
            _ => None,
        })
    }
}

/// 雙行夾注的字數（原版 `雙行夾注` 的 `丙`）。
#[wasm_bindgen(js_name = noteLength)]
pub fn note_length(inner: &str) -> usize {
    layout::夾注字數(inner)
}

/// 雙行夾注的排法。`start` 起始格，`indent` 段首縮進格數，`cells` 每行格數。
#[wasm_bindgen(js_name = notePlan)]
pub fn note_plan(len: usize, start: i32, indent: i32, cells: i32) -> NotePlan {
    NotePlan {
        plan: layout::夾注排法(len, start.into(), indent.into(), cells.into()),
    }
}

/// 改窗口後重新判斷雙行夾注的排法；不用改時返回 undefined。
#[wasm_bindgen(js_name = noteReplan)]
pub fn note_replan(
    big: i32,
    start: i32,
    indent: i32,
    cells: i32,
    is_long: bool,
) -> Option<NotePlan> {
    layout::調整排法(
        big.into(),
        start.into(),
        indent.into(),
        cells.into(),
        is_long,
    )
    .map(|plan| NotePlan { plan })
}

use kanben_classic::misc;

/// `getBrowserInfo`：返回 `名\t版本`。
#[wasm_bindgen(js_name = browserInfo)]
pub fn browser_info(ua: &str) -> String {
    let (名, 版本) = misc::瀏覽器(ua);
    format!("{名}\t{版本}")
}

/// `detectOS`
#[wasm_bindgen(js_name = detectOs)]
pub fn detect_os(ua: &str, platform: &str) -> String {
    misc::系統(ua, platform).to_string()
}

/// `數.讀百以內合文`
#[wasm_bindgen(js_name = readUnder100)]
pub fn read_under_100(n: u32) -> String {
    kanben_classic::text::讀百以內合文(n as usize)
}

/// `數.讀十萬以內`
#[wasm_bindgen(js_name = readUnder100k)]
pub fn read_under_100k(n: f64) -> String {
    misc::讀十萬以內(n.max(0.0) as u64)
}

/// `析整數`
#[wasm_bindgen(js_name = parseHanNumber)]
pub fn parse_han_number(s: &str) -> f64 {
    misc::析整數(s)
}

/// `整數籌`：返回 `數\t類\t寬窄\t負號`。`n` 為字串時按碼子字處理，為數字時按數值。
#[wasm_bindgen(js_name = rodStyle)]
pub fn rod_style(n: &JsValue, units_vertical: bool) -> String {
    let r = match n.as_f64() {
        Some(v) => misc::整數籌_數(v, units_vertical),
        None => misc::整數籌_字(&n.as_string().unwrap_or_default(), units_vertical),
    };
    format!(
        "{}\t{}\t{}\t{}",
        r.數,
        r.類,
        kanben_classic::text::num(r.寬窄),
        r.負號
    )
}
