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
    /// 緊跟在 `\\toc` 後面（不空行）的各行是手寫的條目，有了就不自動列標題。
    Toc {
        depth: u8,
        entries: Vec<Line>,
        span: Span,
    },
    /// `\\begin{quote}` 之類的結構環境，裏面是若干塊。
    Env {
        kind: EnvKind,
        blocks: Vec<Block>,
        /// 緊接上一段（中間不空行）：表格這樣寫就屬於上一段。
        attached: bool,
        span: Span,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnvKind {
    /// 引文（原版 `‘‘‘…’’’`），整體低一格。
    Quote,
    /// 分段單兒：每段一條。`ordered` 是有序的列，否則是集。
    List { ordered: bool },
    /// 表格：每行一列，格子之間用 `&` 分開。
    Table { title: String, header: TableHeader },
}

/// 表頭在哪：首列、首行，或兩者。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableHeader {
    None,
    Row,
    Column,
    Both,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub attrs: Vec<Attr>,
    pub content: Vec<Inline>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attr {
    pub kind: AttrKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttrKind {
    /// `\\indent[N]`：整段低 N 格。
    Indent(u8),
    /// `\\raise[N]`：段首高出版框 N 格。
    Raise(u8),
    /// `\\bottom[N]`：每行靠底排，离底 N 格。
    Bottom(u8),
    /// 特殊段落：撰者、綱、目、按語、琵琶譜。
    Style(ParaStyle),
    /// `\\list`：段中每行是一條（列或集）。
    List { ordered: bool },
    /// `\\colors[甲, 乙]`：字色；給幾種就逐行輪換。
    Colors(Vec<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParaStyle {
    /// 撰者、校者署名（靠底，下留空）。
    Byline,
    /// 綱（大字）。
    Outline,
    /// 目。
    Detail,
    /// 按語。
    Remark,
    /// 琵琶譜。
    Pipa,
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
    /// 句段旁注（旁批）。`stagger`：字距拉開，與正文錯開。
    Comment { stagger: bool },
    /// 並行：左右兩行小字並排，`\\note[pair]{右\\\\左}`。
    PairNote,
    /// 尾注：文中只留編號，注文列在段後。`anchor` 是所注之字。
    Endnote { anchor: Option<String> },
    /// 眉批，排在天頭。
    HeadNote,
    /// 專名號（左側直綫）。`double`：兩側都有綫。
    Name { double: bool },
    /// 書名。`brackets` 是正文里的書名號（如 `《》`），`\title{}` 则没有。
    /// 画成波綫还是括號由版式决定。
    Title { brackets: Option<(char, char)> },
    /// 圈點（右側逐字）。
    Emphasis(EmphMark),
    /// 校勘標記：抹、點、圈、改色。
    Stroke(StrokeMark),
    /// 韻腳的四聲標記（抹在字右），`group` 是韻部（決定顏色）。
    Rhyme { tone: Tone, group: Option<u8> },
    /// 字右的小字標記，如平仄 ○●◒◓。
    Ruby(String),
    /// 樂譜旁字：字旁正書律呂等。
    Pitch { text: String, left: bool },
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
    /// 算籌數碼，內容是阿拉伯數字（可帶負號，多個數用逗號分開）。
    Rod(String),
    /// 縱中橫排的阿拉伯數字（可帶小數點、冒號）。
    Digits(String),
    /// 蘇州碼子，內容是阿拉伯數字。
    Suzhou(String),
    /// 數字卦（筮數）。
    Divination(String),
    /// 字書的字樣（字圖）。
    Glyph(String),
    /// 插圖：裏面是 `\\image`、`\\map`。
    Figure,
    /// 圖片文件。
    Image(String),
    /// 電子地圖。
    Map(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StrokeMark {
    /// 右側直綫（抹）。
    Line,
    /// 右側實點。
    Dot,
    /// 右側空圈。
    Circle,
    /// 改字色（校注所及之字）。
    Tint,
}

/// 四聲：平■上◆去▲入，平作●。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    Ping,
    Shang,
    Qu,
    Ru,
}

impl Tone {
    pub fn mark(self) -> char {
        match self {
            Tone::Ping => '●',
            Tone::Shang => '■',
            Tone::Qu => '◆',
            Tone::Ru => '▲',
        }
    }
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
    /// ロ（琵琶譜的方框）
    Box,
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
