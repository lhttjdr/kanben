//! 字格坐标 → SVG。
//!
//! 冊頁版式每葉一张 SVG：右半葉、版心、左半葉。長卷版式只有一张。
//! SVG 不内嵌样式，样式见 [`STYLE`]（页面里引入一次即可）；
//! 颜色、字体都是 CSS 变量，可以在页面上改。

pub mod ids;
pub mod numerals;
mod page;
mod svg;

use crate::layout::{Format, Layout};
pub use page::Geometry;

/// SVG 用到的样式。
pub const STYLE: &str = include_str!("style.css");

/// 渲染成若干页 SVG（冊頁每葉一页；長卷只有一页）。
pub fn render(layout: &Layout) -> Vec<String> {
    let mut geo = Geometry::new(&layout.options);
    // 原版長卷總比正文多一行空行。
    geo.scroll_cols = layout.columns + 1;
    let pages = match layout.options.format {
        Format::Leaves => geo.leaf_pages(layout.columns),
        Format::Scroll => 1,
    };
    (0..pages).map(|p| svg::page(layout, &geo, p)).collect()
}

/// 单独打开也能看的 SVG（内嵌样式）。
pub fn standalone(svg: &str) -> String {
    svg.replacen(
        "<g class=\"kb-root\">",
        &format!("<style>{STYLE}</style><g class=\"kb-root\">"),
        1,
    )
}

/// 完整的 HTML 页面，供命令行导出、打印。
/// `font_loader` 是字體加载脚本的地址（如 `https://…/fontload.js`）；不给则用系統字體。
pub fn html(title: &str, pages: &[String], font_loader: Option<&str>) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"zh-Hant\"><head><meta charset=\"utf-8\">");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    if let Some(src) = font_loader {
        // 只登记页面上用到的字，浏览器按需下载。
        out.push_str(&format!(
            "<script type=\"module\">import {{ loadFonts, fitCompose }} from \"{}\"; loadFonts(document.body.textContent); fitCompose(); document.fonts.addEventListener(\"loadingdone\", () => fitCompose());</script>",
            svg::escape(src)
        ));
    }
    out.push_str(&format!(
        "<title>{}</title><style>{STYLE}",
        svg::escape(if title.is_empty() { "刊本" } else { title })
    ));
    out.push_str(
        "body{margin:0;background:#d9d2c3}\
         .kb-pages{display:flex;flex-direction:row-reverse;gap:24px;padding:24px;overflow-x:auto;width:max-content;min-width:100%;box-sizing:border-box}\
         .kb-pages svg{flex:none;box-shadow:0 1px 6px rgba(0,0,0,.25);height:calc(100vh - 48px);width:auto}\
         @media print{body{background:none}.kb-pages{display:block;padding:0}.kb-pages svg{display:block;box-shadow:none;width:100%;height:auto;page-break-after:always}}",
    );
    out.push_str("</style></head><body><div class=\"kb-pages\">\n");
    for p in pages {
        out.push_str(p);
        out.push('\n');
    }
    out.push_str("</div></body></html>\n");
    out
}
