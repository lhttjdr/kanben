//! 生成单页 SVG。

use std::fmt::Write;

use super::page::{Geometry, PX1};
use crate::layout::{Deco, Figure, Format, Glyph, Layout, Role};
use crate::syntax::ast::{EmphMark, Shape};

/// SVG 坐标：一个字 = 100。线宽直接用这个单位写。
const U: f32 = 100.0;
/// 屏幕上一个字多少像素（SVG 的默认尺寸）。
const PX: f32 = 28.0;
/// 字号相对字格的比例，留一点字间距。
const FONT: f32 = 1.0;

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
    let (w, h) = (geo.width(page), geo.height());
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

/// 冊頁半葉，照原版 切䈎（.版框、.冊䈎絲欄、.版心）。
/// 單頁（右半葉，`page` 為雙數）版心在左、無左外框；雙頁版心在右、無右外框。
/// 紙邊切在版心正中，兩頁拼起来才是整個版心。
fn leaf_frame(s: &mut String, geo: &Geometry, o: &crate::layout::Options, page: i32) {
    let tt = geo.text_top();
    let end = tt + geo.cells as f32;
    let n = geo.columns;
    let cw = geo.col_w;
    let r = geo.first_rule(page);
    let b = |j: i32| r - j as f32 * cw;
    let l = b(n);
    let right_page = page % 2 == 0;
    let (it, ib) = (tt - 10.0 * PX1, end + 8.0 * PX1);
    // 版心盒子左綫。
    let xl = if right_page { l - cw } else { r };
    // 絲欄（含版心）左右界、外框左右界。
    let (s0, s1) = if right_page {
        (xl, r + PX1)
    } else {
        (l, xl + cw + PX1)
    };
    // 外框（含 2px 邊）：單頁無左邊，雙頁無右邊。
    let (ox0, ox1) = if right_page {
        (s0 - 3.0 * PX1, s1 + 5.0 * PX1)
    } else {
        (s0 - 5.0 * PX1, s1 + 3.0 * PX1)
    };
    let (oy0, oy1) = (tt - 15.0 * PX1, end + 13.0 * PX1);
    bar(s, ox0, oy0, ox1, oy0 + 2.0 * PX1);
    bar(s, ox0, oy1 - 2.0 * PX1, ox1, oy1);
    if right_page {
        bar(s, ox1 - 2.0 * PX1, oy0 + 2.0 * PX1, ox1, oy1 - 2.0 * PX1);
    } else {
        bar(s, ox0, oy0 + 2.0 * PX1, ox0 + 2.0 * PX1, oy1 - 2.0 * PX1);
    }
    bar(s, s0, it, s1, it + PX1);
    bar(s, s1 - PX1, it + PX1, s1, ib);
    bar(s, s0, ib - PX1, s1 - PX1, ib);
    for j in 1..=n {
        bar(s, b(j), it + PX1, b(j) + PX1, ib - PX1);
    }
    bar(s, xl, it + PX1, xl + PX1, ib - PX1);
    banxin(s, o, page, xl, tt, ib - PX1);
}

/// 版心：書名、上魚尾、卷名……葉碼、下魚尾、堂號。
/// `xl` 是版心盒子左綫，`bottom` 是版心底綫的上沿。
fn banxin(s: &mut String, o: &crate::layout::Options, page: i32, xl: f32, tt: f32, bottom: f32) {
    let right_page = page % 2 == 0;
    // 單頁版心内容靠右，雙頁靠左（align-items start / end），雙頁的盒子還右移 1px。
    let dx = if right_page { 0.0 } else { PX1 };
    let half = |x: f32| {
        if right_page {
            xl + 24.0 * PX1 + x
        } else {
            xl + PX1 + x
        }
    };
    s.push_str(r#"<g class="banxin nosel">"#);
    // 書名：長於七字就分兩半，單雙頁各寫一半，字小一號。
    let title: Vec<char> = o.title.chars().filter(|c| !c.is_whitespace()).collect();
    let (name, size, cx, len) = if title.len() > 7 {
        let k = title.len().div_ceil(2);
        let part: String = if right_page {
            title[..k].iter().collect()
        } else {
            title[k..].iter().collect()
        };
        (part, 0.8, half(12.0 * PX1), 3.2)
    } else {
        let len = title.len() as f32;
        (title.iter().collect(), 1.0, xl + dx + 24.0 * PX1, len)
    };
    for (k, c) in name.chars().enumerate() {
        banxin_char(s, "bx-title", c, cx, tt + (k as f32 + 0.5) * size, size);
    }
    let d2 = tt + len + PX1;
    bar(s, xl + dx, d2 - PX1, xl + dx + 48.0 * PX1, d2);
    // 下欄（堂號）：上綫 1px、上下各 0.1em 内距，高 3em（0.8 字）。
    // 原版這裏落在小數像素上，瀏覽器畫底色、邊綫時取整；照樣取整（相對字頂）。
    // 字不取整。
    let d3_raw = bottom - (2.4 + 2.0 * 0.08 + PX1);
    let d3 = tt + ((d3_raw - tt) * 30.0).round() / 30.0;
    bar(s, xl + dx, d3, xl + dx + 48.0 * PX1, d3 + PX1);
    let cx = half(12.0 * PX1);
    for (k, c) in o.studio.chars().filter(|c| !c.is_whitespace()).enumerate() {
        banxin_char(
            s,
            "bx-studio",
            c,
            cx,
            d3_raw + PX1 + 0.08 + (k as f32 + 0.5) * 0.8,
            0.8,
        );
    }
    // 魚尾：寬半行、高一行，上下各有 3px 紙色邊。
    let fx = half(0.0);
    let (fw, fh) = (24.0 * PX1, 48.0 * PX1);
    let sign = if right_page { -1.0 } else { 1.0 };
    fish_tail(s, fx, d2 + 3.0 * PX1, fw, fh, sign * 135.0);
    fish_tail(s, fx, d3 - 51.0 * PX1, fw, fh, sign * 45.0);
    // 卷名在上魚尾下，葉碼在下魚尾上，都是 0.8 字。
    for (k, c) in o.volume.chars().filter(|c| !c.is_whitespace()).enumerate() {
        banxin_char(s, "bx-small", c, cx, d2 + 1.6 + (k as f32 + 0.5) * 0.8, 0.8);
    }
    let leaf: Vec<char> = super::numerals::reading(page as u32 / 2 + 1)
        .chars()
        .collect();
    let m = leaf.len();
    for (k, c) in leaf.into_iter().enumerate() {
        banxin_char(
            s,
            "bx-small",
            c,
            cx,
            d3 - 1.6 - ((m - k) as f32 - 0.5) * 0.8,
            0.8,
        );
    }
    s.push_str("</g>");
}

fn banxin_char(s: &mut String, class: &str, c: char, x: f32, y: f32, size: f32) {
    let _ = write!(
        s,
        r#"<text class="{class}" x="{}" y="{}" font-size="{}">{}</text>"#,
        n(x),
        n(y),
        n(size),
        escape(&c.to_string())
    );
}

/// 魚尾：照原版 linear-gradient(角度, 絲欄色 50%, 透明 50% 55%, 絲欄色 55% 60%, 透明 60%)，
/// 换成兩塊多邊形。
fn fish_tail(s: &mut String, x: f32, y: f32, w: f32, h: f32, angle: f32) {
    let a = angle.to_radians();
    let (dx, dy) = (a.sin(), -a.cos());
    let len = (w * dx).abs() + (h * dy).abs();
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    let rect = vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    for (t0, t1) in [(0.0, 0.5), (0.55, 0.6)] {
        let (lo, hi) = ((t0 - 0.5) * len, (t1 - 0.5) * len);
        let proj = |p: &(f32, f32)| (p.0 - cx) * dx + (p.1 - cy) * dy;
        let poly = clip(&clip(&rect, |p| proj(p) - lo), |p| hi - proj(p));
        if poly.len() < 3 {
            continue;
        }
        let pts: Vec<String> = poly
            .iter()
            .map(|p| format!("{},{}", n(p.0), n(p.1)))
            .collect();
        // 原版漸變的分界不抗鋸齒。
        let _ = write!(
            s,
            r#"<polygon class="rule-fill" shape-rendering="crispEdges" points="{}"/>"#,
            pts.join(" ")
        );
    }
}

/// 多邊形按半平面 `f(p) >= 0` 裁剪。
fn clip(poly: &[(f32, f32)], f: impl Fn(&(f32, f32)) -> f32) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let (fp, fq) = (f(&p), f(&q));
        if fp >= 0.0 {
            out.push(p);
        }
        if (fp >= 0.0) != (fq >= 0.0) {
            let t = fp / (fp - fq);
            out.push((p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t));
        }
    }
    out
}

fn bar(s: &mut String, x0: f32, y0: f32, x1: f32, y1: f32) {
    let _ = write!(
        s,
        r#"<rect class="rule-fill" x="{}" y="{}" width="{}" height="{}"/>"#,
        n(x0),
        n(y0),
        n(x1 - x0),
        n(y1 - y0)
    );
}

/// 長卷版框，照原版 #gaeg（2px 邊、3px 內距）、#jiusilan（上、右 1px）、
/// .jienghangsen（每行左、下 1px）。字頂離内綫 9px，離外綫 15px。
/// 原版總比正文多畫一行空行，行數由調用方算好。
fn scroll_frame(s: &mut String, geo: &Geometry, numbers: bool) {
    let tt = geo.text_top();
    let end = tt + geo.cells as f32;
    let cols = geo.scroll_cols.max(1);
    let r0 = geo.first_rule(0);
    let b = |j: i32| r0 - j as f32 * geo.col_w;
    let (r, l) = (b(0), b(cols));
    let (it, ib) = (tt - 10.0 * PX1, end + 8.0 * PX1);
    // 外框
    let (ox0, ox1, oy0, oy1) = (
        l - 5.0 * PX1,
        r + 6.0 * PX1,
        tt - 15.0 * PX1,
        end + 13.0 * PX1,
    );
    bar(s, ox0, oy0, ox1, oy0 + 2.0 * PX1);
    bar(s, ox0, oy1 - 2.0 * PX1, ox1, oy1);
    bar(s, ox0, oy0 + 2.0 * PX1, ox0 + 2.0 * PX1, oy1 - 2.0 * PX1);
    bar(s, ox1 - 2.0 * PX1, oy0 + 2.0 * PX1, ox1, oy1 - 2.0 * PX1);
    // 内綫：上、右，每行的左、下
    bar(s, l, it, r + PX1, it + PX1);
    bar(s, r, it + PX1, r + PX1, ib);
    bar(s, l, ib - PX1, r, ib);
    for j in 1..=cols {
        bar(s, b(j), it + PX1, b(j) + PX1, ib - PX1);
    }
    if numbers {
        for j in 0..cols {
            let x = (b(j) + b(j + 1)) / 2.0 + 0.5 * PX1;
            col_number(s, j as u32 + 1, x, end + 31.6 * PX1);
        }
    }
}

/// 行號：照原版 .hangbiau，「華數碼」字體寫阿拉伯數字，字體自己變成碼子；
/// 18px（0.6 字）、灰色、字寬 120%/位數（一位 80%、全角），个位横。
fn col_number(s: &mut String, n: u32, x: f32, y: f32) {
    let text = n.to_string();
    let digits = text.len();
    let stretch = if digits > 1 {
        120.0 / digits as f32
    } else {
        80.0
    };
    let feature = if digits % 2 == 1 { "rod-h" } else { "rod-v" };
    let one = if digits == 1 { " rod-1" } else { "" };
    let _ = write!(
        s,
        r#"<text class="colnum nosel {feature}{one}" style="font-stretch:{stretch}%" x="{}" y="{}" font-size="60">{text}</text>"#,
        self::n(x),
        self::n(y),
    );
}

/// 原版有字距（letter-spacing）的幾類：字距加在字後，字盒中心比字形中心低半個字距。
/// 排版器給的是字盒中心，這裏換成字形中心（單位：字）。
fn letter_spacing_shift(g: &Glyph) -> f32 {
    match g.role {
        // 雙行夾注與其標點：6px（正文 30px）。
        Role::Note => 0.1,
        Role::Line => 0.1 + 1.0 / 30.0,
        Role::Punct if g.size < 0.9 && !matches!(punct_class(g), "p-nq") => 0.1,
        // 單行小注裏的標點繼承小注的 2px 字距。
        Role::Punct if (g.size - 28.0 / 30.0).abs() < 1e-3 => 1.0 / 30.0,
        // 單行小注：字距 2px 與 `top: 1px` 相抵。
        Role::NoteSingle => 0.0,
        _ => 0.0,
    }
}

/// 字形橫向的細調（字）：原版小字竪排時字形中綫與字盒中心的微小差別，照逐像素比對定出。
fn x_shift(g: &Glyph) -> f32 {
    match g.role {
        Role::NoteSingle => X_SHIFT_SINGLE,
        // 陰文、墨圍裏的字（原版 cite，換了字體）比正文偏右半個像素。
        Role::Main if g.encl => X_SHIFT_ENCL,
        _ => 0.0,
    }
}
const X_SHIFT_ENCL: f32 = 0.5 / 30.0;
const X_SHIFT_SINGLE: f32 = 0.0;

/// 碼子、阿拉伯數字、數字卦（原版 cnumMZ、lrtb o*d、divine3）。
fn numeral(s: &mut String, g: &Glyph, kind: &str, class: &str, x: f32, y: f32, size: f32) {
    let extra = if class.is_empty() || class == kind {
        String::new()
    } else {
        format!(" {class}")
    };
    match kind {
        "num-ar" => {
            let (digits, colon) = match g.text.strip_suffix(':') {
                Some(d) => (d, true),
                None => (g.text.as_str(), false),
            };
            // 三、四位的壓扁到 70%，字距 -1px。逐像素比對：比字心高 1px。
            let squeeze = digits.len() >= 3;
            let y = y - size / 30.0;
            let tf = if squeeze {
                format!(
                    r#" transform="translate({} {}) scale(0.7 1)" letter-spacing="-1""#,
                    n(x),
                    n(y)
                )
            } else {
                String::new()
            };
            let (px, py) = if squeeze { (0.0, 0.0) } else { (x, y) };
            let _ = write!(
                s,
                r#"<text class="num-ar{extra}"{tf} x="{}" y="{}" font-size="{}">{}</text>"#,
                n(px),
                n(py),
                n(size * 0.9),
                escape(digits)
            );
            // 冒號：疊在本格與下一格之間，右移 0.145 字、上提 0.15 字。
            if colon {
                let _ = write!(
                    s,
                    r#"<text class="num-colon{extra}" x="{}" y="{}" font-size="{}">:</text>"#,
                    n(x + 0.145 * size),
                    n(y + 0.5 * size - 0.15 * size),
                    n(size)
                );
            }
        }
        _ => {
            let _ = write!(
                s,
                r#"<text class="{kind}{extra}" x="{}" y="{}" font-size="{}">{}</text>"#,
                n(x),
                n(y),
                n(size),
                escape(&g.text)
            );
        }
    }
}

fn glyph(s: &mut String, g: &Glyph, cx: f32, top: f32) {
    let x = cx + g.x + x_shift(g);
    let y = top + g.y - letter_spacing_shift(g);
    let size = g.size * FONT;
    // 各類文字的字體、字重照原版（見 style.css 的 g-*、p-* 類）。
    let mut class = String::from(match (g.role, g.level) {
        (Role::Main, 0) => "",
        (Role::Main, 1) => "g-h1",
        (Role::Main, 2) => "g-h2",
        (Role::Main, 3) => "g-h3",
        (Role::Main, 4) => "g-h4",
        (Role::Main, 5) => "g-h5",
        (Role::Main, _) => "g-h6",
        (Role::Note, _) => g.extra.unwrap_or("g-note"),
        (Role::Line, _) => g.extra.unwrap_or("g-line"),
        (Role::Ann, _) => g.extra.unwrap_or(""),
        (Role::NoteSingle, _) => "g-note1",
        (Role::Side, _) => "g-side",
        (Role::Head, _) => "g-head",
        // 平、上、入聲圈原版橫排（lrtb）、24px；去聲圈竪排、30px。
        (Role::Tone, _) if g.text == "〬" => "g-tone",
        (Role::Tone, _) => "g-tone tone-h",
        (Role::Punct, _) => punct_class(g),
    });
    if g.encl && class.is_empty() {
        class.push_str("g-encl");
    }
    if let Some(x) = g.extra
        && class.is_empty()
    {
        class.push_str(x);
    }
    if g.faded {
        if !class.is_empty() {
            class.push(' ');
        }
        class.push_str("o60");
    }
    if let Some(c) = g.color
        && g.role != Role::Punct
    {
        if !class.is_empty() {
            class.push(' ');
        }
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
    let mut class_attr = if class.is_empty() {
        String::new()
    } else {
        format!(r#" class="{class}""#)
    };
    // 段落字色（交替行色）：標點不變色。
    if let Some(p) = g.paint
        && g.role != Role::Punct
        && !g.hidden
        && !g.inverse
    {
        let _ = write!(class_attr, r#" style="fill:{p}""#);
    }

    if g.role == Role::Punct && !g.hidden {
        // 原版的標點按簡體中文取字形。
        let _ = write!(
            s,
            r#"<text{class_attr} lang="zh-Hans-CN" xml:lang="zh-Hans-CN" x="{}" y="{}" font-size="{}">{}</text>"#,
            n(x),
            n(y),
            n(size),
            escape(&g.text)
        );
        return;
    }
    if let Some(kind @ ("num-mz" | "num-ar" | "num-gua")) = g.extra {
        numeral(s, g, kind, &class, x, y, size);
        return;
    }
    if g.rod {
        // 照原版 cnumDr：碼子字橫排在一格裏，字寬壓到 120%/位數（一位用全角字形、80%），
        // 最高位縱橫由 ss01 / ss02 選（个位縱）。
        let digits = g.text.chars().count();
        let vertical_high = (1 + digits).is_multiple_of(2);
        let stretch = if digits > 1 {
            120.0 / digits as f32
        } else {
            80.0
        };
        let feature = if vertical_high { "rod-v" } else { "rod-h" };
        let one = if digits == 1 { " rod-1" } else { "" };
        // 負數：原版 DrNegSgn，斜綫居中疊在數上（同一字體、字寬）。
        if g.extra == Some("num-neg") {
            let _ = write!(
                s,
                r#"<text class="g-rod" style="font-stretch:{}%" x="{}" y="{}" font-size="{}">＼</text>"#,
                stretch,
                n(x),
                n(y),
                n(size),
            );
        }
        let _ = write!(
            s,
            r#"<text class="g-rod {feature}{one}{}" style="font-stretch:{}%" x="{}" y="{}" font-size="{}">{}</text>"#,
            if class.is_empty() {
                String::new()
            } else {
                format!(" {class}")
            },
            stretch,
            n(x),
            n(y),
            n(size),
            escape(&g.text)
        );
        return;
    }
    if g.compose {
        compose(s, g, &class_attr, x, y, size);
        return;
    }
    let text = g.text.clone();
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

/// 組字，照原版 構字 與 .charComp：
/// 只有「結構符 + 兩字」（UTF-16 長三）且結構符是 ⿰⿱⿸ 時拼字，部件是整字按 CSS transform 壓扁；
/// 其餘去掉結構符，各部件原大疊在同一格。
/// 部件盒子寬一行（行寬一半 = 0.8 字）、高一字，所以 ⿱⿸ 的部件比格心偏右。
fn compose(s: &mut String, g: &Glyph, class_attr: &str, x: f32, y: f32, size: f32) {
    let chars: Vec<char> = g.text.chars().collect();
    let three = g.text.encode_utf16().count() == 3;
    // 复制时得到原 IDS 串：放一个透明的整字，部件本身不可选。
    let _ = write!(
        s,
        r#"<text class="cp" x="{}" y="{}" font-size="{}">{}</text><g class="nosel">"#,
        n(x),
        n(y),
        n(size),
        escape(&g.text)
    );
    let h = 0.8 * size;
    // （部件，x 偏移，y 偏移，横缩放，竪缩放），偏移單位是字號。
    let parts: Vec<(char, f32, f32, f32, f32)> = match chars.first() {
        // 逐像素比對另加：⿰ 右 1px，⿱ 上 1px（瀏覽器變形後取整的差別）。
        Some('⿰') if three => vec![
            (chars[1], -0.55 + h / size * 0.4 + PX1, 0.0, 0.4, 1.0),
            (chars[2], 0.55 - h / size * 0.6 + PX1, 0.0, 0.6, 1.0),
        ],
        Some('⿱') if three => vec![
            (chars[1], -0.5 + h / size, -0.3 - PX1, 1.0, 0.5),
            (chars[2], -0.5 + h / size, 0.3 - PX1, 1.0, 0.5),
        ],
        Some('⿸') if three => vec![
            (chars[1], -0.5 + h / size, 0.0, 1.0, 1.0),
            (chars[2], 0.5 - h / size * 0.8, 0.1, 0.8, 0.8),
        ],
        _ => chars
            .iter()
            .filter(|c| !('\u{2FF0}'..='\u{2FFB}').contains(*c))
            .map(|&c| (c, 0.0, 0.0, 1.0, 1.0))
            .collect(),
    };
    for (c, dx, dy, sx, sy) in parts {
        let (px, py) = (x + dx * size, y + dy * size);
        if sx == 1.0 && sy == 1.0 {
            let _ = write!(
                s,
                r#"<text{class_attr} x="{}" y="{}" font-size="{}">{}</text>"#,
                n(px),
                n(py),
                n(size),
                escape(&c.to_string())
            );
        } else {
            let _ = write!(
                s,
                r#"<text{class_attr} transform="translate({} {}) scale({sx} {sy})" font-size="{}">{}</text>"#,
                n(px),
                n(py),
                n(size),
                escape(&c.to_string())
            );
        }
    }
    s.push_str("</g>");
}

/// 遮罩、裁切的編號，整頁唯一。
static NEXT_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn next_id() -> u32 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// 圖形的 SVG 片段（不帶樣式），坐標換成頁面坐標。
fn figure_geometry(f: &Figure, cx: f32, top: f32) -> String {
    match f {
        Figure::Rect {
            x0,
            y0,
            x1,
            y1,
            rx,
            ry,
        } => {
            let r = if *rx > 0.0 || *ry > 0.0 {
                format!(r#" rx="{}" ry="{}""#, n(*rx), n(*ry))
            } else {
                String::new()
            };
            format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}"{r}/>"#,
                n(cx + x0),
                n(top + y0),
                n(x1 - x0),
                n(y1 - y0)
            )
        }
        Figure::Circle { x, y, r } => {
            format!(
                r#"<circle cx="{}" cy="{}" r="{}"/>"#,
                n(cx + x),
                n(top + y),
                n(*r)
            )
        }
        Figure::Ring { x, y, r0, r1 } => {
            let circle = |r: f32| {
                format!(
                    "M{} {} a{r} {r} 0 1 0 {d} 0 a{r} {r} 0 1 0 -{d} 0",
                    n(cx + x - r),
                    n(top + y),
                    r = n(r),
                    d = n(2.0 * r)
                )
            };
            format!(
                r#"<path fill-rule="evenodd" d="{} {}"/>"#,
                circle(*r1),
                circle(*r0)
            )
        }
        Figure::Polygon(pts) => {
            let p: Vec<String> = pts
                .iter()
                .map(|(x, y)| format!("{},{}", n(cx + x), n(top + y)))
                .collect();
            format!(r#"<polygon points="{}"/>"#, p.join(" "))
        }
        Figure::Minus(a, b) => {
            let id = next_id();
            format!(
                r##"<mask id="kbm{id}" maskUnits="userSpaceOnUse"><g fill="#fff">{}</g><g fill="#000">{}</g></mask><g mask="url(#kbm{id})">{}</g>"##,
                figure_geometry(a, cx, top),
                figure_geometry(b, cx, top),
                figure_geometry(a, cx, top)
            )
        }
        Figure::Clip(a, [x0, y0, x1, y1]) => {
            let id = next_id();
            format!(
                r#"<clipPath id="kbc{id}"><rect x="{}" y="{}" width="{}" height="{}"/></clipPath><g clip-path="url(#kbc{id})">{}</g>"#,
                n(cx + x0),
                n(top + y0),
                n(x1 - x0),
                n(y1 - y0),
                figure_geometry(a, cx, top)
            )
        }
    }
}

fn figure_svg(s: &mut String, f: &Figure, class: &str, cx: f32, top: f32) {
    let _ = write!(
        s,
        r#"<g class="{class}">{}</g>"#,
        figure_geometry(f, cx, top)
    );
}

/// 標點用原版的哪一類字體（見 style.css）。
fn punct_class(g: &Glyph) -> &'static str {
    let note = g.size < 0.9;
    match (note, g.text.chars().next().unwrap_or(' ')) {
        (_, '﹅' | '﹆') => "p-em",
        (_, 'ロ') => "p-box",
        (false, '：' | '；') => "p-md",
        (false, '？' | '！') => "p-tl",
        (false, '「' | '」' | '『' | '』' | '“' | '”' | '‘' | '’') => "p-q",
        (false, _) => "p-sm",
        (true, '「' | '」' | '『' | '』' | '“' | '”' | '‘' | '’') => "p-nq",
        (true, _) => "p-n",
    }
}

fn color_class(c: Option<crate::syntax::ast::Color>) -> &'static str {
    c.map_or("c-ink", |c| c.css_class())
}

/// `filled_pass` 为真时只画填充的底，否则只画其余的。
fn deco(s: &mut String, d: &Deco, cx: f32, top: f32, filled_pass: bool) {
    if let Deco::Wave {
        x,
        y0,
        y1,
        text,
        right,
        first,
        last,
        ..
    } = d
    {
        if filled_pass {
            // 字透明、只顯波綫；不可選，免得複製時字重複。
            let _ = write!(
                s,
                r#"<text class="wave{} nosel" aria-hidden="true" x="{}" y="{}" font-size="{}"><tspan>{}</tspan></text>"#,
                if *right { " wave-r" } else { "" },
                n(cx + x),
                n(top + y0),
                n(FONT),
                escape(text)
            );
            // 原版書名號頭尾用紙色小塊蓋去波綫端點（3px 高）。
            if !*right {
                let px = 1.0 / 30.0;
                let cap = |s: &mut String, ya: f32| {
                    let _ = write!(
                        s,
                        r#"<rect class="paper" x="{}" y="{}" width="{}" height="{}"/>"#,
                        n(cx + x - 0.8),
                        n(top + ya),
                        n(0.22),
                        n(3.0 * px)
                    );
                };
                if *first {
                    cap(s, *y0);
                }
                if *last {
                    cap(s, *y1 - 3.0 * px);
                }
            }
        }
        return;
    }
    if let Deco::Shape {
        figure,
        class,
        over,
        ..
    } = d
    {
        if *over != filled_pass {
            figure_svg(s, figure, class, cx, top);
        }
        return;
    }
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
        Deco::Shape { .. } | Deco::Wave { .. } => {}
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
                EmphMark::Sesame | EmphMark::Comma | EmphMark::Box => {
                    let c = match mark {
                        EmphMark::Sesame => '﹅',
                        EmphMark::Comma => '﹆',
                        _ => 'ロ',
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
