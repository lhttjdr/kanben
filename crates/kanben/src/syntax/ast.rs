//! 语法树。

use super::span::Span;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {
    /// `\setup[键=值]` 以及 `\book{}` `\volume{}` `\studio{}`。
    pub meta: Vec<MetaEntry>,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetaEntry {
    pub key: String,
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// `#` 至 `######`。
    Heading {
        level: u8,
        content: Vec<Inline>,
        span: Span,
    },
    /// 段落。段内每个换行都是一次平擡（另起一行）。
    /// 段首的属性管整段；之后某行行首的属性只管那一行，并覆盖同类的段落属性。
    Paragraph {
        attrs: Vec<Attr>,
        lines: Vec<Line>,
        span: Span,
    },
    /// 多出来的空行，每个占一整行（版面上的一列）。
    Blank { count: usize, span: Span },
    /// `\\toc`：目次，列出 `depth` 級以內的標題及葉碼。
    Toc { depth: u8, span: Span },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub attrs: Vec<Attr>,
    pub content: Vec<Inline>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attr {
    pub kind: AttrKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttrKind {
    /// `@低N`：整段低 N 格。
    Indent(u8),
    /// `@擡N`：段首高出版框 N 格。
    Raise(u8),
    /// `@底N`：每行靠底排，离底 N 格。
    Bottom(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    Text { text: String, span: Span },
    Element(Element),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub kind: ElementKind,
    pub children: Vec<Inline>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementKind {
    /// 雙行夾注。
    DoubleNote,
    /// 單行小注。
    SingleNote,
    /// 側行小注：只排右（左）半邊的小字，佔正文字格。
    HalfNote { left: bool },
    /// 旁行小字（右或左），不占正文字格。
    Side { left: bool },
    /// 旁批（朱色旁行小字）。
    Comment,
    /// 眉批，排在天頭。
    HeadNote,
    /// 專名號（左側直綫）。
    Name,
    /// 書名。`brackets` 是正文里的書名號（如 `《》`），`\title{}` 则没有。
    /// 画成波綫还是括號由版式决定。
    Title { brackets: Option<(char, char)> },
    /// 圈點（右側逐字）。
    Emphasis(EmphMark),
    /// 抹（右側朱綫）。
    Stroke,
    /// 陰文（黑底白字）。
    Inverse(Encl),
    /// 墨圍（框）。
    Frame(Encl),
    /// 校勘：刪。
    Delete,
    /// 校勘：增。
    Insert,
    /// 字色。
    Color(Color),
    /// 以 IDS 組字，如 `⿰木鬼`。
    Compose(String),
    /// `\\` 或 `\\[N]`：另起一行，並高出 N 格。
    Break { raise: u8 },
    /// 兼行：這段字跨到後（前）面 `n` 行，排在幾行的正中。
    Span { prev: bool, n: u8 },
    /// 算籌數碼，內容是阿拉伯數字。
    Rod(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EmphMark {
    /// ◦
    Circle,
    /// •
    Dot,
    /// ﹅
    Sesame,
    /// ﹆
    Comma,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shape {
    /// 方頭
    Square,
    /// 圓頭
    Round,
    /// 六角
    Hex,
    /// 花頭
    Flower,
    /// 逐字圓圈（帶圈字、圓形陰文）
    Each,
    /// 疊燈籠：逐字六角，上下相接
    Lantern,
}

/// 陰文、墨圍的樣式。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Encl {
    pub shape: Shape,
    /// 窄式：不貼界行。
    pub narrow: bool,
}

impl Shape {
    /// 逐字一個圈或六角，而不是整段連成一個框。
    pub fn is_each(self) -> bool {
        matches!(self, Shape::Each | Shape::Lantern)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Ink,
    Red,
    Blue,
    Green,
    Ochre,
    Gray,
}

impl Color {
    pub fn css_class(self) -> &'static str {
        match self {
            Color::Ink => "c-ink",
            Color::Red => "c-red",
            Color::Blue => "c-blue",
            Color::Green => "c-green",
            Color::Ochre => "c-ochre",
            Color::Gray => "c-gray",
        }
    }

    pub const NAMES: &str = "ink, red, blue, green, ochre, gray";

    pub fn from_name(s: &str) -> Option<Color> {
        Some(match s {
            "ink" => Color::Ink,
            "red" => Color::Red,
            "blue" => Color::Blue,
            "green" => Color::Green,
            "ochre" => Color::Ochre,
            "gray" => Color::Gray,
            _ => return None,
        })
    }
}
