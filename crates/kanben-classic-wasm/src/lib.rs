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
