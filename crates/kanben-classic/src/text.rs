//! 原版 dlNote.js、main.js 裏的純文本變換，逐個照搬。
//! 每個函數上方注明原版的函數名；正則一律照抄原版源碼（經 [`crate::jsre`] 翻譯）。

use crate::jsre::{replace_str, u16len};
use crate::re;

/// 原版依賴的環境值。原版在頁面加載時量出來（`測字距`、`度行`），這裏由調用方傳入。
#[derive(Clone, Debug)]
pub struct Env {
    /// `調距`：各字體的字距調整（em）。
    pub 正文: f64,
    pub 夾注: f64,
    pub 注中注: f64,
    pub 注中小字: f64,
    pub 敘: f64,
    pub 跋: f64,
    /// `行尾空白長`（em）。
    pub 行尾空白長: f64,
    /// `系統`：Android、iPhone 時夾注標點的樣式不同。
    pub 系統: String,
}

impl Default for Env {
    /// 桌面瀏覽器、原版字體都不在時量得的值。
    fn default() -> Self {
        Env {
            正文: 0.0,
            夾注: -0.25,
            注中注: 0.0,
            注中小字: -1.0,
            敘: 0.0,
            跋: 0.0,
            行尾空白長: 0.2,
            系統: "Linux".into(),
        }
    }
}

/// JS 的 `String(數)`。
pub fn num(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// `標點.橫` 與 `標點.縱`：橫排標點與對應的直排字形。
pub const 標點橫: &str = "，、。．：；？！·《》〈〉「」『』【】〖〗（）［］〔〕…〪〭〫〬_‹›";
/// 其中 `\u{E794}\u{E795}` 是原版自造字體裏〖〗的直排字形（私用區）。
pub const 標點縱: &str =
    "︐︑︒・︓︔︖︕･︽︾︿﹀﹁﹂﹃﹄︻︼\u{E794}\u{E795}︵︶﹇﹈︹︺︙¯ˇˋˆ︳﹤﹥";

fn 橫轉縱(s: &str) -> String {
    let mut s = s.to_string();
    for (h, z) in 標點橫.chars().zip(標點縱.chars()) {
        s = s.replace(h, &z.to_string());
    }
    s
}

fn 縱轉橫(s: &str) -> String {
    let mut s = s.to_string();
    for (h, z) in 標點橫.chars().zip(標點縱.chars()) {
        s = s.replace(z, &h.to_string());
    }
    s
}

// ───────────── 組字 ─────────────

/// `構字`
pub fn 構字(構件: &str) -> String {
    let chars: Vec<char> = 構件.chars().collect();
    // 原版按 UTF-16 長度判斷「三個字」。
    let three = u16len(構件) == 3;
    let 成字 = match chars.first() {
        Some('⿰') if three => format!(
            r#"<span class="cc_LR_L">{}</span><span class="cc_LR_R">{}</span>"#,
            chars[1], chars[2]
        ),
        Some('⿱') if three => format!(
            r#"<span class="cc_TB_T">{}</span><span class="cc_TB_B">{}</span>"#,
            chars[1], chars[2]
        ),
        Some('⿸') if three => format!(
            r#"<span class="cc_TLBR_TL">{}</span><span class="cc_TLBR_BR">{}</span>"#,
            chars[1], chars[2]
        ),
        _ => re!(r"[\u2FF0-\u2FFB]", "g").replace(構件, ""),
    };
    format!(r#"<i class="charComp">{成字}</i>"#)
}

// ───────────── punctuateVH ─────────────

/// `punctuateVH(樣式類, true, 文)` 留下的狀態：抽走的補遺字與拼字，第二步再放回。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct 抽字 {
    pub 拼字: Option<Vec<String>>,
    pub 補遺字: Option<Vec<String>>,
}

/// `punctuateVH(樣式類, true, ctxt)`：夾注裏的標點換成直排字形，並把擴展區字、拼字暫換成佔位字。
pub fn punctuate_vh_1(樣式類: &str, ctxt: &str) -> (String, 抽字) {
    let 拼字 = re!(r"\[\[\?.+?¿\]\]", "g").match_all(ctxt);
    // 佔位字是兼容區漢字 U+FA39、U+FA15（形同「塀」「凞」），不能寫成通用的那兩個字。
    let ctxt = re!(r"\[\[\?.+?¿\]\]", "g").replace(ctxt, "\u{FA39}");
    // 原版用 [\uD800-\uDFFF][\uD800-\uDFFF] 匹配一個代理對，即一個擴展區字。
    let 補遺字 = re!(r"[\u{10000}-\u{10FFFF}]", "g").match_all(&ctxt);
    let mut ctxt = re!(r"[\u{10000}-\u{10FFFF}]", "g").replace(&ctxt, "\u{FA15}");
    if let Some(x) = re!(r"\(\(=(.+?)=\)\)", "g").match_all(&ctxt) {
        for xi in x {
            let xi = 橫轉縱(&xi);
            let xi = re!(r"\(\(=(.+?)=\)\)", "g")
                .replace(&xi, &format!(r#"<small class="{樣式類}">$1</small>"#));
            ctxt = re!(r"\(\(=.+?=\)\)").replace(&ctxt, &xi);
        }
    }
    if let Some(x) = re!(r"““=(.+?)=””", "g").match_all(&ctxt) {
        for xi in x {
            let xi = 橫轉縱(&xi);
            let xi =
                re!(r"““=(.+?)=””", "g").replace(&xi, &format!(r#"<dd class="{樣式類}">$1</dd>"#));
            ctxt = re!(r"““=(.+?)=””").replace(&ctxt, &xi);
        }
    }
    (ctxt, 抽字 { 拼字, 補遺字 })
}

/// `punctuateVH(樣式類, false, …)` 的前半：每個夾注元素的內容把直排標點換回橫排。
pub fn punctuate_vh_2_item(inner: &str) -> String {
    縱轉橫(inner)
}

/// `punctuateVH(樣式類, false, 容器號)` 的後半：把佔位字換回擴展區字與拼字。
pub fn punctuate_vh_2_restore(html: &str, 抽: &抽字) -> String {
    let mut x = html.to_string();
    if let Some(補) = &抽.補遺字 {
        for c in 補 {
            x = re!(r"\u{FA15}").replace(&x, &format!(r#"<span class="extGK_FS">{c}</span>"#));
        }
    }
    if let Some(拼) = &抽.拼字 {
        for p in 拼 {
            let p = re!(r"[\[\?¿\]]", "g").replace(p, "");
            x = re!(r"\u{FA39}").replace(&x, &構字(&p));
        }
    }
    x
}

// ───────────── 眉批 ─────────────

/// `寫眉批`：抽出 `(?\u{2329}眉批\u{232A}樣式號¿)`（括號是 U+2329、U+232A，不是〈〉），正文裏留佔位；返回（正文, 天頭各條 (文字, 類名)）。
pub fn 寫眉批(cv: &str) -> (String, Vec<(String, String)>) {
    let Some(批楎) = re!(r"\(\?\u{2329}.+?\u{232A}\d*¿\)", "g").match_all(cv) else {
        return (cv.to_string(), Vec::new());
    };
    let x = re!(r"\(\?\u{2329}(.+?)\u{232A}\d*¿\)", "g")
        .replace(cv, r#"<span class="rsvnote"></span>"#);
    let notes = 批楎
        .iter()
        .map(|b| {
            let 文 = re!(r"\(\?\u{2329}|\u{232A}\d*¿\)", "g").replace(b, "");
            let 號 = re!(r"\d*(?=¿\))")
                .match_all(b)
                .map(|v| v[0].clone())
                .unwrap_or_default();
            let cls = if 號.is_empty() {
                "miipei".to_string()
            } else {
                format!("miipei mpstl{號}")
            };
            (文, cls)
        })
        .collect();
    (x, notes)
}

/// 天頭的 HTML：原版逐條建 `<div id="mp{i}" class="…">` 再取 innerHTML。
pub fn 天頭(notes: &[(String, String)]) -> String {
    notes
        .iter()
        .enumerate()
        .map(|(i, (t, c))| format!(r#"<div id="mp{i}" class="{c}">{t}</div>"#))
        .collect()
}

// ───────────── toHTML 及其步驟 ─────────────

/// `toHTML`
pub fn to_html(cv: &str, env: &Env) -> String {
    let mut x = 數碼(cv);
    x = 畫表格(&x);
    x = re!(r"\(\(\-(.+?)\-\)\)", "g").replace(&x, r#"<small class="dhgd">$1</small>"#);
    x = re!(r"᚜\[¯(.+?)¯\]᚛", "g").replace(&x, r#"<sup class="rightLine">$1</sup>"#);
    x = re!(r"᚜\[_(.+?)_\]᚛", "g").replace(&x, r#"<sub class="leftLine">$1</sub>"#);
    x = re!(r"᚜\(¯(.+?)¯\)᚛", "g").replace(&x, r#"<small class="rightLCmt">$1</small>"#);
    x = re!(r"᚜\(_(.+?)_\)᚛", "g").replace(&x, r#"<small class="leftLCmt">$1</small>"#);
    x = re!(r"᚜\(=(.+?)¯\|_(.+?)=\)᚛", "g")
        .replace(&x, r#"<small class="twoLCmt">$1<br>$2</small>"#);
    x = re!(r"〔〔\-(.+?)\-〕〕", "g").replace(&x, "<dt>$1</dt>");
    x = 分段(&x);
    x = 分節(&x);
    x = 引詩文(&x);
    x = 分段單兒(&x);
    x = 插圖(&x);
    x = 尾注(&x);
    x = re!("<p><table", "g").replace(&x, "<table");
    x = re!("</table></p>", "g").replace(&x, "</table>");
    x = 特殊段(&x, env);
    x = 渲染段(&x, env);
    x = 兼行(&x);
    if env.正文 != 0.0 {
        x = re!(r#"<p( class=".*?"){0,1}>"#, "g").replace(
            &x,
            &format!(r#"<p$1 style="letter-spacing:{}em">"#, num(env.正文)),
        );
    }
    x = 旁字(&x);
    x = 提取旁句(&x);
    x = 字樣(&x, "圖");
    x = re!(r"\\([﹅﹆•◦●○◒◓◉◎▲△])\{(.+?)\}", "g").replace(
        &x,
        r#"<span style="-webkit-text-emphasis-style:'$1';text-emphasis-style:'$1'">$2</span>"#,
    );
    x = re!(r"\\([﹅﹆•◦●○◒◓◉◎▲△])\[(.*?)\]\{(.+?)\}", "g").replace(
        &x,
        r#"<span style="-webkit-text-emphasis-style:'$1';text-emphasis-style:'$1';-webkit-text-emphasis-color:var(--$2);text-emphasis-color:var(--$2)">$3</span>"#,
    );
    x = re!(r"\[([﹅﹆•◦●○◒◓◉◎▲△■◆])\]", "g")
        .replace(&x, r#"<span class="ryme" data-韵標="$1"></span>"#);
    x = re!(r"\[(\d)([﹅﹆•◦●○◒◓◉◎▲△■◆])\]", "g").replace(
        &x,
        r#"<span class="ryme" data-韵標="$2" style="color:var(--韵抹$1) !important"></span>"#,
    );
    x = re!(r"\[(\d+?)(\d)([﹅﹆•◦●○◒◓◉◎▲△■◆])\]", "g").replace(
        &x,
        r#"<span class="ryme" data-韵標="$3" style="color:var(--韵抹$2) !important"></span>"#,
    );
    x = re!("〚單擡〛", "g").replace(&x, r#"<b class="headup1"></b>"#);
    x = re!("〚雙擡〛", "g").replace(&x, r#"<b class="headup2"></b>"#);
    x = re!("〚三擡〛", "g").replace(&x, r#"<b class="headup3"></b>"#);
    x = re!(r"<p[^<>]*?><\/p>", "g").replace(&x, "");
    x
}

/// `縱中橫碼`
pub fn 縱中橫碼(num_s: &str) -> String {
    let sgn = if num_s.starts_with('-') || num_s.starts_with('−') {
        "＼"
    } else {
        ""
    };
    let cleaned = re!(r"[-−+]", "g").replace(num_s, "");
    let s: Vec<&str> = cleaned.split('.').collect();
    if s.len() > 2 {
        return num_s.to_string();
    }
    let s0 = if s[0].is_empty() { "0" } else { s[0] };
    // 数字都是 ASCII，按字节切即可。
    let l = s0.len();
    let c = l % 4;
    let mut 大數 = String::new();
    if c != 0 {
        大數 = format!(r#"<span class="lrtb o{c}d">{}</span>"#, &s0[..c]);
    }
    let mut k = c;
    while k < l {
        大數 += &format!(
            r#"<span class="lrtb o4d">{}</span>"#,
            &s0[k..(k + 4).min(l)]
        );
        k += 4;
    }
    let Some(s1) = s.get(1) else {
        return format!("{sgn}{大數}");
    };
    let l = s1.len();
    let mut 小數 = String::new();
    let mut k = 0;
    while k < l - l % 4 {
        小數 += &format!(r#"<span class="lrtb o4d">{}</span>"#, &s1[k..k + 4]);
        k += 4;
    }
    if l % 4 != 0 {
        小數 += &format!(r#"<span class="lrtb o{}d">{}</span>"#, l % 4, &s1[k..]);
    }
    format!("{sgn}{大數}又{小數}")
}

/// `整數籌`
pub fn 整數籌(整數: &str, 个位縱: bool) -> (String, String, f64, String) {
    let mut 負號 = String::new();
    let mut n = 整數.to_string();
    if re!("[〇〡-〩]").test(&n) {
        if n.starts_with('＼') {
            負號 = "＼".into();
            n = re!("^＼").replace(&n, "");
        }
    } else {
        // 原版其餘分支（數值）在 數碼 裏用不到。
    }
    let 位數 = u16len(&n);
    let 最高位籌式 = ["cnumDrV", "cnumDrH"];
    let mut 類 = 最高位籌式[(usize::from(个位縱) + 位數) % 2].to_string();
    if 位數 == 1 {
        類 += " num1d";
    }
    let 鬆緊 = 120.0 / 位數 as f64;
    (n, 類, if 位數 > 1 { 鬆緊 } else { 80.0 }, 負號)
}

/// `數碼`
pub fn 數碼(txt: &str) -> String {
    let mut txt = txt.to_string();
    let xs = re!(r"〚筭］.+?［筭〛", "g")
        .match_all(&txt)
        .unwrap_or_default();
    for x in xs {
        let x = re!(r"^〚筭］|［筭〛$", "g").replace(&x, "");
        let x = re!(r"[-−+]{0,1}\d+\.{0,1}\d*", "g")
            .replace_with(&x, |c, _| 縱中橫碼(c.get(0).unwrap().as_str()));
        let x = re!(r">:<", "g").replace(&x, r#"><span class="midColon">:</span><"#);
        let x = re!(r"＼*[〇〡-〩]+", "g").replace_with(&x, |c, _| {
            let (數, 類, 寬窄, 負號) = 整數籌(c.get(0).unwrap().as_str(), true);
            let neg = if 負號.is_empty() {
                String::new()
            } else {
                format!(r#"<span class="DrNegSgn">{負號}</span>"#)
            };
            format!(
                r#"<data class="cnumDr {類}" style="font-stretch:{}%">{neg}{數}</data>"#,
                num(寬窄)
            )
        });
        txt = re!(r"〚筭］.+?［筭〛").replace(&txt, &x);
    }
    txt = re!(r"〚碼子］(.+?)［碼子〛", "g").replace(&txt, r#"<data class="cnumMZ">$1</data>"#);
    re!(r"〚數卦］(.+?)［數卦〛", "g").replace(&txt, r#"<data class="divine3">$1</data>"#)
}

/// `畫表格`
pub fn 畫表格(txt: &str) -> String {
    let Some(mut tbts) = re!(r"┌(.|\r\n|\n\r|\r|\n)+?┘", "gm").match_all(txt) else {
        return txt.to_string();
    };
    let mut 表題 = re!(r"┌.+?┐", "g").match_all(txt).unwrap_or_default();
    let mut txt = txt.to_string();
    let split = |s: &str| -> Vec<String> {
        re!(r"\r\n|\n\r|\r|\n", "g")
            .replace(s, "\u{0}")
            .split('\u{0}')
            .map(String::from)
            .collect()
    };
    #[allow(clippy::needless_range_loop)] // 與原版的下標寫法對應，便於對照
    for i in 0..tbts.len() {
        if let Some(t) = 表題.get_mut(i) {
            *t = re!(r"[┌─┐]", "g").replace(t, "");
        }
        let mut 表行 = split(&tbts[i]);
        let mut tab_htm: Vec<Vec<String>> = vec![Vec::new(); 表行.len()];
        let mut out = String::new();
        let n = 表行.len();
        for h in 1..n.saturating_sub(1) {
            表行[h] = 表行[h].trim().to_string();
            let mut cells: Vec<String> = 表行[h]
                .split(['├', '┼', '│', '┤'])
                .map(String::from)
                .collect();
            cells.remove(0);
            tab_htm[h] = cells;
            if 表行[h].starts_with('├') {
                let ss: Vec<char> = 表行[h].chars().filter(|c| "├┼│┤".contains(*c)).collect();
                // 原版 tt 與 ss 是同一個數組，這裏用同一個可變副本模擬。
                let mut tt: Vec<String> = ss.iter().map(|c| c.to_string()).collect();
                for ll in 1..ss.len() {
                    if tt[ll - 1].is_empty() {
                        continue;
                    }
                    let cur = tt[ll].clone();
                    if cur == "├" {
                        tt[ll - 1] = "td".into();
                        tab_htm[h][ll - 1] = format!("<td>{}</td>", tab_htm[h][ll - 1]);
                    } else if cur == "┼" || cur == "┤" {
                        tt[ll - 1] = "th".into();
                        tab_htm[h][ll - 1] = format!("<th>{}</th>", tab_htm[h][ll - 1]);
                    } else if cur == "│" {
                        if ll == ss.len() - 1 {
                            tt[ll - 1] = "td".into();
                            tab_htm[h][ll - 1] = format!("<td>{}</td>", tab_htm[h][ll - 1]);
                        } else {
                            tt[ll] = String::new();
                            tab_htm[h][ll] = String::new();
                        }
                    }
                }
            }
            表行[h] = format!("<tr>{}</tr>", tab_htm[h].join(""));
            out += &表行[h];
        }
        if let Some(t) = 表題.get(i).filter(|t| !t.is_empty()) {
            out = format!("<caption>{t}</caption>{out}");
        }
        tbts[i] = out;
        txt = re!(r"┌(.|\r\n|\n\r|\r|\n)+?┘", "m")
            .replace(&txt, &format!("<table>{}</table>", tbts[i]));
    }
    txt
}

/// `分段`
pub fn 分段(txt: &str) -> String {
    let mut txt = txt.replace("\r\n", "\n").replace('\r', "\n");
    if let Some(連續) = re!(r"\n{2,}", "g").match_all(&txt) {
        for run in 連續 {
            let 空行 = if run.len() > 2 {
                format!(
                    r#"{}</div>"#,
                    r#"<div class="空行"><br>"#.repeat(run.len() - 2)
                )
            } else {
                String::new()
            };
            txt = re!(r"\n{2,}").replace(&txt, &format!("</p>{空行}<p>"));
        }
    }
    format!("<p>{}</p>", txt.replace('\n', "<br>"))
}

/// `分節`
pub fn 分節(txt: &str) -> String {
    let mut t = re!(r"<p>\{◎◎§", "g").replace(txt, "<section><p>");
    t = re!(r"\{◎◎§<\/p>", "g").replace(&t, "</p><section>");
    t = re!(r"§〷〷\}<\/p>", "g").replace(&t, "</p></section>");
    t = re!(r"<p>§〷〷\}", "g").replace(&t, "</section><p>");
    re!(r"\{◎◎§(.+?)§〷〷\}", "g").replace(&t, "<section>$1</section>")
}

/// `引詩文`
pub fn 引詩文(txt: &str) -> String {
    let mut txt = txt.to_string();
    while let (Some(q1), Some(q2)) = (txt.find("‘‘‘"), txt.find("’’’")) {
        let w = "‘‘‘".len();
        let before = |i: usize| txt[..i].ends_with("<p>");
        let after = |i: usize| txt[i + w..].starts_with("</p");
        if (before(q1) || after(q1)) && (after(q2) || before(q2)) {
            txt = txt
                .replacen("‘‘‘", "<blockquote>", 1)
                .replacen("’’’", "</blockquote>", 1);
        } else {
            txt = txt.replacen("‘‘‘", "<q>", 1).replacen("’’’", "</q>", 1);
        }
    }
    txt = txt
        .replace("<p><blockquote>", "<blockquote><p>")
        .replace("<blockquote></p>", "</p><blockquote>");
    txt.replace("<p></blockquote>", "</blockquote><p>")
        .replace("</blockquote></p>", "</p></blockquote>")
}

/// `分段單兒`
pub fn 分段單兒(txt: &str) -> String {
    let Some(諸) = re!(r"〚[列集]］.+?［[列集]〛", "g").match_all(txt) else {
        return txt.to_string();
    };
    let mut txt = txt.to_string();
    for 單 in 諸 {
        let 單 = re!(r"<p(?=[\s>])", "g").replace(&單, "<li");
        let 單 = re!(r"<\/p>", "g").replace(&單, "</li>");
        txt = re!(r"〚[列集]］.+?［[列集]〛").replace(&txt, &單);
    }
    txt = re!(r"(〚[列集]］)<br>", "g").replace(&txt, "$1");
    txt = re!(r"<br>(?=［[列集]〛)", "g").replace(&txt, "");
    txt = re!(r"<p>〚列］", "g").replace(&txt, r#"<ol class="單兒 分段單兒"><li>"#);
    txt = re!(r"〚列］<\/p>", "g").replace(&txt, r#"</p><ol class="單兒 分段單兒">"#);
    txt = re!(r"［列〛<\/p>", "g").replace(&txt, "</li></ol>");
    txt = re!(r"<p>［列〛", "g").replace(&txt, "</ol><p>");
    txt = re!(r"〚列］(.+?)［列〛}", "g").replace(&txt, r#"<ol class="單兒 分段單兒">$1</ol>"#);
    txt = re!(r"<p>〚集］", "g").replace(&txt, r#"<ul class="單兒 分段單兒"><li>"#);
    txt = re!(r"〚集］<\/p>", "g").replace(&txt, r#"</p><ul class="單兒 分段單兒">"#);
    txt = re!(r"［集〛<\/p>", "g").replace(&txt, "</li></ul>");
    txt = re!(r"<p>［集〛", "g").replace(&txt, "</ul><p>");
    txt = re!(r"〚集］(.+?)［集〛}", "g").replace(&txt, r#"<ul class="單兒 分段單兒">$1</ul>"#);
    re!(r"<li><\/li>", "g").replace(&txt, "")
}

/// `插圖`（地圖設置要聯網取 json，與文本無關，略去）。
pub fn 插圖(txt: &str) -> String {
    if !txt.contains("〚圖］") {
        return txt.to_string();
    }
    let mut 諸圖 = re!(r"〚圖］(.+?)［圖〛", "g")
        .match_all(txt)
        .unwrap_or_default();
    let mut txt = txt.to_string();
    let mut ti = 0;
    let w = "〚圖］".len();
    while let (Some(起), Some(訖)) = (txt.find("〚圖］"), txt.find("［圖〛")) {
        let Some(圖) = 諸圖.get_mut(ti) else { break };
        let before = |i: usize| txt[..i].ends_with("<p>");
        let after = |i: usize| txt[i + w..].starts_with("</p");
        if (before(起) || after(起)) && (after(訖) || before(訖)) {
            *圖 = 圖
                .replacen("〚圖］", "<figure>", 1)
                .replacen("［圖〛", "</figure>", 1);
        } else {
            *圖 = 圖
                .replacen("〚圖］", "<fig-il>", 1)
                .replacen("［圖〛", "</fig-il>", 1);
        }
        *圖 = re!(r"〚圖片］(.+?)［圖片〛", "g").replace(圖, r#"<img src="$1">"#);
        *圖 = re!(r"〚地圖〛(.+?)(?=〚|<\/figure>|<\/fig-il>)", "g")
            // 原版嵌它網站上的地圖頁；這裏改嵌本站 地圖/ 下的同名地圖。
            .replace(
                圖,
                r#"<iframe src="地圖/?圖=$1" id="$1" alt="$1"></iframe>"#,
            );
        let g = 圖.clone();
        txt = re!(r"〚圖］(.+?)［圖〛").replace(&txt, &g);
        ti += 1;
    }
    txt = re!(r"<p>(?=<figure)", "g").replace(&txt, "");
    txt = re!(r"<figure><\/p>", "g").replace(&txt, "</p><figure>");
    txt = re!(r"<p><\/figure>", "g").replace(&txt, "</figure><p>");
    re!(r"<\/figure>(?=<\/p>)", "g").replace(&txt, "</figure>")
}

/// `數.讀百以內合文`
pub fn 讀百以內合文(an: usize) -> String {
    const 字: [&str; 10] = ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    const 諸十: [&str; 9] = [
        "十", "廿", "卅", "卌", "五十", "六十", "七十", "八十", "九十",
    ];
    if an < 10 {
        字[an].into()
    } else if an == 10 {
        "十".into()
    } else if an.is_multiple_of(10) {
        format!("{}十", 字[an / 10])
    } else {
        format!("{}{}", 諸十[an / 10 - 1], 字[an % 10])
    }
}

/// `尾注`
pub fn 尾注(txt: &str) -> String {
    if !re!(r"\(\(#.+?#\)\)", "g").test(txt) {
        return txt.to_string();
    }
    let mut txt = txt.to_string();
    if let Some(x) = re!(r"\(\(##.+?##\)\)", "g").match_all(&txt) {
        let mut cmnts = "<ol class=gCmnt>".to_string();
        for (i, xi) in x.iter().enumerate() {
            let xi = re!(r"^\(\(##|##\)\)$", "g").replace(xi, "");
            cmnts += &format!("<li>{xi}</li>");
            txt = re!(r"\(\(##.+?##\)\)")
                .replace(&txt, &format!(r#"<small class="gCmtN">{}</small>"#, i + 1));
        }
        cmnts += "</ol>";
        txt += &format!("<br>{cmnts}");
    }
    let mut 編號 = 0;
    if re!(r"\(\(#.+?#\)\)", "g").test(&txt) {
        for 單位 in ["section", "blockquote", "ol", "ul", "p"] {
            let re = crate::jsre::Re::new(&format!(r"<{單位}[\s>].+?</{單位}>"), "g");
            let Some(ps) = re.match_all(&txt) else {
                continue;
            };
            for p in ps {
                let Some(x) = re!(r"\(\(#.+?#\)\)", "g").match_all(&p) else {
                    continue;
                };
                let orig = p.clone();
                let mut p = p;
                let mut cmnts = r#"<ol class="pCmnt">"#.to_string();
                for (i, xi) in x.iter().enumerate() {
                    let xi = re!(r"^\(\(#|#\)\)$", "g").replace(xi, "");
                    編號 += 1;
                    let n = 讀百以內合文(i + 1);
                    cmnts += &format!(
                        r#"<li id="pCmtA{編號}"><span class="pCmtAfN"><i class="hideCopy">〘</i>{n}<i class="hideCopy">〙</i></span>{xi}</li>"#
                    );
                    p = re!(r"\(\(#.+?#\)\)").replace(
                        &p,
                        &format!(
                            r#"<i class="hideCopy">〘</i><small class="pCmtN" onmouseover="shTailCmtHr({編號},event)" onmouseout="shTailCmtHr(-1)"><span>{n}</span></small><i class="hideCopy">〙</i>"#
                        ),
                    );
                }
                cmnts += "</ol>";
                p += &cmnts;
                txt = replace_str(&txt, &orig, &p);
            }
        }
    }
    txt
}

/// `特殊段`
pub fn 特殊段(txt: &str, env: &Env) -> String {
    let mut txt = txt.to_string();
    for 戠 in ["綱", "目", "按", "案", "琵琶"] {
        txt = txt.replace(&format!("<p>〚{戠}〛"), &format!(r#"<p class="{戠}">"#));
    }
    txt = re!(r"<p>\*　{6,}(.+?)<\/p>", "g").replace(&txt, "<h6>$1</h6>");
    txt = re!(r"<p>\*　{5}(.+?)<\/p>", "g").replace(&txt, "<h5>$1</h5>");
    txt = re!(r"<p>\*　{4}(.+?)<\/p>", "g").replace(&txt, "<h4>$1</h4>");
    txt = re!(r"<p>\*　{3}(.+?)<\/p>", "g").replace(&txt, "<h3>$1</h3>");
    txt = re!(r"<p>\*　{2}(.+?)<\/p>", "g").replace(&txt, "<h2>$1</h2>");
    txt = re!(r"<p>\*　(.+?)<\/p>", "g").replace(&txt, "<h1>$1</h1>");
    txt = re!(r"〚題下〛(.+?)(?=<\/h\d>)", "g").replace(&txt, r#"<span class="te2hra4">$1</span>"#);
    let sio = if env.敘 == 0.0 {
        r#"<p class="sio4">"#.to_string()
    } else {
        format!(
            r#"<p class="sio4" style="letter-spacing:{}em">"#,
            num(env.敘)
        )
    };
    txt = re!(r"<p>([敘序]:|〚[敘序]〛)", "g").replace(&txt, &sio);
    let buad = if env.跋 == 0.0 {
        r#"<p class="buad8">"#.to_string()
    } else {
        format!(
            r#"<p class="buad8" style="letter-spacing:{}em">"#,
            num(env.跋)
        )
    };
    txt = re!(r"<p>(跋:|〚跋〛)", "g").replace(&txt, &buad);
    txt = re!(r"<p>〚[注註]〛", "g").replace(&txt, r#"<p class="註">"#);
    if let Some(mc) = re!(r"<p>〚目次〛(.+?)<\/p>").match_all(&txt) {
        let mut m = re!(r"<p>〚目次〛(<br>){0,1}").replace(
            &mc[0],
            r#"<ol class="卷內目次"><li><a onclick="滾到節(1)">"#,
        );
        let mut mi = 0;
        while m.contains("<br>") {
            m = re!("<br>").replace(
                &m,
                &format!(r#"</a></li><li><a onclick="滾到節({})">"#, mi + 2),
            );
            mi += 1;
        }
        while m.contains("\t\t\t") {
            m = re!(r"\t{3,}").replace(
                &m,
                &format!(r#"</a></li><li><a onclick="滾到節({})">"#, mi + 2),
            );
            mi += 1;
        }
        m = re!(r"<\/p>").replace(&m, "</a></li></ol>");
        txt = re!(r"<p>〚目次〛(.+?)<\/p>").replace(&txt, &m);
    }
    if let Some(諸) = re!(r"<p>〚[列集]〛(.+?)<\/p>", "g").match_all(&txt) {
        for 單 in 諸 {
            let 籤 = if 單.starts_with("<p>〚列〛") {
                "ol"
            } else {
                "ul"
            };
            let mut 單 = re!(r"<p>〚[列集]〛(<br>){0,1}")
                .replace(&單, &format!(r#"<{籤} class="單兒 段單兒"><li>"#));
            單 = re!("<br>", "g").replace(&單, "</li><li>");
            單 = re!(r"\t{2,}", "g").replace(&單, "</li><li>");
            單 = re!(r"<\/p>").replace(&單, &format!("</li></{籤}>"));
            txt = re!(r"<p>〚[列集]〛(.+?)<\/p>").replace(&txt, &單);
        }
    }
    txt = re!(r"<p>〚([譔撰作校]者|落款)〛(.+?)<\/p>", "g").replace(
        &txt,
        &format!(
            r#"<div class="$1" style="margin-bottom:{}em">$2</div>"#,
            num(env.行尾空白長)
        ),
    );
    re!(r"<p>\s*?<\/p>", "g").replace(&txt, "")
}

/// `渲染段`
pub fn 渲染段(txt: &str, env: &Env) -> String {
    let Some(ps) = re!(r">(\s*?)\(\{.+?\}\)", "g").match_all(txt) else {
        return txt.to_string();
    };
    let mut txt = txt.to_string();
    for p in ps {
        let mut p = re!(r"[\(\{\}\)]", "g").replace(&p, "");
        p = re!(r">(\s*?)").replace(&p, "");
        p = re!(r"字色:([^,]+?)($|;)").replace(&p, "color:var(--$1);");
        if let Some(clrs) = re!(r"字色:(.+?)($|;)").match_all(&p) {
            let list = re!(r"字色:|;", "g").replace(&clrs[0], "");
            let mut temp = String::new();
            for (j, c) in list.split(',').enumerate() {
                temp += &format!(
                    ",var(--{c}) calc(var(--行寬)*{j}) calc(var(--行寬)*{})",
                    j + 1
                );
            }
            p = re!(r"字色:(.+?)($|;)").replace(
                &p,
                &format!("color:transparent; background:repeating-linear-gradient(to left{temp}); -webkit-background-clip:text; background-clip:text;"),
            );
        }
        p = re!(r"⇥($|;)").replace(
            &p,
            &format!(
                "text-align:right; margin-bottom:{}em$1",
                num(env.行尾空白長)
            ),
        );
        p = re!(r"首縮進:(\d)($|;)").replace(&p, "margin-top:$1em;");
        p = re!(r"尾縮進:(\d)($|;)").replace(&p, "padding-bottom:$1em;");
        p = re!(r"(\s*?)").replace(&p, "");
        txt = re!(r">(\s*?)\(\{.+?\}\)").replace(&txt, &format!(r#" style="{p}">"#));
    }
    txt
}

/// `兼行`
pub fn 兼行(txt: &str) -> String {
    let Some(所共字) = re!(r"〚([歬前後]跨)\d+行］.+?［\1〛", "g").match_all(txt)
    else {
        return txt.to_string();
    };
    let mut txt = txt.to_string();
    for 各 in 所共字 {
        let n: f64 = re!(r"\d+").match_all(&各).unwrap()[0].parse().unwrap();
        let 迻 = num(n / 2.0);
        let side = if 各.starts_with("〚後跨") {
            "right"
        } else {
            "left"
        };
        txt = re!(r"〚[歬前後]跨\d+行］").replace(
            &txt,
            &format!(r#"<span class="spanRows" style="{side}:calc(var(--行寬)*{迻})">"#),
        );
        txt = re!(r"［[歬前後]跨〛").replace(&txt, "</span>");
    }
    txt
}

/// `旁字`
pub fn 旁字(txt: &str) -> String {
    let Some(sides) = re!(r"\{\{.+?\}\}", "g").match_all(txt) else {
        return txt.to_string();
    };
    let mut txt = txt.to_string();
    for s in sides {
        let mut s = re!(r"^\{\{").replace(&s, "");
        s = re!(r"\}\}$").replace(&s, "");
        if s.contains('⏷') {
            s = re!(r"^(.+?)⏷\^¯(.+?)¯\^⏶$", "g").replace(
                &s,
                r#"<ruby class="mscScr">$1<span class="rightRT">$2</span></ruby>"#,
            );
            s = re!(r"^(.+?)⏷\^_(.+?)_\^⏶$", "g").replace(
                &s,
                r#"<ruby class="mscScr">$1<span class="leftRT">$2</span></ruby>"#,
            );
        }
        txt = re!(r"\{\{.+?\}\}").replace(&txt, &s);
    }
    txt
}

/// `提取旁句`
pub fn 提取旁句(txt: &str) -> String {
    let Some(sides) = re!(r"\(\?¯.+?¯¿\)", "g").match_all(txt) else {
        return txt.to_string();
    };
    let mut txt = txt.to_string();
    for s in sides {
        let mut s =
            re!(r"^\(\?¯〚錯〛").replace(&s, r#"<small class="rubyStc rubyGap"><span><span>"#);
        s = re!(r"¯¿\)$").replace(&s, "</span></span></small>");
        s = re!(r"^\(\?¯").replace(&s, r#"<small class="rubyStc"><span><span>"#);
        s = re!(r"¯¿\)$").replace(&s, "</span></span></small>");
        txt = re!(r"\(\?¯.+?¯¿\)").replace(&txt, &s);
    }
    txt
}

/// `字樣`
pub fn 字樣(txt: &str, 圖夾: &str) -> String {
    let 格式 = [".svg", ".png"];
    let mut 夾 = 圖夾.replace('\\', "/");
    if !夾.ends_with('/') {
        夾.push('/');
    }
    let mut txt = txt.to_string();
    for 樣 in re!(r"〚字樣］(.+?)［字樣〛", "g").captures_all(&txt.clone()) {
        let v = 樣[1].clone().unwrap_or_default();
        let rep = if !v.contains('◎') {
            let 字 = v;
            let 徑0 = format!("{夾}{字}");
            format!(
                r#"<dfn class="字樣" data-char="{字}"><img src="{徑0}{}" onerror="this.onerror=null; this.src='{徑0}{}'" /><b>{字}</b></dfn>"#,
                格式[0], 格式[1]
            )
        } else {
            let mut it = v.split('◎');
            let 字 = it.next().unwrap_or("").to_string();
            let 徑 = format!("{夾}{}", it.next().unwrap_or(""));
            format!(r#"<dfn class="字樣" data-char="{字}"><img src="{徑}" /><b>{字}</b></dfn>"#)
        };
        txt = re!(r"〚字樣］.+?［字樣〛").replace(&txt, &rep);
    }
    txt
}
