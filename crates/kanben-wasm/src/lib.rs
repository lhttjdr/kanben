//! 浏览器接口。排版全部在 wasm 里完成，JS 只负责把 SVG 放进页面。
//!
//! ```js
//! import init, { typeset, style } from "./pkg/kanben_wasm.js";
//! await init();
//! const r = typeset(source, "format=scroll\npunct=grid");
//! r.pageCount; r.page(0); JSON.parse(r.diagnostics);
//! ```

use kanben::syntax::span::{LineIndex, Severity};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Typeset {
    pages: Vec<String>,
    diagnostics: String,
    title: String,
}

#[wasm_bindgen]
impl Typeset {
    #[wasm_bindgen(getter, js_name = pageCount)]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 第 `i` 页的 SVG。
    pub fn page(&self, i: usize) -> String {
        self.pages.get(i).cloned().unwrap_or_default()
    }

    /// 诊断信息，JSON 数组。偏移有两种：`start`/`end` 是 UTF-8 字节，
    /// `from`/`to` 是 UTF-16 码元（可直接用于 textarea 的 setSelectionRange）。
    #[wasm_bindgen(getter)]
    pub fn diagnostics(&self) -> String {
        self.diagnostics.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }
}

/// 排版。`overrides` 每行一个 `键=值`（同 `\\setup`），覆盖源码里的设定。
#[wasm_bindgen]
pub fn typeset(src: &str, overrides: &str) -> Typeset {
    let pairs = parse_overrides(overrides);
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let out = kanben::typeset(src, &refs);
    Typeset {
        diagnostics: diagnostics_json(src, &out.diagnostics),
        title: out.options.title.clone(),
        pages: out.pages,
    }
}

/// 导出完整 HTML（可打印）。
#[wasm_bindgen]
pub fn html(src: &str, overrides: &str) -> String {
    let pairs = parse_overrides(overrides);
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let out = kanben::typeset(src, &refs);
    kanben::render::html(&out.options.title, &out.pages)
}

/// 按可用高度（像素）算每行字數，让一个字约 `cell_px` 像素高。
#[wasm_bindgen(js_name = fitCells)]
pub fn fit_cells(src: &str, overrides: &str, available_px: f32, cell_px: f32) -> i32 {
    let pairs = parse_overrides(overrides);
    let refs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    kanben::fit_cells(src, &refs, available_px, cell_px)
}

/// 整理从版面复制出来的文字：去掉空白，直排字形还原成普通标点。
#[wasm_bindgen(js_name = plainText)]
pub fn plain_text(copied: &str) -> String {
    let compact: String = copied.chars().filter(|c| !c.is_whitespace()).collect();
    kanben::plain_text(&compact)
}

/// SVG 需要的样式表。
#[wasm_bindgen]
pub fn style() -> String {
    kanben::render::STYLE.to_string()
}

/// 示例文档。
#[wasm_bindgen]
pub fn example() -> String {
    include_str!("../../../examples/通鑑.kb").to_string()
}

fn parse_overrides(s: &str) -> Vec<(String, String)> {
    s.lines()
        .filter_map(|l| l.split_once(['=', '＝']))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .filter(|(k, v)| !k.is_empty() && !v.is_empty())
        .collect()
}

fn diagnostics_json(src: &str, diags: &[kanben::syntax::span::Diagnostic]) -> String {
    let index = LineIndex::new(src);
    let utf16 = |byte: usize| {
        src.get(..byte.min(src.len()))
            .map_or(0, |s| s.encode_utf16().count())
    };
    let items: Vec<String> = diags
        .iter()
        .map(|d| {
            let (line, col) = index.line_col(d.span.start);
            format!(
                r#"{{"severity":"{}","start":{},"end":{},"from":{},"to":{},"line":{},"col":{},"message":"{}"}}"#,
                match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                },
                d.span.start,
                d.span.end,
                utf16(d.span.start),
                utf16(d.span.end),
                line,
                col,
                json_escape(&d.message)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_have_utf16_offsets() {
        let t = typeset("𠀀\\note{甲", "");
        assert!(
            t.diagnostics.contains(r#""from":2,"to":8"#),
            "{}",
            t.diagnostics
        );
        assert!(t.diagnostics.contains(r#""line":1,"col":2"#));
    }

    #[test]
    fn overrides_apply() {
        let t = typeset("甲", "format=scroll\npunct = bare");
        assert_eq!(t.page_count(), 1);
        assert_eq!(t.diagnostics, "[]");
        let bad = typeset("甲", "columns=90");
        assert!(bad.diagnostics.contains("columns"));
    }
}
