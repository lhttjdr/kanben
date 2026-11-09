//! 行内语法（LaTeX 风格）。
//!
//! ```text
//! \名[參數]{內容}     命令。名是 ASCII 字母；參數可省；內容可以嵌套命令
//! \\  \\[2]          另起一行（擡二格）
//! \{ \} \[ \] \%     转义
//! 《…》 〈…〉          正文里的書名號：保留原样，画法由版式决定
//! ```
//! 标记只用 ASCII 符号，正文只用书里本来就有的字，两者不会混淆。
//! 元素不能跨行；没闭合的会在行尾自动闭合并报错。

use super::ast::{Element, ElementKind, Inline};
use super::commands::{self, Arg, Kind};
use super::span::{Diagnostic, Span};

/// 可以用反斜线转义的字符。
const ESCAPABLE: &str = "{}[]%《》〈〉";

pub fn parse_inline(line: &str, base: usize, diags: &mut Vec<Diagnostic>) -> Vec<Inline> {
    let mut p = Parser::new(line, base, diags);
    p.seq(None).0
}

/// 行首块级命令拆开后的各部分。
pub struct CommandParts {
    pub name: String,
    pub span: Span,
    pub args: Vec<Arg>,
    pub body: Option<Vec<Inline>>,
    /// 内容后面的参数：`\begin{indent}[2]`。
    pub post_args: Vec<Arg>,
    /// 命令结束处在 `line` 里的字节偏移。
    pub end: usize,
}

/// 若 `line` 以 `\字母` 开头，拆出命令名、参数、内容。
pub fn parse_command_at(
    line: &str,
    base: usize,
    diags: &mut Vec<Diagnostic>,
) -> Option<CommandParts> {
    let mut p = Parser::new(line, base, diags);
    let (_, c) = p.chars.first().copied()?;
    let (_, n) = p.chars.get(1).copied()?;
    if c != '\\' || !n.is_ascii_alphabetic() {
        return None;
    }
    p.i = 1;
    let name = p.name();
    let args = p.args();
    let body = if p.peek() == Some('{') {
        Some(p.brace(0))
    } else {
        None
    };
    let post_args = if body.is_some() { p.args() } else { Vec::new() };
    let span = p.span(0, p.i);
    let end = p.chars.get(p.i).map_or(line.len(), |&(o, _)| o);
    Some(CommandParts {
        name,
        span,
        args,
        body,
        post_args,
        end,
    })
}

struct Parser<'d> {
    chars: Vec<(usize, char)>,
    line_len: usize,
    i: usize,
    base: usize,
    /// 正在等待的闭合符号，内层在后。
    closers: Vec<char>,
    diags: &'d mut Vec<Diagnostic>,
}

struct TextBuf {
    text: String,
    start: usize,
}

impl<'d> Parser<'d> {
    fn new(line: &str, base: usize, diags: &'d mut Vec<Diagnostic>) -> Self {
        Parser {
            chars: line.char_indices().collect(),
            line_len: line.len(),
            i: 0,
            base,
            closers: Vec::new(),
            diags,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).map(|&(_, c)| c)
    }

    fn off(&self, i: usize) -> usize {
        self.base + self.chars.get(i).map_or(self.line_len, |&(o, _)| o)
    }

    fn span(&self, from: usize, to: usize) -> Span {
        Span::new(self.off(from), self.off(to))
    }

    fn push(&self, buf: &mut TextBuf, c: char) {
        if buf.text.is_empty() {
            buf.start = self.i;
        }
        buf.text.push(c);
    }

    fn flush(&self, buf: &mut TextBuf, out: &mut Vec<Inline>) {
        if !buf.text.is_empty() {
            let span = self.span(buf.start, self.i);
            out.push(Inline::Text {
                text: std::mem::take(&mut buf.text),
                span,
            });
        }
    }

    /// 解析到当前闭合符号为止（消耗它）。遇到外层的闭合符号就停下（不消耗），
    /// 这样漏写一个闭合符号只影响自己这一层。
    fn seq(&mut self, close: Option<char>) -> (Vec<Inline>, bool) {
        let mut out = Vec::new();
        let mut buf = TextBuf {
            text: String::new(),
            start: self.i,
        };
        while let Some(c) = self.peek() {
            if Some(c) == close {
                self.flush(&mut buf, &mut out);
                self.i += 1;
                return (out, true);
            }
            if self.closers.contains(&c) {
                break;
            }
            match c {
                '\\' => {
                    self.flush(&mut buf, &mut out);
                    self.command(&mut out);
                }
                '《' | '〈' => {
                    self.flush(&mut buf, &mut out);
                    let el = self.title(c);
                    out.push(Inline::Element(el));
                }
                '{' => {
                    let span = self.span(self.i, self.i + 1);
                    self.diags.push(Diagnostic::error(
                        span,
                        "多餘的「{」（字面的大括號寫作 \\{）",
                    ));
                    self.i += 1;
                }
                '}' => {
                    let span = self.span(self.i, self.i + 1);
                    self.diags.push(Diagnostic::error(
                        span,
                        "多餘的「}」（字面的大括號寫作 \\}）",
                    ));
                    self.i += 1;
                }
                _ => {
                    self.push(&mut buf, c);
                    self.i += 1;
                }
            }
        }
        self.flush(&mut buf, &mut out);
        (out, false)
    }

    fn name(&mut self) -> String {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '-' {
                name.push(c);
                self.i += 1;
            } else {
                break;
            }
        }
        name
    }

    /// `[a, b=c]`，没有方括号时返回空。
    fn args(&mut self) -> Vec<Arg> {
        if self.peek() != Some('[') {
            return Vec::new();
        }
        let open = self.i;
        let Some(close) = (open + 1..self.chars.len()).find(|&j| self.chars[j].1 == ']') else {
            self.diags.push(Diagnostic::error(
                self.span(open, open + 1),
                "「[」沒有對應的「]」",
            ));
            self.i = self.chars.len();
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut start = open + 1;
        for j in open + 1..=close {
            if j == close || self.chars[j].1 == ',' {
                let piece: String = self.chars[start..j].iter().map(|&(_, c)| c).collect();
                let trimmed = piece.trim();
                if !trimmed.is_empty() {
                    let span = self.span(start, j);
                    let (key, value) = match trimmed.split_once('=') {
                        Some((k, v)) => (Some(k.trim().to_string()), v.trim().to_string()),
                        None => (None, trimmed.to_string()),
                    };
                    out.push(Arg { key, value, span });
                }
                start = j + 1;
            }
        }
        self.i = close + 1;
        out
    }

    /// `{…}`。调用时 `self.i` 指向 `{`。
    fn brace(&mut self, cmd_start: usize) -> Vec<Inline> {
        let open = self.i;
        self.i += 1;
        self.closers.push('}');
        let (children, closed) = self.seq(Some('}'));
        self.closers.pop();
        if !closed {
            let span = self.span(cmd_start, open + 1);
            self.diags
                .push(Diagnostic::error(span, "「{」沒有對應的「}」"));
        }
        children
    }

    fn title(&mut self, open: char) -> Element {
        let start = self.i;
        let close = if open == '《' { '》' } else { '〉' };
        self.i += 1;
        self.closers.push(close);
        let (children, closed) = self.seq(Some(close));
        self.closers.pop();
        if !closed {
            let span = self.span(start, start + 1);
            self.diags.push(Diagnostic::warning(
                span,
                format!("「{open}」沒有對應的「{close}」，書名號只標到這裏"),
            ));
        }
        Element {
            kind: ElementKind::Title {
                brackets: Some((open, close)),
            },
            children,
            span: self.span(start, self.i),
        }
    }

    /// `\…`。调用时 `self.i` 指向反斜线。
    fn command(&mut self, out: &mut Vec<Inline>) {
        let start = self.i;
        self.i += 1;
        match self.peek() {
            None => {
                let span = self.span(start, self.i);
                self.diags
                    .push(Diagnostic::warning(span, "行尾多了一個「\\」"));
                return;
            }
            Some('\\') => {
                self.i += 1;
                let args = self.args();
                let raise = match commands::number_arg(&args, "raise") {
                    Ok(n) => n.unwrap_or(0),
                    Err((span, msg)) => {
                        self.diags.push(Diagnostic::error(span, msg));
                        0
                    }
                };
                let span = self.span(start, self.i);
                out.push(Inline::Element(Element {
                    kind: ElementKind::Break { raise },
                    children: Vec::new(),
                    span,
                }));
                return;
            }
            Some(c) if ESCAPABLE.contains(c) => {
                self.i += 1;
                out.push(Inline::Text {
                    text: c.to_string(),
                    span: self.span(start, self.i),
                });
                return;
            }
            Some(c) if c.is_ascii_alphabetic() => {}
            Some(_) => {
                let span = self.span(start, self.i + 1);
                self.diags.push(Diagnostic::error(
                    span,
                    "「\\」後面應是命令名，或要轉義的 { } [ ] % 《 》 〈 〉",
                ));
                return;
            }
        }

        let name = self.name();
        let name_span = self.span(start, self.i);
        let args = self.args();
        match commands::kind_of(&name) {
            None | Some(Kind::Block) => {
                let msg = if commands::kind_of(&name).is_some() {
                    format!("\\{name} 只能寫在行首")
                } else {
                    format!("未知的命令 \\{name}（可用：{}）", commands::INLINE_COMMANDS)
                };
                self.diags.push(Diagnostic::error(name_span, msg));
                // 内容仍按普通文字保留，免得丢字。
                if self.peek() == Some('{') {
                    let body = self.brace(start);
                    out.extend(body);
                }
            }
            Some(Kind::Inline) => {
                if self.peek() != Some('{') {
                    self.diags.push(Diagnostic::error(
                        name_span,
                        format!("\\{name} 後面缺少 {{內容}}"),
                    ));
                    return;
                }
                let children = self.brace(start);
                let span = self.span(start, self.i);
                let resolved = commands::resolve_inline(&name, &args);
                for (s, w) in resolved.warnings {
                    let s = if s == Span::default() { name_span } else { s };
                    self.diags.push(Diagnostic::warning(s, w));
                }
                let el = match resolved.kind {
                    ElementKind::Compose(_) => self.compose(children, span),
                    ElementKind::Rod(_) => self.rod(children, span),
                    k @ (ElementKind::Suzhou(_)
                    | ElementKind::Divination(_)
                    | ElementKind::Digits(_)
                    | ElementKind::Glyph(_)
                    | ElementKind::Image(_)
                    | ElementKind::Map(_)) => self.text_body(&name, k, children, span),
                    kind => Element {
                        kind,
                        children,
                        span,
                    },
                };
                out.push(Inline::Element(el));
            }
        }
    }

    /// `\\rod{1971}`、`\\rod{4720, -398}`：幾個整數，可帶負號，存成「4720,-398」。
    fn rod(&mut self, children: Vec<Inline>, span: Span) -> Element {
        let mut text = String::new();
        for child in &children {
            collect_text(child, &mut text);
        }
        let mut nums = Vec::new();
        let mut bad = false;
        for part in text.split([',', '，', '、']) {
            let part: String = part.chars().filter(|c| !c.is_whitespace()).collect();
            if part.is_empty() {
                continue;
            }
            let (neg, rest) = match part.strip_prefix(['-', '−']) {
                Some(r) => ("-", r),
                None => ("", part.as_str()),
            };
            let d: String = rest.chars().filter(char::is_ascii_digit).collect();
            bad |= d.chars().count() != rest.chars().count();
            if !d.is_empty() {
                nums.push(format!("{neg}{d}"));
            }
        }
        let digits = nums.join(",");
        if digits.is_empty() {
            self.diags.push(Diagnostic::error(
                span,
                "\\rod 的內容應是阿拉伯數字，如 \\rod{1971}",
            ));
        } else if bad {
            self.diags.push(Diagnostic::warning(
                span,
                "\\rod 裏只能寫阿拉伯數字（可帶負號，用逗號分開幾個數），其餘的字已忽略",
            ));
        }
        Element {
            kind: ElementKind::Rod(digits),
            children: Vec::new(),
            span,
        }
    }

    /// 內容只取文字的命令：數碼、字樣、圖片、地圖。
    fn text_body(
        &mut self,
        name: &str,
        kind: ElementKind,
        children: Vec<Inline>,
        span: Span,
    ) -> Element {
        let mut text = String::new();
        for child in &children {
            if let Inline::Element(_) = child {
                self.diags.push(Diagnostic::warning(
                    span,
                    format!("\\{name} 裏只能寫文字，其中的標記已忽略"),
                ));
            }
            collect_text(child, &mut text);
        }
        let text = text.trim().to_string();
        let ok = match kind {
            ElementKind::Suzhou(_) | ElementKind::Divination(_) => {
                text.chars().all(|c| c.is_ascii_digit())
            }
            ElementKind::Digits(_) => text
                .chars()
                .all(|c| c.is_ascii_digit() || ".:-−+".contains(c)),
            _ => true,
        };
        if text.is_empty() || !ok {
            self.diags.push(Diagnostic::error(
                span,
                format!(
                    "\\{name} 的內容不對：應是{}",
                    match kind {
                        ElementKind::Suzhou(_) | ElementKind::Divination(_) => "阿拉伯數字",
                        ElementKind::Digits(_) => "阿拉伯數字（可帶小數點、冒號、負號）",
                        _ => "文字",
                    }
                ),
            ));
        }
        let kind = match kind {
            ElementKind::Suzhou(_) => ElementKind::Suzhou(text),
            ElementKind::Divination(_) => ElementKind::Divination(text),
            ElementKind::Digits(_) => ElementKind::Digits(text),
            ElementKind::Glyph(_) => ElementKind::Glyph(text),
            ElementKind::Image(_) => ElementKind::Image(text),
            ElementKind::Map(_) => ElementKind::Map(text),
            k => k,
        };
        Element {
            kind,
            children: Vec::new(),
            span,
        }
    }

    fn compose(&mut self, children: Vec<Inline>, span: Span) -> Element {
        let mut ids = String::new();
        for child in &children {
            if let Inline::Element(_) = child {
                self.diags.push(Diagnostic::warning(
                    span,
                    "\\compose 裏只能寫 IDS 文字，其中的標記已忽略",
                ));
            }
            collect_text(child, &mut ids);
        }
        let ids: String = ids.chars().filter(|c| !c.is_whitespace()).collect();
        if ids.is_empty() {
            self.diags
                .push(Diagnostic::error(span, "\\compose 的內容是空的"));
        }
        Element {
            kind: ElementKind::Compose(ids),
            children: Vec::new(),
            span,
        }
    }
}

pub fn collect_text(node: &Inline, out: &mut String) {
    match node {
        Inline::Text { text, .. } => out.push_str(text),
        Inline::Element(el) => el.children.iter().for_each(|c| collect_text(c, out)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::ast::{Encl, Shape};

    fn parse(s: &str) -> (Vec<Inline>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let r = parse_inline(s, 0, &mut d);
        (r, d)
    }

    fn kinds(nodes: &[Inline]) -> Vec<String> {
        nodes
            .iter()
            .map(|n| match n {
                Inline::Text { text, .. } => format!("T({text})"),
                Inline::Element(e) => format!("{:?}{}", e.kind, kinds(&e.children).join("")),
            })
            .collect()
    }

    #[test]
    fn command_with_body() {
        let (r, d) = parse(r"曰\note{繫，戶計翻。}言");
        assert!(d.is_empty(), "{d:?}");
        assert_eq!(kinds(&r), ["T(曰)", "DoubleNoteT(繫，戶計翻。)", "T(言)"]);
    }

    #[test]
    fn nested_and_titles() {
        let (r, d) = parse(r"\note{《史記》曰\name{微子}}");
        assert!(d.is_empty(), "{d:?}");
        assert_eq!(
            kinds(&r),
            [
                "DoubleNoteTitle { brackets: Some(('《', '》')) }T(史記)T(曰)Name { double: false }T(微子)"
            ]
        );
    }

    #[test]
    fn args() {
        let (r, d) = parse(r"\inv[round]{注}");
        assert!(d.is_empty());
        assert_eq!(
            kinds(&r),
            [format!(
                "{:?}T(注)",
                ElementKind::Inverse(Encl {
                    shape: Shape::Round,
                    narrow: false
                })
            )]
        );
    }

    #[test]
    fn line_break() {
        let (r, d) = parse(r"臣\\[2]皇帝\\末");
        assert!(d.is_empty(), "{d:?}");
        assert_eq!(
            kinds(&r),
            [
                "T(臣)",
                "Break { raise: 2 }",
                "T(皇帝)",
                "Break { raise: 0 }",
                "T(末)"
            ]
        );
    }

    #[test]
    fn content_brackets_are_plain_text() {
        let (r, d) = parse("【注】云：「是」");
        assert!(d.is_empty());
        assert_eq!(kinds(&r), ["T(【注】云：「是」)"]);
    }

    #[test]
    fn escapes() {
        let (r, d) = parse(r"a\{b\}\%\《");
        assert!(d.is_empty());
        let text: String = r
            .iter()
            .map(|n| match n {
                Inline::Text { text, .. } => text.clone(),
                _ => panic!(),
            })
            .collect();
        assert_eq!(text, "a{b}%《");
    }

    #[test]
    fn unclosed_brace_stops_at_outer_level() {
        // 《 漏了 》：书名只标到 } 前，夾注仍正常闭合。
        let (r, d) = parse(r"甲\note{乙《丙}丁");
        assert_eq!(d.len(), 1);
        assert_eq!(
            kinds(&r),
            [
                "T(甲)",
                "DoubleNoteT(乙)Title { brackets: Some(('《', '》')) }T(丙)",
                "T(丁)"
            ]
        );
    }

    #[test]
    fn unclosed_command_reports_error() {
        let src = r"甲\note{乙";
        let (r, d) = parse(src);
        assert_eq!(d.len(), 1);
        assert_eq!(&src[d[0].span.start..d[0].span.end], r"\note{");
        assert_eq!(kinds(&r), ["T(甲)", "DoubleNoteT(乙)"]);
    }

    #[test]
    fn unknown_command_keeps_text() {
        let (r, d) = parse(r"\foo{甲}乙");
        assert_eq!(d.len(), 1);
        assert_eq!(kinds(&r), ["T(甲)", "T(乙)"]);
    }

    #[test]
    fn block_command_inline_is_error() {
        let (_, d) = parse(r"甲\indent[2]乙");
        assert_eq!(d.len(), 1);
        assert!(d[0].message.contains("行首"));
    }

    #[test]
    fn compose() {
        let (r, d) = parse(r"\compose{⿰木鬼}");
        assert!(d.is_empty());
        assert_eq!(kinds(&r), ["Compose(\"⿰木鬼\")"]);
    }

    #[test]
    fn stray_braces() {
        let (r, d) = parse("甲}乙{");
        assert_eq!(d.len(), 2);
        assert_eq!(kinds(&r), ["T(甲乙)"]);
    }

    #[test]
    fn missing_body() {
        let (_, d) = parse(r"\name 甲");
        assert_eq!(d.len(), 1);
    }
}
