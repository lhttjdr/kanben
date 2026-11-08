//! 刊本：古籍竖排排版引擎。
//!
//! 三层，各自独立：
//! - [`syntax`]：源码 → 语法树（带出错位置）
//! - [`layout`]：语法树 → 字格坐标（每个字落在哪一行、哪一格）
//! - [`render`]：字格坐标 → SVG 页面

pub mod classic;
pub mod layout;
pub mod render;
pub mod syntax;

use layout::Options;
use syntax::span::Diagnostic;

/// 一次完整排版的结果。
pub struct Output {
    /// 每页一张 SVG。
    pub pages: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
    /// 实际使用的版式（文件头 + 覆盖项）。
    pub options: Options,
}

/// 源码 → SVG。`overrides` 里的「鍵 = 值」会覆盖文件头的设定。
pub fn typeset(src: &str, overrides: &[(&str, &str)]) -> Output {
    let parsed = syntax::parse(src);
    let mut diagnostics = parsed.diagnostics;
    let mut options = Options::from_meta(&parsed.document.meta, &mut diagnostics);
    for (k, v) in overrides {
        if let Err(e) = options.set(k, v) {
            diagnostics.push(Diagnostic::error(syntax::span::Span::default(), e));
        }
    }
    let laid = layout::layout(&parsed.document, options.clone(), &mut diagnostics);
    diagnostics.sort_by_key(|d| d.span.start);
    Output {
        pages: render::render(&laid),
        diagnostics,
        options,
    }
}

/// 按可用高度算每行字數：让一个字大约 `cell_px` 像素高，结果限制在 4–60。
/// `available_px` 是 SVG 要占的高度（整页高，含天頭、地腳）。
pub fn fit_cells(src: &str, overrides: &[(&str, &str)], available_px: f32, cell_px: f32) -> i32 {
    let parsed = syntax::parse(src);
    let mut ignored = Vec::new();
    let mut options = Options::from_meta(&parsed.document.meta, &mut ignored);
    for (k, v) in overrides {
        let _ = options.set(k, v);
    }
    layout::adjust(&parsed.document, &mut options);
    let geo = render::Geometry::new(&options);
    let extra = geo.height() - options.cells as f32;
    let cells = (available_px / cell_px.max(1.0) - extra).floor();
    if cells.is_finite() {
        (cells as i32).clamp(4, 60)
    } else {
        options.cells
    }
}

/// 复制出来的文字：直排字形还原成普通标点。
pub fn plain_text(copied: &str) -> String {
    copied.chars().map(layout::token::horizontal_form).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_cells_tracks_height() {
        // 天頭 1 + 留白、地腳等共 4 字；840px / 30px = 28 字高 → 每行 24 字。
        let n = fit_cells("甲", &[], 840.0, 30.0);
        assert_eq!(n, 24);
        assert!(fit_cells("甲", &[], 1680.0, 30.0) > n);
        assert_eq!(fit_cells("甲", &[], 10.0, 30.0), 4);
        assert_eq!(fit_cells("\\setup[head=0]\n甲", &[], 840.0, 30.0), 25);
    }

    #[test]
    fn compose_copies_as_ids() {
        let out = typeset(r"\compose{⿰木鬼}", &[]);
        assert!(out.pages[0].contains(r#"class="cp""#));
        assert!(out.pages[0].contains(">⿰木鬼</text>"));
    }

    #[test]
    fn plain_text_restores_punctuation() {
        assert_eq!(plain_text("曰︓﹁天︒﹂"), "曰：「天。」");
    }
}
