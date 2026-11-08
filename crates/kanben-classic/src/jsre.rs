//! 按 JavaScript 的语义使用正则：把原版的正则源码原样翻成 fancy-regex，
//! 并实现 JS 的 `replace`（含 `$1`、`$&` 等替换模板）与 `match`。
//!
//! 差别要点（都按 JS 处理）：
//! - `.` 不匹配 `\n` `\r` `\u2028` `\u2029`；
//! - `\d` `\w` 只含 ASCII，`\s` 是 JS 的空白集合；
//! - 字符类里的 `[` 是普通字符；不成量词的 `{` `}` 是普通字符；
//! - 转义非 ASCII 字符（如 `\「`）就是该字符本身。

use std::borrow::Cow;

use fancy_regex::{Captures, Regex};

const JS_SPACE: &str =
    r"\t\n\x0B\x0C\r \xA0\x{1680}\x{2000}-\x{200A}\x{2028}\x{2029}\x{202F}\x{205F}\x{3000}\x{FEFF}";

/// 把 JS 正则源码翻成 fancy-regex 源码。`flags` 同 JS（g 由调用方处理）。
pub fn translate(src: &str, flags: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::new();
    if flags.contains('m') {
        out.push_str("(?m)");
    }
    if flags.contains('i') {
        out.push_str("(?i)");
    }
    let mut in_class = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' => {
                let Some(&n) = chars.get(i + 1) else {
                    out.push_str(r"\\");
                    break;
                };
                i += 2;
                match n {
                    'd' => out.push_str(if in_class { "0-9" } else { "[0-9]" }),
                    'D' => out.push_str("[^0-9]"),
                    'w' => out.push_str(if in_class {
                        "A-Za-z0-9_"
                    } else {
                        "[A-Za-z0-9_]"
                    }),
                    'W' => out.push_str("[^A-Za-z0-9_]"),
                    's' => {
                        if in_class {
                            out.push_str(JS_SPACE)
                        } else {
                            out.push('[');
                            out.push_str(JS_SPACE);
                            out.push(']');
                        }
                    }
                    'S' => {
                        out.push_str("[^");
                        out.push_str(JS_SPACE);
                        out.push(']');
                    }
                    'b' if !in_class => out.push_str(r"(?-u:\b)"),
                    'B' if !in_class => out.push_str(r"(?-u:\B)"),
                    'u' => {
                        if chars.get(i) == Some(&'{') {
                            let end = chars[i..].iter().position(|&c| c == '}').map(|p| p + i);
                            let end = end.expect("unterminated \\u{");
                            let hex: String = chars[i + 1..end].iter().collect();
                            out.push_str(&format!(r"\x{{{hex}}}"));
                            i = end + 1;
                        } else {
                            let hex: String = chars[i..i + 4].iter().collect();
                            out.push_str(&format!(r"\x{{{hex}}}"));
                            i += 4;
                        }
                    }
                    'x' => {
                        let hex: String = chars[i..i + 2].iter().collect();
                        out.push_str(&format!(r"\x{{{hex}}}"));
                        i += 2;
                    }
                    't' | 'n' | 'r' | 'v' | 'f' => {
                        out.push('\\');
                        out.push(n);
                    }
                    '0'..='9' if !in_class => {
                        out.push('\\');
                        out.push(n);
                    }
                    '/' => out.push('/'),
                    n if n.is_ascii() => {
                        out.push('\\');
                        out.push(n);
                    }
                    // 非 ASCII 字符的转义就是字符本身。
                    n => push_literal(&mut out, n, in_class),
                }
                continue;
            }
            '[' if !in_class => {
                in_class = true;
                out.push('[');
                if chars.get(i + 1) == Some(&'^') {
                    out.push('^');
                    i += 1;
                }
                // JS 允许 `[]` 开头的 `]` 表示空类；原版没有这种写法。
            }
            '[' => out.push_str(r"\["),
            ']' if in_class => {
                in_class = false;
                out.push(']');
            }
            '&' | '~' if in_class => {
                out.push('\\');
                out.push(c);
            }
            '.' if !in_class => out.push_str(r"[^\n\r\x{2028}\x{2029}]"),
            '{' if !in_class => {
                // 成量词就照抄，否则是普通字符。
                let rest: String = chars[i + 1..].iter().take(12).collect();
                let quant = rest.find('}').map(|e| &rest[..e]).is_some_and(|q| {
                    let mut parts = q.splitn(2, ',');
                    let a = parts.next().unwrap_or("");
                    !a.is_empty()
                        && a.bytes().all(|b| b.is_ascii_digit())
                        && parts
                            .next()
                            .is_none_or(|b| b.bytes().all(|b| b.is_ascii_digit()))
                });
                if quant {
                    let end = i + 1 + rest.find('}').unwrap();
                    let q: String = chars[i..=end].iter().collect();
                    out.push_str(&q);
                    i = end;
                } else {
                    out.push_str(r"\{");
                }
            }
            '}' if !in_class => out.push_str(r"\}"),
            c => out.push(c),
        }
        i += 1;
    }
    out
}

fn push_literal(out: &mut String, c: char, _in_class: bool) {
    out.push(c);
}

/// 一条 JS 正则。
pub struct Re {
    re: Regex,
    global: bool,
}

impl Re {
    pub fn new(src: &str, flags: &str) -> Re {
        let t = translate(src, flags);
        let re = Regex::new(&t).unwrap_or_else(|e| panic!("bad regex /{src}/ → {t}: {e}"));
        Re {
            re,
            global: flags.contains('g'),
        }
    }

    /// `s.replace(re, 模板)`：模板按 JS 规则展开 `$$ $& $\` $' $1…$99`。
    pub fn replace(&self, s: &str, template: &str) -> String {
        self.replace_with(s, |caps, s| expand(template, caps, s))
    }

    /// `s.replace(re, 函数)`。
    pub fn replace_with(
        &self,
        s: &str,
        mut f: impl FnMut(&Captures<'_, str>, &str) -> String,
    ) -> String {
        let mut out = String::with_capacity(s.len());
        let mut last = 0;
        let mut at = 0;
        while at <= s.len() {
            let Some(caps) = self.re.captures_from_pos(s, at).expect("regex failed") else {
                break;
            };
            let m = caps.get(0).unwrap();
            out.push_str(&s[last..m.start()]);
            out.push_str(&f(&caps, s));
            last = m.end();
            if !self.global {
                break;
            }
            at = if m.end() == m.start() {
                // 空匹配：前进一个字符（JS 前进一个 UTF-16 单元，对 BMP 字符相同）。
                match s[m.end()..].chars().next() {
                    Some(c) => {
                        if m.end() + c.len_utf8() > s.len() {
                            break;
                        }
                        out.push_str(&s[m.end()..m.end() + c.len_utf8()]);
                        last = m.end() + c.len_utf8();
                        m.end() + c.len_utf8()
                    }
                    None => break,
                }
            } else {
                m.end()
            };
        }
        out.push_str(&s[last..]);
        out
    }

    /// `s.match(re)`：全局时返回所有匹配，否则只返回第一个；没有则 `None`（JS 的 null）。
    pub fn match_all(&self, s: &str) -> Option<Vec<String>> {
        let mut v = Vec::new();
        let mut at = 0;
        while at <= s.len() {
            let Some(m) = self.re.find_from_pos(s, at).expect("regex failed") else {
                break;
            };
            v.push(m.as_str().to_string());
            if !self.global {
                break;
            }
            at = if m.end() == m.start() {
                match s[m.end()..].chars().next() {
                    Some(c) => m.end() + c.len_utf8(),
                    None => break,
                }
            } else {
                m.end()
            };
        }
        (!v.is_empty()).then_some(v)
    }

    /// 所有匹配的分组（`matchAll`）。
    pub fn captures_all(&self, s: &str) -> Vec<Vec<Option<String>>> {
        let mut v = Vec::new();
        let mut at = 0;
        while at <= s.len() {
            let Some(c) = self.re.captures_from_pos(s, at).expect("regex failed") else {
                break;
            };
            let m = c.get(0).unwrap();
            v.push(
                (0..c.len())
                    .map(|i| c.get(i).map(|g| g.as_str().to_string()))
                    .collect(),
            );
            at = if m.end() == m.start() {
                match s[m.end()..].chars().next() {
                    Some(ch) => m.end() + ch.len_utf8(),
                    None => break,
                }
            } else {
                m.end()
            };
        }
        v
    }

    pub fn test(&self, s: &str) -> bool {
        self.re.is_match(s).expect("regex failed")
    }

    /// 第一个匹配的起点（字节）。
    pub fn search(&self, s: &str) -> Option<usize> {
        self.re.find(s).expect("regex failed").map(|m| m.start())
    }
}

/// 展开 JS 替换模板。
pub fn expand(template: &str, caps: &Captures<'_, str>, s: &str) -> String {
    if !template.contains('$') {
        return template.to_string();
    }
    let m = caps.get(0).unwrap();
    let n = caps.len() - 1;
    let t: Vec<char> = template.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < t.len() {
        if t[i] != '$' || i + 1 >= t.len() {
            out.push(t[i]);
            i += 1;
            continue;
        }
        match t[i + 1] {
            '$' => {
                out.push('$');
                i += 2;
            }
            '&' => {
                out.push_str(m.as_str());
                i += 2;
            }
            '`' => {
                out.push_str(&s[..m.start()]);
                i += 2;
            }
            '\'' => {
                out.push_str(&s[m.end()..]);
                i += 2;
            }
            d @ '0'..='9' => {
                let one = d.to_digit(10).unwrap() as usize;
                let two = t
                    .get(i + 2)
                    .and_then(|c| c.to_digit(10))
                    .map(|e| one * 10 + e as usize);
                let (k, used) = match two {
                    Some(k) if k >= 1 && k <= n => (k, 3),
                    _ if one >= 1 && one <= n => (one, 2),
                    _ => {
                        out.push('$');
                        i += 1;
                        continue;
                    }
                };
                if let Some(g) = caps.get(k) {
                    out.push_str(g.as_str());
                }
                i += used;
            }
            _ => {
                out.push('$');
                i += 1;
            }
        }
    }
    out
}

/// JS 的 `str.replace(字符串, 模板)`：只换第一处，按字面找；模板仍展开 `$&` 等。
pub fn replace_str(s: &str, pat: &str, template: &str) -> String {
    match s.find(pat) {
        None => s.to_string(),
        Some(i) => {
            let mut out = String::with_capacity(s.len());
            out.push_str(&s[..i]);
            out.push_str(&expand_literal(template, pat, &s[..i], &s[i + pat.len()..]));
            out.push_str(&s[i + pat.len()..]);
            out
        }
    }
}

fn expand_literal(template: &str, m: &str, before: &str, after: &str) -> String {
    if !template.contains('$') {
        return template.to_string();
    }
    let mut out = String::new();
    let mut it = template.chars().peekable();
    while let Some(c) = it.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        match it.peek() {
            Some('$') => {
                out.push('$');
                it.next();
            }
            Some('&') => {
                out.push_str(m);
                it.next();
            }
            Some('`') => {
                out.push_str(before);
                it.next();
            }
            Some('\'') => {
                out.push_str(after);
                it.next();
            }
            _ => out.push('$'),
        }
    }
    out
}

/// JS 字符串的长度：UTF-16 单元数。
pub fn u16len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// 编译一次、到处复用的正则。
#[macro_export]
macro_rules! re {
    ($src:expr) => {
        $crate::re!($src, "")
    };
    ($src:expr, $flags:expr) => {{
        static R: std::sync::OnceLock<$crate::jsre::Re> = std::sync::OnceLock::new();
        R.get_or_init(|| $crate::jsre::Re::new($src, $flags))
    }};
}

pub fn cow(s: &str) -> Cow<'_, str> {
    Cow::Borrowed(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_and_classes() {
        let r = Re::new(r"\(\(=(.+?)=\)\)", "g");
        assert_eq!(
            r.replace("甲((=乙=))丙((=丁=))", "<s>$1</s>"),
            "甲<s>乙</s>丙<s>丁</s>"
        );
        assert!(!Re::new("a.b", "").test("a\rb"));
        assert!(Re::new(r"[\[\?¿\]]", "g").test("["));
        assert_eq!(
            Re::new(r"[\[\?¿\]]", "g").replace("[[?⿰木鬼¿]]", ""),
            "⿰木鬼"
        );
    }

    #[test]
    fn templates() {
        let r = Re::new("(a)(b)?", "g");
        assert_eq!(r.replace("ac", "[$1|$2|$&|$$|$3]"), "[a||a|$|$3]c");
        assert_eq!(Re::new("b", "").replace("abc", "<$`|$'>"), "a<a|c>c");
    }

    #[test]
    fn braces_and_backrefs() {
        assert_eq!(
            Re::new(r"〚列］(.+?)［列〛}", "g").replace("〚列］x［列〛}", "<$1>"),
            "<x>"
        );
        assert_eq!(Re::new(r"\n{2,}", "").replace("a\n\n\nb", "|"), "a|b");
        let r = Re::new(r"〚([歬前後]跨)\d+行］.+?［\1〛", "g");
        assert_eq!(r.match_all("〚後跨1行］字［後跨〛").unwrap().len(), 1);
    }

    #[test]
    fn utf16() {
        assert_eq!(u16len("𠀀甲"), 3);
    }
}
