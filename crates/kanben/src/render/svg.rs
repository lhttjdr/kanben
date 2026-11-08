//! 生成单页 SVG。

use std::fmt::Write;

use super::ids;
use super::numerals::hanzi;
use super::page::{FRAME_PAD, Geometry, MARGIN};
use crate::layout::flatten::rod_digit;
use crate::layout::{Deco, Format, Glyph, Layout, Role};
use crate::syntax::ast::{EmphMark, Shape};

/// SVG 坐标：一个字 = 100。线宽直接用这个单位写。
const U: f32 = 100.0;
/// 屏幕上一个字多少像素（SVG 的默认尺寸）。
const PX: f32 = 28.0;
/// 字号相对字格的比例，留一点字间距。
const FONT: f32 = 1.0;
/// 组字部件相对其格子的放大比例。
const COMPOSE_FILL: f32 = 1.12;

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
        r#"<svg xmlns="http://www.w3.org/2000/svg" class="kb f-{}" viewBox="0 0 {} {}" width="{:.0}" height="{:.0}" data-page="{}"><g class="kb-root">"#,
        layout.options.font.name(),
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
        Format::Scroll => scroll_frame(&mut s, geo, layout.options.numbers),
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
        r#"<rect class="rule" x="{}" y="{}" width="{}" height="{}" stroke-width="3"/>"#,
        n(x0),
        n(y0),
        n(x1 - x0),
        n(y1 - y0)
    );
    let o = 0.14;
    let _ = write!(
        s,
        r#"<rect class="rule" x="{}" y="{}" width="{}" height="{}" stroke-width="6.5"/>"#,
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
        vline(s, w - MARGIN - j as f32 * geo.col_w, ft, fb, 3.0);
        vline(s, b0 - j as f32 * geo.col_w, ft, fb, 3.0);
    }
    if o.numbers {
        for j in 0..geo.columns {
            let n = j as u32 + 1;
            col_number(s, n, w - MARGIN - (j as f32 + 0.5) * geo.col_w, fb + 0.45);
            col_number(s, n, b0 - (j as f32 + 0.5) * geo.col_w, fb + 0.45);
        }
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

fn scroll_frame(s: &mut String, geo: &Geometry, numbers: bool) {
    let w = geo.width();
    let (ft, fb) = (geo.frame_top(), geo.frame_bottom());
    double_box(s, MARGIN, ft, w - MARGIN, fb);
    for j in 1..geo.scroll_cols {
        vline(s, w - MARGIN - j as f32 * geo.col_w, ft, fb, 3.0);
    }
    if numbers {
        for j in 0..geo.scroll_cols {
            let x = w - MARGIN - (j as f32 + 0.5) * geo.col_w;
            col_number(s, j as u32 + 1, x, fb + 0.45);
        }
    }
}

/// 行號：版框下方的算籌數碼，横着排，高位在左。
fn col_number(s: &mut String, n: u32, x: f32, y: f32) {
    let digits: Vec<u8> = n.to_string().bytes().map(|b| b - b'0').collect();
    let size = 0.42;
    let step = size * 0.62;
    let x0 = x - step * (digits.len() as f32 - 1.0) / 2.0;
    s.push_str(r#"<g class="colnum nosel">"#);
    for (i, &d) in digits.iter().enumerate() {
        let vertical = (digits.len() - 1 - i).is_multiple_of(2);
        rod_lines(s, d, vertical, x0 + i as f32 * step, y, size);
    }
    s.push_str("</g>");
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

/// 浏览器竖排时直立的汉字比字格中线偏右、偏上一点；照原版实测补上，
/// 墨迹才和原版重合（正文右 0.09、上 0.035，字号越小按比例越小）。
const GLYPH_DX: f32 = 0.09;
const GLYPH_DY: f32 = -0.035;

fn glyph(s: &mut String, g: &Glyph, cx: f32, top: f32) {
    let (dx, dy) = match g.role {
        Role::Punct => (0.0, 0.0),
        Role::Main => (GLYPH_DX * g.size, GLYPH_DY * g.size),
        _ => (0.0, GLYPH_DY * g.size),
    };
    let x = cx + g.x + dx;
    let y = top + g.y + dy;
    let size = g.size * FONT;
    let mut class = String::new();
    if let Some(c) = g.color {
        class.push_str(c.css_class());
    }
    if g.bold {
        if !class.is_empty() {
            class.push(' ');
        }
        class.push('b');
    }
    // 旁批、眉批用楷書。
    if matches!(g.role, Role::Side | Role::Head) {
        if !class.is_empty() {
            class.push(' ');
        }
        class.push_str("kai");
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

    if g.role == Role::Punct && !g.hidden {
        judou(s, g, x, y);
        return;
    }
    let mut chars = g.text.chars();
    if let (Some(c), None) = (chars.next(), chars.next())
        && let Some((d, vertical)) = rod_digit(c)
    {
        rod(s, &g.text, d, vertical, x, y, size, &class);
        return;
    }
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
        // 部件的目标框写在 data-box 里，页面脚本按字形实际的墨迹把部件填满这个框
        // （见 web/fontload.js 的 fitCompose）；没有脚本时按下面的缩放近似。
        let part_class = if class.is_empty() {
            "idsp".to_string()
        } else {
            format!("idsp {class}")
        };
        for (t, r) in parts {
            let (bx, by) = (x - bw / 2.0 + r.x * bw, y - bh / 2.0 + r.y * bh);
            let (pw, ph) = (r.w * bw, r.h * bh);
            let _ = write!(
                s,
                r#"<text class="{part_class}" data-box="{} {} {} {}" transform="translate({} {}) scale({:.3} {:.3})" font-size="{}">{}</text>"#,
                n(bx),
                n(by),
                n(pw),
                n(ph),
                n(bx + pw / 2.0),
                n(by + ph / 2.0),
                (r.w * COMPOSE_FILL).min(1.0),
                (r.h * COMPOSE_FILL).min(1.0) * g.stretch,
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

/// 句讀：画成小圈、小點、小鉤，不借字体里的標點字形（那些字形偏在字身一角）。
/// (x, y) 是墨迹的中心，由排版器按原版实测定好；这里只管画形状，尺寸也照原版：
/// 。外径 0.25，，约 0.22×0.24，、约 0.24×0.26，：的点径 0.12，「」约 0.44×0.26。
/// 原字作透明文字留给复制。
fn judou(s: &mut String, g: &Glyph, x: f32, y: f32) {
    // 夾注（0.8 字号）的標點按比例缩小。
    let z = g.size.min(1.0);
    let color = g.color.map_or("c-red", |c| c.css_class());
    let _ = write!(
        s,
        r#"<text class="cp" x="{}" y="{}" font-size="{}">{}</text><g class="jd nosel {color}">"#,
        n(x),
        n(y),
        n(z * 0.3),
        escape(&g.text)
    );
    let pt = |dx: f32, dy: f32| format!("{} {}", n(x + dx * z), n(y + dy * z));
    let mut stroke = |d: String, w: f32| {
        let _ = write!(
            s,
            r#"<path class="s" d="{d}" stroke-width="{}"/>"#,
            n(w * z)
        );
    };
    let circle = |dx: f32, dy: f32, r: f32| {
        format!(
            "M{} a{r} {r} 0 1 0 {d} 0 a{r} {r} 0 1 0 -{d} 0",
            pt(dx - r, dy),
            r = n(r * z),
            d = n(2.0 * r * z)
        )
    };
    // 逗號：圆头在右上，尾向左下甩出，宽略大于高。
    let comma = |dy: f32| {
        (
            circle(0.03, dy - 0.045, 0.03),
            format!(
                "M{} Q{} {}",
                pt(0.075, dy - 0.03),
                pt(0.06, dy + 0.07),
                pt(-0.1, dy + 0.095)
            ),
        )
    };
    match g.text.chars().next().unwrap_or(' ') {
        '。' | '︒' | '．' | '.' => stroke(circle(-0.025, 0.02, 0.11), 0.03),
        '，' | '︐' | ',' => {
            let (head, tail) = comma(0.0);
            stroke(head, 0.07);
            stroke(tail, 0.04);
        }
        '、' | '︑' => stroke(format!("M{} L{}", pt(-0.13, -0.06), pt(0.04, 0.1)), 0.085),
        '：' | '︓' | ':' => {
            stroke(circle(0.025, -0.24, 0.03), 0.06);
            stroke(circle(0.025, 0.01, 0.03), 0.06);
        }
        '；' | '︔' | ';' => {
            stroke(circle(0.02, -0.24, 0.03), 0.06);
            let (head, tail) = comma(0.06);
            stroke(head, 0.07);
            stroke(tail, 0.04);
        }
        // 前引號 ┐、後引號 └；雙引號在里面再套一层。
        '「' | '﹁' | '“' | '‘' => stroke(
            format!(
                "M{} L{} L{}",
                pt(-0.205, -0.115),
                pt(0.205, -0.115),
                pt(0.205, 0.115)
            ),
            0.045,
        ),
        '『' | '﹃' => {
            stroke(
                format!(
                    "M{} L{} L{}",
                    pt(-0.205, -0.115),
                    pt(0.205, -0.115),
                    pt(0.205, 0.115)
                ),
                0.04,
            );
            stroke(
                format!(
                    "M{} L{} L{}",
                    pt(-0.205, -0.03),
                    pt(0.12, -0.03),
                    pt(0.12, 0.115)
                ),
                0.04,
            );
        }
        '」' | '﹂' | '”' | '’' => stroke(
            format!(
                "M{} L{} L{}",
                pt(-0.205, -0.115),
                pt(-0.205, 0.115),
                pt(0.205, 0.115)
            ),
            0.045,
        ),
        '』' | '﹄' => {
            stroke(
                format!(
                    "M{} L{} L{}",
                    pt(-0.205, -0.115),
                    pt(-0.205, 0.115),
                    pt(0.205, 0.115)
                ),
                0.04,
            );
            stroke(
                format!(
                    "M{} L{} L{}",
                    pt(-0.12, -0.115),
                    pt(-0.12, 0.03),
                    pt(0.205, 0.03)
                ),
                0.04,
            );
        }
        // 問號、嘆號等：与字心齐平，用字形。
        other => {
            let _ = write!(
                s,
                r#"<text class="f" x="{}" y="{}" font-size="{}">{}</text>"#,
                n(x - 0.017 * z),
                n(y - 0.288 * z),
                n(z * 0.85),
                escape(&other.to_string())
            );
        }
    }
    s.push_str("</g>");
}

/// 算籌：用直綫画，复制时得到 Unicode 算籌字符。
#[allow(clippy::too_many_arguments)]
fn rod(s: &mut String, text: &str, d: u8, vertical: bool, x: f32, y: f32, size: f32, class: &str) {
    let _ = write!(
        s,
        r#"<text class="cp" x="{}" y="{}" font-size="{}">{}</text><g class="rod nosel {class}">"#,
        n(x),
        n(y),
        n(size),
        escape(text)
    );
    rod_lines(s, d, vertical, x, y, size);
    s.push_str("</g>");
}

/// 一位算籌：一至五是并排（縱）或叠起（橫）的筹，六至九上面一筹当五；零是圈。
fn rod_lines(s: &mut String, d: u8, vertical: bool, x: f32, y: f32, size: f32) {
    if d == 0 {
        let _ = write!(
            s,
            r#"<circle cx="{}" cy="{}" r="{}"/>"#,
            n(x),
            n(y),
            n(size * 0.3)
        );
        return;
    }
    let h = size * 0.36;
    let mut line = |x0: f32, y0: f32, x1: f32, y1: f32| {
        let _ = write!(
            s,
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
            n(x + x0),
            n(y + y0),
            n(x + x1),
            n(y + y1)
        );
    };
    let spread = |k: u8| -> Vec<f32> {
        let k = f32::from(k);
        (0..k as u8)
            .map(|i| {
                if k == 1.0 {
                    0.0
                } else {
                    -h * 0.8 + 1.6 * h * f32::from(i) / (k - 1.0)
                }
            })
            .collect()
    };
    match (vertical, d) {
        // 縱式：一至五是并排的竖筹；六至九上面一横当五。
        (true, 1..=5) => spread(d).into_iter().for_each(|p| line(p, -h, p, h)),
        (true, _) => {
            line(-h, -h, h, -h);
            spread(d - 5)
                .into_iter()
                .for_each(|p| line(p, -h * 0.55, p, h));
        }
        // 橫式：一至五是叠起的横筹；六至九上面一竖当五。
        (false, 1..=5) => spread(d).into_iter().for_each(|p| line(-h, p, h, p)),
        (false, _) => {
            line(0.0, -h, 0.0, -h * 0.1);
            spread(d - 5)
                .into_iter()
                .for_each(|p| line(-h, h * 0.2 + p * 0.45, h, h * 0.2 + p * 0.45));
        }
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
                Shape::Flower => {
                    // 花頭：两端作如意云头，中间尖、两肩圆。
                    let w = x1 - x0;
                    let c = (half_w * 0.75).min((y1 - y0) / 3.0);
                    let xm = cx + x;
                    let _ = write!(
                        s,
                        r#"<path {class} d="M{a} {b} Q{a} {top} {l} {sh} Q{lm} {y0} {xm} {y0} Q{rm} {y0} {r} {sh} Q{x1} {top} {x1} {b} L{x1} {bb} Q{x1} {bot} {r} {shb} Q{rm} {y1} {xm} {y1} Q{lm} {y1} {l} {shb} Q{a} {bot} {a} {bb} Z"/>"#,
                        a = n(x0),
                        b = n(y0 + c),
                        top = n(y0 + c * 0.25),
                        l = n(x0 + w * 0.25),
                        sh = n(y0 + c * 0.45),
                        lm = n(xm - w * 0.12),
                        y0 = n(y0),
                        xm = n(xm),
                        rm = n(xm + w * 0.12),
                        r = n(x1 - w * 0.25),
                        x1 = n(x1),
                        bb = n(y1 - c),
                        bot = n(y1 - c * 0.25),
                        shb = n(y1 - c * 0.45),
                        y1 = n(y1),
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
            x,
            y,
            r,
            filled,
            hex,
            ..
        } => {
            if filled != filled_pass {
                return;
            }
            let class = if filled {
                r#"class="ink""#
            } else {
                r#"class="frame" stroke-width="3""#
            };
            let (x, y) = (cx + x, top + y);
            if hex {
                // 疊燈籠：上下尖的六角，一字一个，上下相接。
                let w = r * 0.9;
                let c = r * 0.4;
                let _ = write!(
                    s,
                    r#"<polygon {class} points="{},{} {},{} {},{} {},{} {},{} {},{}"/>"#,
                    n(x),
                    n(y - r),
                    n(x + w),
                    n(y - r + c),
                    n(x + w),
                    n(y + r - c),
                    n(x),
                    n(y + r),
                    n(x - w),
                    n(y + r - c),
                    n(x - w),
                    n(y - r + c)
                );
            } else {
                let _ = write!(
                    s,
                    r#"<circle {class} cx="{}" cy="{}" r="{}"/>"#,
                    n(x),
                    n(y),
                    n(r)
                );
            }
        }
        Deco::Gap {
            x, half_w, y0, y1, ..
        } => {
            if filled_pass {
                let _ = write!(
                    s,
                    r#"<rect class="paper" x="{}" y="{}" width="{}" height="{}"/>"#,
                    n(cx + x - half_w),
                    n(top + y0),
                    n(half_w * 2.0),
                    n(y1 - y0)
                );
            }
        }
        _ if filled_pass => {}
        Deco::Line {
            x,
            y0,
            y1,
            wavy,
            color,
            width,
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
                    r#"<path class="line {class}" d="{path}" stroke-width="{}"/>"#,
                    n(width)
                );
            } else {
                let _ = write!(
                    s,
                    r#"<line class="line {class}" x1="{x}" y1="{}" x2="{x}" y2="{}" stroke-width="{}"/>"#,
                    n(y0),
                    n(y1),
                    n(width),
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
