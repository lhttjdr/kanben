//! 生成单页 SVG。

use std::fmt::Write;

use super::ids;
use super::numerals::hanzi;
use super::page::{FRAME_PAD, Geometry, MARGIN};
use crate::layout::{Deco, Format, Glyph, Layout};
use crate::syntax::ast::{EmphMark, Shape};

/// SVG 坐标：一个字 = 100。线宽直接用这个单位写。
const U: f32 = 100.0;
/// 屏幕上一个字多少像素（SVG 的默认尺寸）。
const PX: f32 = 28.0;
/// 字号相对字格的比例，留一点字间距。
const FONT: f32 = 0.92;

fn n(v: f32) -> String {
    let r = (v * U * 10.0).round() / 10.0;
    if r == r.trunc() {
        format!("{}", r as i64)
    } else {
        format!("{r:.1}")
    }
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

pub fn page(layout: &Layout, geo: &Geometry, page: i32) -> String {
    let (w, h) = (geo.width(), geo.height());
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" class="kb" viewBox="0 0 {} {}" width="{:.0}" height="{:.0}" data-page="{}"><g class="kb-root">"#,
        n(w),
        n(h),
        w * PX,
        h * PX,
        page + 1
    );
    let _ = write!(
        s,
        r#"<rect class="paper" x="0" y="0" width="{}" height="{}"/>"#,
        n(w),
        n(h)
    );
    match geo.format {
        Format::Leaves => leaf_frame(&mut s, geo, &layout.options, page),
        Format::Scroll => scroll_frame(&mut s, geo),
    }

    let top = geo.text_top();
    let here = |col: i32| {
        let (p, cx) = geo.locate(col);
        (p == page).then_some(cx)
    };

    // 先画填充的底（陰文），再画字，最后画綫和點。
    for d in &layout.decos {
        if let Some(cx) = here(d.col()) {
            deco(&mut s, d, cx, top, true);
        }
    }
    for g in &layout.glyphs {
        if let Some(cx) = here(g.col) {
            glyph(&mut s, g, cx, top);
        }
    }
    for d in &layout.decos {
        if let Some(cx) = here(d.col()) {
            deco(&mut s, d, cx, top, false);
        }
    }
    s.push_str("</g></svg>");
    s
}

fn vline(s: &mut String, x: f32, y0: f32, y1: f32, width: f32) {
    let _ = write!(
        s,
        r#"<line class="rule" x1="{x}" y1="{}" x2="{x}" y2="{}" stroke-width="{width}"/>"#,
        n(y0),
        n(y1),
        x = n(x)
    );
}

fn hline(s: &mut String, x0: f32, x1: f32, y: f32, width: f32) {
    let _ = write!(
        s,
        r#"<line class="rule" x1="{}" y1="{y}" x2="{}" y2="{y}" stroke-width="{width}"/>"#,
        n(x0),
        n(x1),
        y = n(y)
    );
}

/// 四周雙邊：外粗内细。
fn double_box(s: &mut String, x0: f32, y0: f32, x1: f32, y1: f32) {
    let _ = write!(
        s,
        r#"<rect class="rule" x="{}" y="{}" width="{}" height="{}" stroke-width="2.5"/>"#,
        n(x0),
        n(y0),
        n(x1 - x0),
        n(y1 - y0)
    );
    let o = 0.1;
    let _ = write!(
        s,
        r#"<rect class="rule" x="{}" y="{}" width="{}" height="{}" stroke-width="7"/>"#,
        n(x0 - o),
        n(y0 - o),
        n(x1 - x0 + 2.0 * o),
        n(y1 - y0 + 2.0 * o)
    );
}

fn leaf_frame(s: &mut String, geo: &Geometry, o: &crate::layout::Options, page: i32) {
    let w = geo.width();
    let (ft, fb) = (geo.frame_top(), geo.frame_bottom());
    double_box(s, MARGIN, ft, w - MARGIN, fb);
    let (b0, b1) = geo.banxin();
    for j in 1..geo.columns {
        vline(s, w - MARGIN - j as f32 * geo.col_w, ft, fb, 1.5);
        vline(s, b0 - j as f32 * geo.col_w, ft, fb, 1.5);
    }
    vline(s, b0, ft, fb, 2.0);
    vline(s, b1, ft, fb, 2.0);

    // 版心：書名、魚尾、卷名、葉碼，底下是堂號。
    let bc = (b0 + b1) / 2.0;
    s.push_str(r#"<g class="banxin nosel">"#);
    let mut y = ft + FRAME_PAD;
    y = vertical_text(s, &o.title, bc, y, 0.62);
    y += 0.25;
    let bw = geo.banxin_w * 0.9;
    let _ = write!(
        s,
        r#"<polygon class="rule-fill" points="{},{} {},{} {},{} {},{} {},{}"/>"#,
        n(bc - bw / 2.0),
        n(y),
        n(bc + bw / 2.0),
        n(y),
        n(bc + bw / 2.0),
        n(y + 0.6),
        n(bc),
        n(y + 0.32),
        n(bc - bw / 2.0),
        n(y + 0.6)
    );
    hline(s, b0, b1, y - 0.08, 1.5);
    y += 0.9;
    y = vertical_text(s, &o.volume, bc, y, 0.5);
    y += 0.3;
    vertical_text(s, &hanzi(page as u32 + 1), bc, y, 0.5);
    let studio_len = o.studio.chars().count() as f32;
    if studio_len > 0.0 {
        let step = 0.5 * 1.1;
        let start = fb - FRAME_PAD - studio_len * step;
        hline(s, b0, b1, start - 0.2, 1.5);
        vertical_text(s, &o.studio, bc, start, 0.5);
    }
    s.push_str("</g>");
}

fn scroll_frame(s: &mut String, geo: &Geometry) {
    let w = geo.width();
    let (ft, fb) = (geo.frame_top(), geo.frame_bottom());
    double_box(s, MARGIN, ft, w - MARGIN, fb);
    for j in 1..geo.scroll_cols {
        vline(s, w - MARGIN - j as f32 * geo.col_w, ft, fb, 1.5);
    }
}

/// 竖排一串小字，返回结束时的 y。
fn vertical_text(s: &mut String, text: &str, x: f32, mut y: f32, size: f32) -> f32 {
    let step = size * 1.1;
    for c in text.chars().filter(|c| !c.is_whitespace()) {
        let _ = write!(
            s,
            r#"<text x="{}" y="{}" font-size="{}">{}</text>"#,
            n(x),
            n(y + step / 2.0),
            n(size * FONT),
            escape(&c.to_string())
        );
        y += step;
    }
    y
}

fn glyph(s: &mut String, g: &Glyph, cx: f32, top: f32) {
    let x = cx + g.x;
    let y = top + g.y;
    let size = g.size * FONT;
    let mut class = String::new();
    if let Some(c) = g.color {
        class.push_str(c.css_class());
    }
    if g.hidden {
        class = "cp".into();
    } else if g.inverse {
        if !class.is_empty() {
            class.push(' ');
        }
        class.push_str("inv");
    }
    let class_attr = if class.is_empty() {
        String::new()
    } else {
        format!(r#" class="{class}""#)
    };

    if g.compose
        && let Some(parts) = ids::components(&g.text)
    {
        let (bw, bh) = (size, size * g.stretch);
        // 复制时得到原 IDS 串：放一个透明的整字，部件本身不可选。
        let _ = write!(
            s,
            r#"<text class="cp" x="{}" y="{}" font-size="{}">{}</text><g class="nosel">"#,
            n(x),
            n(y),
            n(size),
            escape(&g.text)
        );
        for (t, r) in parts {
            let px = x - bw / 2.0 + (r.x + r.w / 2.0) * bw;
            let py = y - bh / 2.0 + (r.y + r.h / 2.0) * bh;
            let _ = write!(
                s,
                r#"<text{class_attr} transform="translate({} {}) scale({:.3} {:.3})" font-size="{}">{}</text>"#,
                n(px),
                n(py),
                r.w,
                r.h * g.stretch,
                n(size),
                escape(&t)
            );
        }
        s.push_str("</g>");
        return;
    }
    let text = if g.compose {
        // IDS 不合法：去掉结构符，按普通字排。
        g.text
            .chars()
            .filter(|c| !('\u{2FF0}'..='\u{2FFF}').contains(c))
            .collect()
    } else {
        g.text.clone()
    };
    if (g.stretch - 1.0).abs() > 1e-3 {
        let _ = write!(
            s,
            r#"<text{class_attr} transform="translate({} {}) scale(1 {:.3})" font-size="{}">{}</text>"#,
            n(x),
            n(y),
            g.stretch,
            n(size),
            escape(&text)
        );
    } else {
        let _ = write!(
            s,
            r#"<text{class_attr} x="{}" y="{}" font-size="{}">{}</text>"#,
            n(x),
            n(y),
            n(size),
            escape(&text)
        );
    }
}

fn color_class(c: Option<crate::syntax::ast::Color>) -> &'static str {
    c.map_or("c-ink", |c| c.css_class())
}

/// `filled_pass` 为真时只画填充的底，否则只画其余的。
fn deco(s: &mut String, d: &Deco, cx: f32, top: f32, filled_pass: bool) {
    match *d {
        Deco::Box {
            x,
            half_w,
            y0,
            y1,
            shape,
            filled,
            ..
        } => {
            if filled != filled_pass {
                return;
            }
            let (x0, x1) = (cx + x - half_w, cx + x + half_w);
            let (y0, y1) = (top + y0, top + y1);
            let class = if filled {
                r#"class="ink""#
            } else {
                r#"class="frame" stroke-width="3.5""#
            };
            match shape {
                Shape::Hex => {
                    let c = half_w * 0.6;
                    let xm = cx + x;
                    let _ = write!(
                        s,
                        r#"<polygon {class} points="{},{} {},{} {},{} {},{} {},{} {},{}"/>"#,
                        n(x0),
                        n(y0 + c),
                        n(xm),
                        n(y0),
                        n(x1),
                        n(y0 + c),
                        n(x1),
                        n(y1 - c),
                        n(xm),
                        n(y1),
                        n(x0),
                        n(y1 - c)
                    );
                }
                _ => {
                    let r = if shape == Shape::Round { half_w } else { 0.05 };
                    let _ = write!(
                        s,
                        r#"<rect {class} x="{}" y="{}" width="{}" height="{}" rx="{}"/>"#,
                        n(x0),
                        n(y0),
                        n(x1 - x0),
                        n(y1 - y0),
                        n(r)
                    );
                }
            }
        }
        Deco::Ring {
            x, y, r, filled, ..
        } => {
            if filled != filled_pass {
                return;
            }
            let class = if filled {
                r#"class="ink""#
            } else {
                r#"class="frame" stroke-width="3""#
            };
            let _ = write!(
                s,
                r#"<circle {class} cx="{}" cy="{}" r="{}"/>"#,
                n(cx + x),
                n(top + y),
                n(r)
            );
        }
        _ if filled_pass => {}
        Deco::Line {
            x,
            y0,
            y1,
            wavy,
            color,
            ..
        } => {
            let x = cx + x;
            let (y0, y1) = (top + y0, top + y1);
            let class = color_class(color);
            if wavy {
                let mut path = format!("M{} {}", n(x), n(y0));
                let (amp, half) = (0.05, 0.12);
                let mut y = y0;
                let mut side = 1.0;
                while y + half <= y1 + 1e-3 {
                    let _ = write!(
                        path,
                        " Q{} {} {} {}",
                        n(x + amp * side * 2.0),
                        n(y + half / 2.0),
                        n(x),
                        n(y + half)
                    );
                    y += half;
                    side = -side;
                }
                let _ = write!(
                    s,
                    r#"<path class="line {class}" d="{path}" stroke-width="3"/>"#
                );
            } else {
                let _ = write!(
                    s,
                    r#"<line class="line {class}" x1="{x}" y1="{}" x2="{x}" y2="{}" stroke-width="3.5"/>"#,
                    n(y0),
                    n(y1),
                    x = n(x)
                );
            }
        }
        Deco::Mark {
            x,
            y,
            size,
            mark,
            color,
            ..
        } => {
            let class = color_class(color);
            let (x, y) = (cx + x, top + y);
            match mark {
                EmphMark::Circle => {
                    let _ = write!(
                        s,
                        r#"<circle class="line {class}" cx="{}" cy="{}" r="{}" stroke-width="2.5"/>"#,
                        n(x),
                        n(y),
                        n(size * 0.08)
                    );
                }
                EmphMark::Dot => {
                    let _ = write!(
                        s,
                        r#"<circle class="{class}" cx="{}" cy="{}" r="{}"/>"#,
                        n(x),
                        n(y),
                        n(size * 0.065)
                    );
                }
                EmphMark::Sesame | EmphMark::Comma => {
                    let c = if mark == EmphMark::Sesame {
                        '﹅'
                    } else {
                        '﹆'
                    };
                    let _ = write!(
                        s,
                        r#"<text class="{class} nosel" x="{}" y="{}" font-size="{}">{c}</text>"#,
                        n(x),
                        n(y),
                        n(size * 0.42)
                    );
                }
            }
        }
        Deco::Tone { x, y, r, .. } => {
            let _ = write!(
                s,
                r#"<circle class="line c-red" cx="{}" cy="{}" r="{}" stroke-width="2"/>"#,
                n(cx + x),
                n(top + y),
                n(r)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_compact() {
        assert_eq!(n(1.0), "100");
        assert_eq!(n(0.1234), "12.3");
    }

    #[test]
    fn escapes() {
        assert_eq!(escape("<a&b>"), "&lt;a&amp;b&gt;");
    }
}
