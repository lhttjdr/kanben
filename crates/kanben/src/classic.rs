//! 把刊本源碼譯成原版排版器的標記，交給原版排法（web/classic）排印。
//!
//! 譯法按含義對應，例如 `\note{…}` → `((=…=))`、`\name{…}` → `‹…›`、`\inv[round]{…}` → `(【…】)`。
//! 原版沒有的寫法取最接近的，並給出提示：行內字色略去；花頭作平頭；
//! 整段低格、靠底排用原版的段樣式 `({首縮進:n})`、`({⇥;尾縮進:n})`。

use crate::syntax::ast::{AttrKind, Block, Document, ElementKind, EmphMark, Inline, Shape};
use crate::syntax::parse;
use crate::syntax::span::{Diagnostic, Span};

/// 譯好的結果。
pub struct Classic {
    /// 原版標記的文本。
    pub text: String,
    /// 書名（`\book{…}`），原版放在版心。
    pub book: String,
    pub diagnostics: Vec<Diagnostic>,
}

/// 刊本源碼 → 原版標記。
pub fn to_classic(src: &str) -> Classic {
    let parsed = parse(src);
    let mut t = Translator {
        diags: parsed.diagnostics,
    };
    let text = t.document(&parsed.document);
    let book = parsed
        .document
        .meta
        .iter()
        .rev()
        .find(|m| m.key == "book")
        .map(|m| m.value.clone())
        .unwrap_or_default();
    let mut diagnostics = t.diags;
    diagnostics.sort_by_key(|d| d.span.start);
    Classic {
        text,
        book,
        diagnostics,
    }
}

const 全角空格: char = '　';
/// 原版眉批的括號是 U+2329、U+232A（不是〈〉）。
const 眉批起: &str = "(?\u{2329}";
const 眉批訖: &str = "\u{232A}¿)";

struct Translator {
    diags: Vec<Diagnostic>,
}

impl Translator {
    fn note(&mut self, span: Span, msg: &str) {
        self.diags
            .push(Diagnostic::warning(span, format!("原版排法：{msg}")));
    }

    fn document(&mut self, doc: &Document) -> String {
        let mut out = String::new();
        let mut first = true;
        for (i, block) in doc.blocks.iter().enumerate() {
            if let Block::Blank { count, .. } = block {
                // 原版：段與段之間空一行；每多空一行，版面上多一空行。
                out.push_str(&"\n".repeat(*count));
                continue;
            }
            if !first {
                out.push_str("\n\n");
            }
            first = false;
            match block {
                Block::Heading { level, content, .. } => {
                    out.push('*');
                    out.extend(std::iter::repeat_n(全角空格, usize::from(*level)));
                    out.push_str(&self.inlines(content));
                }
                Block::Paragraph { attrs, lines, .. } => {
                    // 整段低格用原版的段樣式「首縮進」（整段下移，折行也低格），靠底用「⇥」。
                    let mut indent = 0usize;
                    let mut raise = 0;
                    let mut bottom: Option<u8> = None;
                    for a in attrs {
                        match a.kind {
                            AttrKind::Indent(n) => indent += usize::from(n),
                            AttrKind::Raise(n) => raise = n,
                            AttrKind::Bottom(n) => bottom = Some(n),
                        }
                    }
                    if indent > 9 || bottom.is_some_and(|n| n > 9) {
                        self.note(Span::default(), "原版的段首、段尾縮進最多 9 格");
                    }
                    let mut style = Vec::new();
                    if indent > 0 {
                        style.push(format!("首縮進:{}", indent.min(9)));
                    }
                    if let Some(n) = bottom {
                        style.push("⇥".to_string());
                        if n > 0 {
                            style.push(format!("尾縮進:{}", n.min(9)));
                        }
                    }
                    if !style.is_empty() {
                        out.push_str(&format!("({{{}}})", style.join(";")));
                    }
                    for (k, line) in lines.iter().enumerate() {
                        if k > 0 {
                            out.push('\n');
                        }
                        // 段落的低格已由段樣式負責；行首屬性只管這一行（用行首空格）。
                        let mut line_indent = 0;
                        let mut line_raise = if k == 0 { raise } else { 0 };
                        for a in &line.attrs {
                            match a.kind {
                                AttrKind::Indent(n) => line_indent = usize::from(n),
                                AttrKind::Raise(n) => line_raise = n,
                                AttrKind::Bottom(_) => {
                                    self.note(a.span, "原版的靠底排只能管整段，這一行按普通行排")
                                }
                            }
                        }
                        out.push_str(擡(line_raise));
                        out.extend(std::iter::repeat_n(全角空格, line_indent));
                        out.push_str(&self.inlines(&line.content));
                    }
                }
                Block::Toc { depth, .. } => {
                    out.push_str("〚目次〛");
                    for b in &doc.blocks[i + 1..] {
                        if let Block::Heading { level, content, .. } = b
                            && level <= depth
                        {
                            out.push('\n');
                            out.push_str(&plain(content));
                        }
                    }
                }
                Block::Blank { .. } => unreachable!(),
            }
        }
        out
    }

    fn inlines(&mut self, nodes: &[Inline]) -> String {
        nodes.iter().map(|n| self.inline(n)).collect()
    }

    fn inline(&mut self, node: &Inline) -> String {
        let el = match node {
            Inline::Text { text, .. } => return text.clone(),
            Inline::Element(el) => el,
        };
        let body = self.inlines(&el.children);
        let wrap = |a: &str, b: &str| format!("{a}{body}{b}");
        match &el.kind {
            ElementKind::DoubleNote => wrap("((=", "=))"),
            ElementKind::SingleNote => wrap("((-", "-))"),
            ElementKind::HalfNote { left: false } => wrap("᚜(¯", "¯)᚛"),
            ElementKind::HalfNote { left: true } => wrap("᚜(_", "_)᚛"),
            ElementKind::Side { left: false } | ElementKind::Comment => wrap("(?¯", "¯¿)"),
            ElementKind::Side { left: true } => wrap("᚜[_", "_]᚛"),
            ElementKind::HeadNote => wrap(眉批起, 眉批訖),
            ElementKind::Name => wrap("‹", "›"),
            ElementKind::Title {
                brackets: Some((o, c)),
            } => format!("{o}{body}{c}"),
            ElementKind::Title { brackets: None } => wrap("《", "》"),
            ElementKind::Emphasis(m) => {
                let c = match m {
                    EmphMark::Circle => '◦',
                    EmphMark::Dot => '•',
                    EmphMark::Sesame => '﹅',
                    EmphMark::Comma => '﹆',
                };
                format!("!!¯{c}{body}{c}¯¡¡")
            }
            ElementKind::Stroke => wrap("??-", "-¿¿"),
            ElementKind::Inverse(e) => {
                let (a, b) = match e.shape {
                    Shape::Square => ("[【", "】]"),
                    Shape::Round => ("(【", "】)"),
                    Shape::Hex => ("〔【", "】〕"),
                    Shape::Each => ("![●", "●]¡"),
                    Shape::Lantern => ("![⬣", "⬣]¡"),
                    Shape::Flower => {
                        self.note(el.span, "原版沒有花頭陰文，改作平頭");
                        ("[【", "】]")
                    }
                };
                if e.narrow {
                    self.note(el.span, "原版的陰文沒有窄式");
                }
                wrap(a, b)
            }
            ElementKind::Frame(e) => {
                let (a, b) = match (e.shape, e.narrow) {
                    (Shape::Round, true) => ("﹙〖", "〗﹚"),
                    (Shape::Hex, true) => ("﹝〖", "〗﹞"),
                    (Shape::Square, _) => ("[〖", "〗]"),
                    (Shape::Round, _) => ("(〖", "〗)"),
                    (Shape::Hex, _) => ("〔〖", "〗〕"),
                    (Shape::Each, _) => ("![○", "○]¡"),
                    (Shape::Lantern, _) => ("![⎔", "⎔]¡"),
                    (Shape::Flower, _) => {
                        self.note(el.span, "原版沒有花頭墨圍，改作平頭");
                        ("[〖", "〗]")
                    }
                };
                if e.narrow && !matches!(e.shape, Shape::Round | Shape::Hex) {
                    self.note(el.span, "原版的窄式墨圍只有圓頭、六角兩種");
                }
                wrap(a, b)
            }
            ElementKind::Delete => wrap("{（", "）}"),
            ElementKind::Insert => wrap("{［", "］}"),
            ElementKind::Color(_) => {
                self.note(el.span, "原版沒有行內字色，已略去顏色");
                body
            }
            ElementKind::Compose(ids) => format!("[[?{ids}¿]]"),
            ElementKind::Break { raise } => format!("\n{}", 擡(*raise)),
            ElementKind::Span { prev, n } => {
                let k = if *prev { "歬" } else { "後" };
                format!("〚{k}跨{n}行］{body}［{k}跨〛")
            }
            ElementKind::Rod(digits) => {
                const 碼子: [char; 10] =
                    ['〇', '〡', '〢', '〣', '〤', '〥', '〦', '〧', '〨', '〩'];
                let codes: String = digits
                    .bytes()
                    .map(|d| 碼子[usize::from(d - b'0')])
                    .collect();
                format!("〚筭］{codes}［筭〛")
            }
        }
    }
}

/// 擡頭標記：原版只有單、雙、三擡。
fn 擡(n: u8) -> &'static str {
    match n {
        0 => "",
        1 => "〚單擡〛",
        2 => "〚雙擡〛",
        _ => "〚三擡〛",
    }
}

/// 標題的純文字（目次用）：只取正文，去掉注釋類元素。
fn plain(nodes: &[Inline]) -> String {
    let mut s = String::new();
    for n in nodes {
        match n {
            Inline::Text { text, .. } => s.push_str(text),
            Inline::Element(e) => match e.kind {
                ElementKind::DoubleNote
                | ElementKind::SingleNote
                | ElementKind::HalfNote { .. }
                | ElementKind::Side { .. }
                | ElementKind::Comment
                | ElementKind::HeadNote => {}
                ElementKind::Compose(ref ids) => s.push_str(ids),
                _ => s.push_str(&plain(&e.children)),
            },
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tr(src: &str) -> String {
        to_classic(src).text
    }

    #[test]
    fn inline_marks() {
        assert_eq!(tr(r"甲\note{乙，丙。}丁"), "甲((=乙，丙。=))丁");
        assert_eq!(tr(r"\name{孔子}曰《易》"), "‹孔子›曰《易》");
        assert_eq!(
            tr(r"\inv[round]{疏}\frame[hex, narrow]{目}"),
            "(【疏】)﹝〖目〗﹞"
        );
        assert_eq!(tr(r"\emph{甲乙}\stroke{丙}"), "!!¯◦甲乙◦¯¡¡??-丙-¿¿");
        assert_eq!(tr(r"\del{官}\ins{觀}"), "{（官）}{［觀］}");
        assert_eq!(tr(r"\head{批}"), "(?\u{2329}批\u{232A}¿)");
        assert_eq!(
            tr(r"\compose{⿰木鬼}\rod{105}"),
            "[[?⿰木鬼¿]]〚筭］〡〇〥［筭〛"
        );
        assert_eq!(
            tr(r"\span{明算}\span[prev]{校}"),
            "〚後跨1行］明算［後跨〛〚歬跨1行］校［歬跨〛"
        );
    }

    #[test]
    fn blocks() {
        assert_eq!(
            tr("\\heading{卷一}\n\n甲\n乙\n\n\n丙"),
            "*　卷一\n\n甲\n乙\n\n\n丙"
        );
        assert_eq!(tr("\\heading[2]{紀}"), "*　　紀");
        assert_eq!(tr("\\indent[2] 甲\n乙"), "({首縮進:2})甲\n乙");
        assert_eq!(tr("\\indent[2]\\bottom 甲"), "({首縮進:2;⇥})甲");
        assert_eq!(tr("\\raise[1] 甲"), "〚單擡〛甲");
        assert_eq!(tr("\\bottom 某撰"), "({⇥})某撰");
        assert_eq!(tr("\\bottom[1] 某撰"), "({⇥;尾縮進:1})某撰");
        assert_eq!(tr("甲\\\\[1]乙"), "甲\n〚單擡〛乙");
    }

    #[test]
    fn toc_and_book() {
        let c = to_classic("\\book{資治通鑑}\n\\toc\n\n\\heading{卷一}\n\n甲\n\n\\heading{卷二}");
        assert_eq!(c.book, "資治通鑑");
        assert!(c.text.starts_with("〚目次〛\n卷一\n卷二"), "{}", c.text);
    }

    #[test]
    fn unsupported_is_reported() {
        let c = to_classic(r"\color[blue]{藍}");
        assert_eq!(c.text, "藍");
        assert_eq!(c.diagnostics.len(), 1);
    }
}
