//! 语法树 → 排版条目。
//!
//! 一行文字展平成若干条目：正文、雙行夾注、單行小注、旁行、眉批、擡頭換行。
//! 标记（專名、圈點、陰文等）不拆条目，而是写进每个字的 [`Style`]。

use super::options::{PunctStyle, TitleMark};
use super::token::{Mark, MarkKind, Style, Tok, TokKind, Tone, classify, is_combining};
use crate::syntax::ast::{Color, Element, ElementKind, Inline};
use crate::syntax::span::{Diagnostic, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Main(Vec<Tok>),
    Double(Vec<Tok>),
    Single(Vec<Tok>),
    /// 側行小注：只排右（左）半邊。
    Half {
        toks: Vec<Tok>,
        left: bool,
    },
    /// 兼行：排在當前行與後（前）`n` 行的正中。
    Span {
        toks: Vec<Tok>,
        prev: bool,
        n: u8,
        /// 源碼位置，用于提示。
        at: Span,
    },
    Side {
        toks: Vec<Tok>,
        left: bool,
    },
    Head(Vec<Tok>),
    Break {
        raise: u8,
    },
}

pub struct Flattener<'d> {
    punct: PunctStyle,
    title_mark: TitleMark,
    /// 下一个标记编号。整篇文档里要唯一，所以由调用方传入并取回。
    pub next_id: u32,
    diags: &'d mut Vec<Diagnostic>,
}

/// 当前所在的上下文。
#[derive(Clone)]
struct Ctx {
    style: Style,
    /// 是否在夾注、旁行、眉批里（其中不能再开新的條目）。
    nested: Option<&'static str>,
}

impl<'d> Flattener<'d> {
    pub fn new(
        punct: PunctStyle,
        title_mark: TitleMark,
        next_id: u32,
        diags: &'d mut Vec<Diagnostic>,
    ) -> Self {
        Flattener {
            punct,
            title_mark,
            next_id,
            diags,
        }
    }

    pub fn line(&mut self, nodes: &[Inline]) -> Vec<Item> {
        let mut out = Vec::new();
        let mut cur = Vec::new();
        let ctx = Ctx {
            style: Style::default(),
            nested: None,
        };
        self.walk(nodes, &ctx, &mut out, &mut cur);
        if !cur.is_empty() {
            out.push(Item::Main(cur));
        }
        out
    }

    fn walk(&mut self, nodes: &[Inline], ctx: &Ctx, out: &mut Vec<Item>, cur: &mut Vec<Tok>) {
        for node in nodes {
            match node {
                Inline::Text { text, span } => self.text(text, *span, ctx, cur),
                Inline::Element(el) => self.element(el, ctx, out, cur),
            }
        }
    }

    fn text(&mut self, text: &str, span: Span, ctx: &Ctx, cur: &mut Vec<Tok>) {
        for c in text.chars() {
            if let Some(tone) = Tone::from_char(c) {
                match cur.last_mut() {
                    Some(t) if t.kind == TokKind::Cell => t.tone = Some(tone),
                    _ => self.diags.push(Diagnostic::warning(
                        span,
                        format!("圈發「{c}」前面沒有可附的字"),
                    )),
                }
                continue;
            }
            if is_combining(c) {
                if let Some(t) = cur.last_mut() {
                    t.text.push(c);
                }
                continue;
            }
            if let Some((kind, shown)) = classify(c, self.punct) {
                cur.push(Tok {
                    text: shown.to_string(),
                    kind,
                    tone: None,
                    style: ctx.style.clone(),
                    compose: false,
                    rod: false,
                    hidden: false,
                });
            }
        }
    }

    fn mark(&mut self, ctx: &Ctx, kind: MarkKind) -> Ctx {
        let mut c = ctx.clone();
        c.style.marks.push(Mark {
            id: self.next_id,
            kind,
        });
        self.next_id += 1;
        c
    }

    fn element(&mut self, el: &Element, ctx: &Ctx, out: &mut Vec<Item>, cur: &mut Vec<Tok>) {
        let styled = |this: &mut Self, kind| this.mark(ctx, kind);
        match &el.kind {
            ElementKind::Name => {
                let c = styled(self, MarkKind::Name);
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Title { brackets } => match (brackets, self.title_mark) {
                // 保留括號：括號就是普通標點。
                (Some((open, close)), TitleMark::Bracket) => {
                    self.text(&open.to_string(), el.span, ctx, cur);
                    self.walk(&el.children, ctx, out, cur);
                    self.text(&close.to_string(), el.span, ctx, cur);
                }
                // 画波綫：括號隐藏，只留给复制。
                _ => {
                    let hidden = |c: &char, kind| Tok {
                        text: c.to_string(),
                        kind,
                        tone: None,
                        style: Style::default(),
                        compose: false,
                        rod: false,
                        hidden: true,
                    };
                    if let Some((open, _)) = brackets {
                        cur.push(hidden(open, TokKind::Before));
                    }
                    let c = styled(self, MarkKind::Title);
                    self.walk(&el.children, &c, out, cur);
                    if let Some((_, close)) = brackets {
                        cur.push(hidden(close, TokKind::After));
                    }
                }
            },
            ElementKind::Emphasis(m) => {
                let c = styled(self, MarkKind::Emphasis(*m));
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Stroke => {
                let c = styled(self, MarkKind::Stroke);
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Inverse(s) => {
                let c = styled(self, MarkKind::Inverse(*s));
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Frame(s) => {
                let c = styled(self, MarkKind::Frame(*s));
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Delete => {
                let mut c = styled(self, MarkKind::Delete);
                c.style.color = Some(Color::Gray);
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Insert => {
                let mut c = ctx.clone();
                c.style.color = Some(Color::Blue);
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Color(color) => {
                let mut c = ctx.clone();
                c.style.color = Some(*color);
                self.walk(&el.children, &c, out, cur);
            }
            ElementKind::Rod(digits) => {
                // 照原版：整串碼子字排在一格裏，由「華數碼」字體畫成算籌。
                const 碼子: [char; 10] =
                    ['〇', '〡', '〢', '〣', '〤', '〥', '〦', '〧', '〨', '〩'];
                if !digits.is_empty() {
                    cur.push(Tok {
                        text: digits
                            .bytes()
                            .map(|d| 碼子[usize::from(d - b'0')])
                            .collect(),
                        kind: TokKind::Cell,
                        tone: None,
                        style: ctx.style.clone(),
                        compose: false,
                        rod: true,
                        hidden: false,
                    });
                }
            }
            ElementKind::Span { prev, n } => {
                if let Some(inside) = ctx.nested {
                    self.diags.push(Diagnostic::warning(
                        el.span,
                        format!("{inside}裏不能兼行，按普通文字排"),
                    ));
                    self.walk(&el.children, ctx, out, cur);
                    return;
                }
                let mut c = ctx.clone();
                c.nested = Some("兼行");
                let mut buf = Vec::new();
                self.walk(&el.children, &c, out, &mut buf);
                flush(out, cur);
                out.push(Item::Span {
                    toks: buf,
                    prev: *prev,
                    n: *n,
                    at: el.span,
                });
            }
            ElementKind::Compose(ids) => {
                if !ids.is_empty() {
                    cur.push(Tok {
                        text: ids.clone(),
                        kind: TokKind::Cell,
                        tone: None,
                        style: ctx.style.clone(),
                        compose: true,
                        rod: false,
                        hidden: false,
                    });
                }
            }
            ElementKind::Break { raise } => {
                if let Some(inside) = ctx.nested {
                    self.diags.push(Diagnostic::warning(
                        el.span,
                        format!("{inside}裏不能擡頭換行，已忽略"),
                    ));
                    return;
                }
                flush(out, cur);
                out.push(Item::Break { raise: *raise });
            }
            ElementKind::DoubleNote
            | ElementKind::SingleNote
            | ElementKind::HalfNote { .. }
            | ElementKind::Side { .. }
            | ElementKind::Comment
            | ElementKind::HeadNote => {
                let name = match el.kind {
                    ElementKind::DoubleNote => "雙行夾注",
                    ElementKind::SingleNote => "單行小注",
                    ElementKind::HalfNote { .. } => "側行小注",
                    ElementKind::HeadNote => "眉批",
                    _ => "旁行",
                };
                if let Some(inside) = ctx.nested {
                    self.diags.push(Diagnostic::warning(
                        el.span,
                        format!("{inside}裏不能再有{name}，按普通文字排"),
                    ));
                    self.walk(&el.children, ctx, out, cur);
                    return;
                }
                let mut c = ctx.clone();
                c.nested = Some(name);
                if matches!(el.kind, ElementKind::Comment) && c.style.color.is_none() {
                    c.style.color = Some(Color::Red);
                }
                let mut buf = Vec::new();
                self.walk(&el.children, &c, out, &mut buf);
                flush(out, cur);
                out.push(match el.kind {
                    ElementKind::DoubleNote => Item::Double(buf),
                    ElementKind::SingleNote => Item::Single(buf),
                    ElementKind::HalfNote { left } => Item::Half { toks: buf, left },
                    ElementKind::Side { left } => Item::Side { toks: buf, left },
                    ElementKind::Comment => Item::Side {
                        toks: buf,
                        left: false,
                    },
                    _ => Item::Head(buf),
                });
            }
        }
    }
}

/// 算籌數碼：個位、百位……用縱式，十位、千位……用橫式（「一縱十橫」）。零作〇。
pub fn rod_char(d: u8, vertical: bool) -> char {
    match d {
        0 => '〇',
        d => {
            let base = if vertical { 0x1D360 } else { 0x1D369 };
            char::from_u32(base + u32::from(d) - 1).expect("counting rod range")
        }
    }
}

/// 算籌字符 → (數字, 是否縱式)。
pub fn rod_digit(c: char) -> Option<(u8, bool)> {
    match c as u32 {
        v @ 0x1D360..=0x1D368 => Some(((v - 0x1D360 + 1) as u8, true)),
        v @ 0x1D369..=0x1D371 => Some(((v - 0x1D369 + 1) as u8, false)),
        _ => None,
    }
}

fn flush(out: &mut Vec<Item>, cur: &mut Vec<Tok>) {
    if !cur.is_empty() {
        out.push(Item::Main(std::mem::take(cur)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::inline::parse_inline;

    fn items_with(src: &str, mark: TitleMark) -> (Vec<Item>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let nodes = parse_inline(src, 0, &mut d);
        let r = Flattener::new(PunctStyle::Judou, mark, 0, &mut d).line(&nodes);
        (r, d)
    }

    fn items(src: &str) -> (Vec<Item>, Vec<Diagnostic>) {
        items_with(src, TitleMark::Wavy)
    }

    fn text(t: &[Tok]) -> String {
        t.iter().map(|t| t.text.as_str()).collect()
    }

    #[test]
    fn splits_items() {
        let (r, d) = items(r"甲\note{乙丙}丁\\[1]戊\head{眉}");
        assert!(d.is_empty(), "{d:?}");
        let shape: Vec<String> = r
            .iter()
            .map(|i| match i {
                Item::Main(t) => format!("M{}", text(t)),
                Item::Double(t) => format!("D{}", text(t)),
                Item::Head(t) => format!("H{}", text(t)),
                Item::Break { raise } => format!("B{raise}"),
                other => format!("{other:?}"),
            })
            .collect();
        assert_eq!(shape, ["M甲", "D乙丙", "M丁", "B1", "M戊", "H眉"]);
    }

    #[test]
    fn marks_and_tones() {
        let (r, d) = items(r"\name{朝〪陽}");
        assert!(d.is_empty());
        let Item::Main(t) = &r[0] else { panic!() };
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].tone, Some(Tone::Level));
        assert_eq!(t[0].style.marks, t[1].style.marks);
        assert_eq!(t[0].style.marks[0].kind, MarkKind::Name);
    }

    #[test]
    fn nested_note_is_flattened_with_warning() {
        let (r, d) = items(r"\note{甲\note{乙}}");
        assert_eq!(d.len(), 1);
        let Item::Double(t) = &r[0] else { panic!() };
        assert_eq!(text(t), "甲乙");
    }

    #[test]
    fn title_brackets_follow_style() {
        let (r, _) = items("《史記》");
        let Item::Main(t) = &r[0] else { panic!() };
        assert_eq!(text(t), "《史記》");
        assert!(t[0].hidden && t[3].hidden && !t[1].hidden);
        assert_eq!(t[1].style.marks[0].kind, MarkKind::Title);

        let (r, _) = items_with("《史記》", TitleMark::Bracket);
        let Item::Main(t) = &r[0] else { panic!() };
        assert_eq!(text(t), "《史記》");
        assert!(t.iter().all(|t| !t.hidden && t.style.marks.is_empty()));
    }
}
