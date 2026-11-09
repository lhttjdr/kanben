//! 把刊本源碼譯成原版排版器的標記，交給原版排法（web/classic）排印。
//!
//! 譯法按含義對應，例如 `\note{…}` → `((=…=))`、`\name{…}` → `‹…›`、`\inv[round]{…}` → `(【…】)`。
//! 原版沒有的寫法取最接近的，並給出提示：行內字色略去；花頭作平頭；
//! 整段低格、靠底排用原版的段樣式 `({首縮進:n})`、`({⇥;尾縮進:n})`。

use crate::syntax::ast::{
    AttrKind, Block, Document, ElementKind, EmphMark, EnvKind, Inline, ParaStyle, Shape,
    StrokeMark, TableHeader,
};
use crate::syntax::parse;
use crate::syntax::span::{Diagnostic, Span};

/// 譯好的結果。
pub struct Classic {
    /// 原版標記的文本。
    pub text: String,
    /// 書名（`\book{…}`），原版放在版心。
    pub book: String,
    /// 卷名（`\volume{…}`）、堂號（`\studio{…}`），冊頁版心用。
    pub volume: String,
    pub studio: String,
    pub diagnostics: Vec<Diagnostic>,
}

/// 刊本源碼 → 原版標記。
pub fn to_classic(src: &str) -> Classic {
    let parsed = parse(src);
    let mut t = Translator {
        diags: parsed.diagnostics,
    };
    let text = t.document(&parsed.document);
    let meta = |key: &str| {
        parsed
            .document
            .meta
            .iter()
            .rev()
            .find(|m| m.key == key)
            .map(|m| m.value.clone())
            .unwrap_or_default()
    };
    let (book, volume, studio) = (meta("book"), meta("volume"), meta("studio"));
    let mut diagnostics = t.diags;
    diagnostics.sort_by_key(|d| d.span.start);
    Classic {
        text,
        book,
        volume,
        studio,
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
        self.blocks(&doc.blocks, &doc.blocks)
    }

    /// `all` 是整篇的塊，自動目次要從中找標題。
    fn blocks(&mut self, blocks: &[Block], all: &[Block]) -> String {
        let mut out = String::new();
        let mut first = true;
        for block in blocks {
            if let Block::Blank { count, .. } = block {
                // 原版：段與段之間空一行；每多空一行，版面上多一空行。
                out.push_str(&"\n".repeat(*count));
                continue;
            }
            if !first {
                // 緊接上一段的表格屬於那一段（原版：表格寫在段中，前後只隔一個換行）。
                let attached = matches!(
                    block,
                    Block::Env {
                        attached: true,
                        kind: EnvKind::Table { .. },
                        ..
                    }
                );
                out.push_str(if attached { "\n" } else { "\n\n" });
            }
            first = false;
            match block {
                Block::Heading { level, content, .. } => {
                    out.push('*');
                    out.extend(std::iter::repeat_n(全角空格, usize::from(*level)));
                    out.push_str(&self.inlines(content));
                }
                Block::Paragraph { attrs, lines, .. } => self.paragraph(attrs, lines, &mut out),
                Block::Toc { depth, entries, .. } => {
                    out.push_str("〚目次〛");
                    if entries.is_empty() {
                        let start = all
                            .iter()
                            .position(|b| std::ptr::eq(b, block))
                            .map_or(0, |i| i + 1);
                        for b in &all[start..] {
                            if let Block::Heading { level, content, .. } = b
                                && level <= depth
                            {
                                out.push('\n');
                                out.push_str(&plain(content));
                            }
                        }
                    } else {
                        for e in entries {
                            out.push('\n');
                            out.push_str(&self.inlines(&e.content));
                        }
                    }
                }
                Block::Env { kind, blocks, .. } => match kind {
                    EnvKind::Quote => {
                        out.push_str("‘‘‘");
                        out.push_str(&self.blocks(blocks, all));
                        out.push_str("’’’");
                    }
                    EnvKind::List { ordered } => {
                        let k = if *ordered { "列" } else { "集" };
                        out.push_str(&format!("〚{k}］\n"));
                        out.push_str(&self.blocks(blocks, all));
                        out.push_str(&format!("\n［{k}〛"));
                    }
                    EnvKind::Table { title, header } => self.table(title, *header, blocks, &mut out),
                },
                Block::Blank { .. } => unreachable!(),
            }
        }
        out
    }

    fn paragraph(&mut self, attrs: &[crate::syntax::ast::Attr], lines: &[crate::syntax::ast::Line], out: &mut String) {
        // 整段低格用原版的段樣式「首縮進」（整段下移，折行也低格），靠底用「⇥」。
        let mut indent = 0usize;
        let mut raise = 0;
        let mut bottom: Option<u8> = None;
        let mut colors: Option<&[String]> = None;
        let mut special = None;
        let mut list = None;
        for a in attrs {
            match &a.kind {
                AttrKind::Indent(n) => indent += usize::from(*n),
                AttrKind::Raise(n) => raise = *n,
                AttrKind::Bottom(n) => bottom = Some(*n),
                AttrKind::Colors(c) => colors = Some(c),
                AttrKind::Style(s) => special = Some(*s),
                AttrKind::List { ordered } => list = Some(*ordered),
            }
        }
        if indent > 9 || bottom.is_some_and(|n| n > 9) {
            self.note(Span::default(), "原版的段首、段尾縮進最多 9 格");
        }
        let mut style = Vec::new();
        if let Some(c) = colors {
            style.push(format!("字色:{}", c.join(",")));
        }
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
            if special.is_some() || list.is_some() {
                self.note(Span::default(), "原版的特殊段不能再加段樣式，已略去");
            } else {
                out.push_str(&format!("({{{}}})", style.join(";")));
            }
        }
        if let Some(s) = special {
            out.push_str(match s {
                ParaStyle::Byline => "〚譔者〛",
                ParaStyle::Outline => "〚綱〛",
                ParaStyle::Detail => "〚目〛",
                ParaStyle::Remark => "〚按〛",
                ParaStyle::Pipa => "〚琵琶〛",
            });
        }
        if let Some(ordered) = list {
            out.push_str(if ordered { "〚列〛" } else { "〚集〛" });
            // 一段之內各條用兩個 Tab 分開（原版的寫法）。
            let items: Vec<String> = lines.iter().map(|l| self.inlines(&l.content)).collect();
            out.push_str(&items.join("\t\t"));
            return;
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
                    _ => self.note(a.span, "原版的這種屬性只能管整段，這一行按普通行排"),
                }
            }
            out.push_str(擡(line_raise));
            out.extend(std::iter::repeat_n(全角空格, line_indent));
            out.push_str(&self.inlines(&line.content));
        }
    }

    /// 表格：原版用製表符畫框。格子後面是 ┼（或行末 ┤）就是表頭格，├（或行末 │）是普通格。
    fn table(&mut self, title: &str, header: TableHeader, blocks: &[Block], out: &mut String) {
        out.push('┌');
        out.push_str(title);
        out.push_str("──┐");
        let mut r = 0;
        for b in blocks {
            let Block::Paragraph { lines, .. } = b else {
                continue;
            };
            for line in lines {
                let text = self.inlines(&line.content);
                let cells: Vec<&str> = text.split('&').map(|c| c.trim_matches(' ')).collect();
                out.push_str("\n├");
                for (c, cell) in cells.iter().enumerate() {
                    let th = match header {
                        TableHeader::None => false,
                        TableHeader::Row => r == 0,
                        TableHeader::Column => c == 0,
                        TableHeader::Both => r == 0 || c == 0,
                    };
                    out.push_str(cell);
                    let last = c + 1 == cells.len();
                    out.push(match (th, last) {
                        (true, false) => '┼',
                        (true, true) => '┤',
                        (false, false) => '├',
                        (false, true) => '│',
                    });
                }
                r += 1;
            }
        }
        out.push_str("\n└──┘");
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
            ElementKind::Side { left: false } => wrap("᚜[¯", "¯]᚛"),
            ElementKind::Side { left: true } => wrap("᚜[_", "_]᚛"),
            ElementKind::Comment { stagger: false } => wrap("(?¯", "¯¿)"),
            ElementKind::Comment { stagger: true } => wrap("(?¯〚錯〛", "¯¿)"),
            ElementKind::PairNote => {
                // 以 \\ 分開右、左兩行。
                let mut parts = el.children.splitn(2, |n| {
                    matches!(n, Inline::Element(e) if matches!(e.kind, ElementKind::Break { .. }))
                });
                let right = parts.next().map(|p| self.inlines(p)).unwrap_or_default();
                let left = parts.next().map(|p| self.inlines(p)).unwrap_or_default();
                if left.is_empty() {
                    self.note(el.span, "並行要用 \\\\ 分開右、左兩行");
                }
                format!("᚜(={right}¯|_{left}=)᚛")
            }
            ElementKind::Endnote { anchor } => {
                let a = anchor
                    .as_ref()
                    .map(|a| format!("〔〔-{a}-〕〕"))
                    .unwrap_or_default();
                format!("((#{a}““={body}=””#))")
            }
            ElementKind::HeadNote => wrap(眉批起, 眉批訖),
            ElementKind::Name { double: false } => wrap("‹", "›"),
            ElementKind::Name { double: true } => wrap(r#"<u class="jmks">"#, "</u>"),
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
                    EmphMark::Box => 'ロ',
                };
                // 字右的點（﹅﹆ロ）原版逐字寫在字後：「字¯¯﹅」。單字、或方框，就這樣寫，
                // 譜字（如琵琶譜的假名）也標得上。
                // 單字帶旁行（如 \\emph[box]{七\\side{ㄣ}}），點寫在旁行之後。
                let single = main_len(&el.children) == 1;
                match m {
                    EmphMark::Sesame | EmphMark::Comma | EmphMark::Box if single => {
                        format!("{body}¯¯{c}")
                    }
                    EmphMark::Box => el
                        .children
                        .iter()
                        .map(|n| match n {
                            Inline::Text { text, .. } => {
                                text.chars().map(|ch| format!("{ch}¯¯{c}")).collect()
                            }
                            n => self.inline(n),
                        })
                        .collect(),
                    _ => format!("!!¯{c}{body}{c}¯¡¡"),
                }
            }
            ElementKind::Stroke(m) => match m {
                StrokeMark::Line => wrap("??-", "-¿¿"),
                StrokeMark::Dot => wrap("??•", "•¿¿"),
                StrokeMark::Circle => wrap("??◦", "◦¿¿"),
                StrokeMark::Tint => wrap("??", "¿¿"),
            },
            ElementKind::Rhyme { tone, group } => {
                let g = group.map(|g| g.to_string()).unwrap_or_default();
                format!("{body}[{g}{}]", tone.mark())
            }
            ElementKind::Ruby(t) => format!("{{{body}|{t}}}"),
            ElementKind::Pitch { text, left: false } => format!("{{{{{body}⏷^¯{text}¯^⏶}}}}"),
            ElementKind::Pitch { text, left: true } => format!("{{{{{body}⏷^_{text}_^⏶}}}}"),
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
            ElementKind::Rod(nums) => {
                // 幾個數連寫，負數前加「＼」。
                let codes: String = nums
                    .split(',')
                    .map(|n| match n.strip_prefix('-') {
                        Some(d) => format!("＼{}", 碼子(d)),
                        None => 碼子(n),
                    })
                    .collect();
                format!("〚筭］{codes}［筭〛")
            }
            ElementKind::Digits(t) => format!("〚筭］{t}［筭〛"),
            ElementKind::Suzhou(t) => format!("〚碼子］{}［碼子〛", 碼子(t)),
            ElementKind::Divination(t) => format!("〚數卦］{t}［數卦〛"),
            ElementKind::Glyph(t) => format!("〚字樣］{t}［字樣〛"),
            ElementKind::Figure => wrap("〚圖］", "［圖〛"),
            ElementKind::Image(f) => format!("〚圖片］{f}［圖片〛"),
            ElementKind::Map(n) => format!("〚地圖〛{n}"),
        }
    }
}

/// 正文的字數（不算注、旁行、眉批）。
fn main_len(nodes: &[Inline]) -> usize {
    nodes
        .iter()
        .map(|n| match n {
            Inline::Text { text, .. } => text.chars().count(),
            Inline::Element(e) => match e.kind {
                ElementKind::DoubleNote
                | ElementKind::SingleNote
                | ElementKind::PairNote
                | ElementKind::Endnote { .. }
                | ElementKind::HalfNote { .. }
                | ElementKind::Side { .. }
                | ElementKind::Comment { .. }
                | ElementKind::HeadNote => 0,
                _ => main_len(&e.children).max(1),
            },
        })
        .sum()
}

/// 阿拉伯數字 → 碼子字（〇〡〢…〩）。
fn 碼子(digits: &str) -> String {
    const 字: [char; 10] = ['〇', '〡', '〢', '〣', '〤', '〥', '〦', '〧', '〨', '〩'];
    digits
        .bytes()
        .filter(u8::is_ascii_digit)
        .map(|d| 字[usize::from(d - b'0')])
        .collect()
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
                | ElementKind::PairNote
                | ElementKind::Endnote { .. }
                | ElementKind::HalfNote { .. }
                | ElementKind::Side { .. }
                | ElementKind::Comment { .. }
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
    fn showcase_marks() {
        assert_eq!(tr(r"\side{臣}\side[left]{正}"), "᚜[¯臣¯]᚛᚜[_正_]᚛");
        assert_eq!(tr(r"\comment[stagger]{妙}"), "(?¯〚錯〛妙¯¿)");
        assert_eq!(tr(r"\note[pair]{屈原\\釋文}"), "᚜(=屈原¯|_釋文=)᚛");
        assert_eq!(
            tr(r"\endnote[年]{作歲。}\endnote{校。}"),
            "((#〔〔-年-〕〕““=作歲。=””#))((#““=校。=””#))"
        );
        assert_eq!(tr(r"\rhyme[ru,1]{翟}\ruby[◓]{鳳}"), "翟[1▲]{鳳|◓}");
        assert_eq!(tr(r"\pitch[清黃]{呦}"), "{{呦⏷^¯清黃¯^⏶}}");
        assert_eq!(tr(r"\emph[box]{七\side{ㄣ}}\emph[sesame]{ス}"), "七᚜[¯ㄣ¯]᚛¯¯ロス¯¯﹅");
        assert_eq!(tr(r"\stroke[dot]{甲}\stroke[tint]{乙}"), "??•甲•¿¿??乙¿¿");
        assert_eq!(tr(r"\name[double]{蘇軾}"), r#"<u class="jmks">蘇軾</u>"#);
        assert_eq!(
            tr(r"\rod{4720, -398}\rod[suzhou]{589}\rod[gua]{16}\digits{18:20}"),
            "〚筭］〤〧〢〇＼〣〩〨［筭〛〚碼子］〥〨〩［碼子〛〚數卦］16［數卦〛〚筭］18:20［筭〛"
        );
        assert_eq!(
            tr(r"\glyph{瓊}\figure{\image{a.png}\map{南徼圖}}"),
            "〚字樣］瓊［字樣〛〚圖］〚圖片］a.png［圖片〛〚地圖〛南徼圖［圖〛"
        );
    }

    #[test]
    fn showcase_blocks() {
        assert_eq!(tr("\\byline 某　擬\n乙"), "〚譔者〛某　擬\n乙");
        assert_eq!(tr("\\outline 綱\n\n\\detail 目"), "〚綱〛綱\n\n〚目〛目");
        assert_eq!(tr("\\list 永徽\n顯慶"), "〚列〛永徽\t\t顯慶");
        assert_eq!(tr("\\list[unordered] 甲\n乙"), "〚集〛甲\t\t乙");
        assert_eq!(
            tr("\\colors[孔雀綠, 葛巾紫] 甲\n\n\\colors[正黃] \\bottom 乙"),
            "({字色:孔雀綠,葛巾紫})甲\n\n({字色:正黃;⇥})乙"
        );
        assert_eq!(
            tr("甲\n\n\\begin{quote}\n乙\n\n丙\n\\end{quote}"),
            "甲\n\n‘‘‘乙\n\n丙’’’"
        );
        assert_eq!(
            tr("\\begin{list}[unordered]\n甲\n\n乙\n\\end{list}"),
            "〚集］\n甲\n\n乙\n［集〛"
        );
        assert_eq!(
            tr("例如：\n\\begin{table}[題, header=both]\n　 & 列一\n行一 & 格\n\\end{table}"),
            "例如：\n┌題──┐\n├　┼列一┤\n├行一┼格│\n└──┘"
        );
        assert_eq!(tr("\\toc\n甲\n乙\\note{丙}"), "〚目次〛\n甲\n乙((=丙=))");
    }

    #[test]
    fn unsupported_is_reported() {
        let c = to_classic(r"\color[blue]{藍}");
        assert_eq!(c.text, "藍");
        assert_eq!(c.diagnostics.len(), 1);
    }
}
