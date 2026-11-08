//! 原版 ts.js 編輯器的語法著色：純文本 → 帶顏色標記的 HTML（每行一個 div）。

use crate::jsre::Re;
use crate::re;

/// 原版 `複符號對` 與 `單符號對` 的各個正則源碼，次序照原版（非 ASCII 字一律轉義）。
const 符號: &[&str] = &[
    r"\(\(=",
    r"=\)\)",
    r"\(\(-",
    r"-\)\)",
    r"\(\(#",
    r"#\)\)",
    r"\u{3014}\u{3014}-",
    r"-\u{3015}\u{3015}",
    r"\u{201C}\u{201C}=",
    r"=\u{201D}\u{201D}",
    r"\(\u{3010}",
    r"\u{3011}\)",
    r"\[\u{3010}",
    r"\u{3011}\]",
    r"\u{3014}\u{3010}",
    r"\u{3011}\u{3015}",
    r"\(\u{3016}",
    r"\u{3017}\)",
    r"\[\u{3016}",
    r"\u{3017}\]",
    r"\u{3014}\u{3016}",
    r"\u{3017}\u{3015}",
    r"\u{FE5D}\u{3016}",
    r"\u{3017}\u{FE5E}",
    r"\u{FE59}\u{3016}",
    r"\u{3017}\u{FE5A}",
    r"\{\u{FF3B}",
    r"\u{FF3D}\}",
    r"\{\u{FF08}",
    r"\u{FF09}\}",
    r"\[\[\?",
    r"\u{BF}\]\]",
    r"\u{169C}\[\u{AF}",
    r"\u{AF}\]\u{169B}",
    r"\u{169C}\[_",
    r"_\]\u{169B}",
    r"\u{169C}\(\u{AF}",
    r"\u{AF}\)\u{169B}",
    r"\u{169C}\(_",
    r"_\)\u{169B}",
    r"\(\?\u{AF}",
    r"\u{AF}\u{BF}\)",
    r"\(\?\u{2329}",
    r"\u{232A}\u{BF}\)",
    r"\?\?[\-\u{2022}\u{25E6}]",
    r"[\-\u{2022}\u{25E6}]\u{BF}\u{BF}",
    r"\!\!\u{AF}[\u{FE46}\u{FE45}\u{2022}\u{25E6}]",
    r"[\u{FE46}\u{FE45}\u{2022}\u{25E6}]\u{AF}\u{A1}\u{A1}",
    r"\!\[[\u{25CF}\u{2B23}\u{25CB}\u{2394}]",
    r"[\u{25CF}\u{2B23}\u{25CB}\u{2394}]\]\u{A1}",
    r"\u{2039}",
    r"\u{203A}",
    r"\u{300A}",
    r"\u{300B}",
    r"\u{3008}",
    r"\u{3009}",
];

/// 所有標記符合成的一條正則：`(甲|乙|…)`。
fn 所有符() -> &'static Re {
    static R: std::sync::OnceLock<Re> = std::sync::OnceLock::new();
    R.get_or_init(|| Re::new(&format!("({})", 符號.join("|")), "g"))
}

/// `編輯純文本.渲染` 的文本部分：返回寫入富文本框的 HTML（標題的後續段落由頁面另行標出）。
pub fn 渲染(文本: &str) -> String {
    let mut t =
        re!(r"\(\(=(.+?)(?==\)\))", "g").replace(文本, r#"((=<span class="edit_dlnote">$1</span>"#);
    t = re!(r"\(\(-(.+?)(?=-\)\))", "g").replace(&t, r#"((-<span class="edit_1lnote">$1</span>"#);
    t = re!(r"\(\(#(.+?)(?=#\)\))", "g").replace(&t, r#"((#<span class="edit_pCmnt">$1</span>"#);
    t = re!(r"〚([^〚［］〛]+?)〛", "g").replace(&t, r#"<span class="edit_toke">〚$1〛</span>"#);
    t = re!(r"〚([^〚［］〛]+?)］", "g").replace(&t, r#"<span class="edit_toke">〚$1］</span>"#);
    t = re!(r"［([^〚［］〛]+?)〛", "g").replace(&t, r#"<span class="edit_toke">［$1〛</span>"#);
    t = 所有符().replace(&t, r#"<span class="edit_marco">$1</span>"#);
    t = format!("<div>{}</div>", t.replace('\n', "</div><div>"));
    t = re!(r"(<div><\/div>)(<div>)(\*　.+)", "g").replace(&t, r#"$1<div class="edit_title">$3"#);
    re!(r"^(<div>)(\*　.+?<\/div>)", "g").replace(&t, r#"<div class="edit_title">$2"#)
}

/// `在何段`：游標（UTF-16 下標）之前有幾個換行。
pub fn 在何段(文本: &str, 字符敘: usize) -> usize {
    let mut n = 0;
    let mut u = 0;
    for c in 文本.chars() {
        if u >= 字符敘 {
            break;
        }
        if c == '\n' {
            n += 1;
        }
        u += c.len_utf16();
    }
    n
}
