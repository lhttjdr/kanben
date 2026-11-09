//! 字格排版：把语法树排成一列一列的字。
//!
//! 坐标系（单位都是「一个字」）：
//! - `col`：第几行（版面上的一列），从右往左数，从 0 开始，不分葉；分葉由渲染层做。
//! - `x`：相对该行中线的水平偏移，向右为正。
//! - `y`：字心到版框内上沿的距离，向下为正；擡頭和眉批是负数。
//!
//! 排版只按字格计算，不依赖字体度量，所以结果是确定的。

mod engine;
pub mod flatten;
pub mod options;
pub mod token;

use crate::syntax::ast::{Block, Color, Document, EmphMark, Shape};
use crate::syntax::span::Diagnostic;
use engine::TocEntry;
pub use options::{Font, Format, Options, PunctStyle, TitleMark};

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub col: i32,
    pub x: f32,
    pub y: f32,
    /// 字号（1 = 正文）。
    pub size: f32,
    /// 纵向拉伸（夾注小字略长）。
    pub stretch: f32,
    pub text: String,
    /// `text` 是 IDS 组字式。
    pub compose: bool,
    /// 算籌數碼（見 [`token::Tok::rod`]）。
    pub rod: bool,
    pub color: Option<Color>,
    /// 陰文：用纸色写字。
    pub inverse: bool,
    /// 透明：只为复制时文字完整（画成波綫的書名號的括號）。
    pub hidden: bool,
    /// 淡（原版刪除的字 60% 不透明）。
    pub faded: bool,
    /// 在陰文、墨圍裏（原版是 `cite`，另用一套字體）。
    pub encl: bool,
    pub role: Role,
    /// 標題級別（0 = 不是標題）；標題用原版的標題字體與字重。
    pub level: u8,
    /// 段落字色（`\colors`），CSS 顏色；優先於 `color`。
    pub paint: Option<&'static str>,
    /// 特殊段落（撰者、綱、目、按語、琵琶譜）的字體類。
    pub extra: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Main,
    Note,
    /// 單行小注（原版 `dhgd`）。
    NoteSingle,
    /// 圈發（原版是「標點符號」字體裏的圈）。
    Tone,
    /// 單側行（原版 rightLine、leftLine）：正文字體的 0.8 倍小字。
    Line,
    /// 字上附的小標（韻標、平仄、樂譜旁字），字體類在 `extra`。
    Ann,
    Side,
    Head,
    Punct,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Deco {
    /// 書名、篇名的波綫：原版是瀏覽器畫的 `text-decoration: wavy`。這裏同樣交給瀏覽器：
    /// 在字的位置再放一串同樣的字（字透明，只顯波綫），起點是首字字盒的上沿。
    Wave {
        col: i32,
        x: f32,
        y0: f32,
        y1: f32,
        text: String,
        /// 篇名（右側，overline）；否則書名（左側，underline）。
        right: bool,
        /// 原版書名號頭尾用紙色小塊蓋去波綫的端點。
        first: bool,
        last: bool,
    },
    /// 照原版 CSS 畫的圖形（陰文底、墨圍綫、專名號、圈點、校勘底色等）。
    Shape {
        col: i32,
        figure: Figure,
        /// 填色的樣式類（見 style.css 的 `f-*`）。
        class: &'static str,
        /// 畫在字的上面（如刪除綫）；否則在字的下面。
        over: bool,
    },
    /// 陰文底或墨圍框。
    Box {
        col: i32,
        x: f32,
        half_w: f32,
        y0: f32,
        y1: f32,
        shape: Shape,
        filled: bool,
    },
    /// 逐字的圓圈（帶圈字、圓形陰文）。
    Ring {
        col: i32,
        x: f32,
        y: f32,
        r: f32,
        filled: bool,
        /// 六角（疊燈籠），否则是圆。
        hex: bool,
    },
    /// 專名號、書名號、抹、刪除綫。
    Line {
        col: i32,
        x: f32,
        y0: f32,
        y1: f32,
        wavy: bool,
        color: Option<Color>,
        /// 綫宽（字）。
        width: f32,
    },
    /// 圈點。
    Mark {
        col: i32,
        x: f32,
        y: f32,
        size: f32,
        mark: EmphMark,
        color: Option<Color>,
    },
    /// 四聲圈發。
    Tone { col: i32, x: f32, y: f32, r: f32 },
    /// 兼行处断开界行：用纸色盖住。
    Gap {
        col: i32,
        x: f32,
        half_w: f32,
        y0: f32,
        y1: f32,
    },
}

/// 圖形，坐標相對行中綫與文字頂（單位：字）。
#[derive(Clone, Debug, PartialEq)]
pub enum Figure {
    /// 矩形，可帶圓角（`rx` 橫向、`ry` 縱向半徑，同 CSS 的 `border-radius: rx/ry`）。
    Rect {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        rx: f32,
        ry: f32,
    },
    Circle {
        x: f32,
        y: f32,
        r: f32,
    },
    /// 圓環：內外半徑之間。
    Ring {
        x: f32,
        y: f32,
        r0: f32,
        r1: f32,
    },
    Polygon(Vec<(f32, f32)>),
    /// 前者減去後者（原版用內陰影畫的綫）。
    Minus(Box<Figure>, Box<Figure>),
    /// 前者只留在矩形 `(x0, y0, x1, y1)` 內。
    Clip(Box<Figure>, [f32; 4]),
}

impl Figure {
    /// 把縱坐標都換成 `f(y)`（整體挪動、拉伸時用）。
    pub fn map_y(&mut self, f: impl Fn(f32) -> f32 + Copy) {
        match self {
            Figure::Rect { y0, y1, .. } => {
                *y0 = f(*y0);
                *y1 = f(*y1);
            }
            Figure::Circle { y, .. } | Figure::Ring { y, .. } => *y = f(*y),
            Figure::Polygon(pts) => pts.iter_mut().for_each(|p| p.1 = f(p.1)),
            Figure::Minus(a, b) => {
                a.map_y(f);
                b.map_y(f);
            }
            Figure::Clip(a, r) => {
                a.map_y(f);
                r[1] = f(r[1]);
                r[3] = f(r[3]);
            }
        }
    }
}

impl Deco {
    pub fn col(&self) -> i32 {
        match *self {
            Deco::Shape { col, .. }
            | Deco::Wave { col, .. }
            | Deco::Box { col, .. }
            | Deco::Ring { col, .. }
            | Deco::Line { col, .. }
            | Deco::Mark { col, .. }
            | Deco::Tone { col, .. }
            | Deco::Gap { col, .. } => col,
        }
    }

    pub fn set_col(&mut self, c: i32) {
        match self {
            Deco::Shape { col, .. }
            | Deco::Wave { col, .. }
            | Deco::Box { col, .. }
            | Deco::Ring { col, .. }
            | Deco::Line { col, .. }
            | Deco::Mark { col, .. }
            | Deco::Tone { col, .. }
            | Deco::Gap { col, .. } => *col = c,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub options: Options,
    /// 正文共用了多少行（版面上的列）。
    pub columns: i32,
    /// 連天頭的眉批一起算，最左用到第幾行之後（長卷紙寬要包住眉批）。
    pub extent: i32,
    pub glyphs: Vec<Glyph>,
    pub decos: Vec<Deco>,
}

/// 有眉批時天頭至少這麼高：連紙邊共約 4.8 字，與原版天頭（16%，約 4.7 字）相近。
const HEAD_FOR_NOTES: f32 = 4.0;

pub fn layout(doc: &Document, mut options: Options, diags: &mut Vec<Diagnostic>) -> Layout {
    adjust(doc, &mut options);
    layout_with(doc, options, diags)
}

/// 按内容调整版式：有眉批而天頭没指定时加高天頭。
pub fn adjust(doc: &Document, options: &mut Options) {
    if !options.head_set && options.head < HEAD_FOR_NOTES && has_head_notes(doc) {
        options.head = HEAD_FOR_NOTES;
    }
}

fn has_head_notes(doc: &Document) -> bool {
    use crate::syntax::ast::{ElementKind, Inline};
    fn any(nodes: &[Inline]) -> bool {
        nodes.iter().any(|n| match n {
            Inline::Element(e) => e.kind == ElementKind::HeadNote || any(&e.children),
            Inline::Text { .. } => false,
        })
    }
    fn blocks(bs: &[Block]) -> bool {
        bs.iter().any(|b| match b {
            Block::Heading { content, .. } => any(content),
            Block::Paragraph { lines, .. } => lines.iter().any(|l| any(&l.content)),
            Block::Env { blocks: inner, .. } => blocks(inner),
            _ => false,
        })
    }
    blocks(&doc.blocks)
}

fn layout_with(doc: &Document, options: Options, diags: &mut Vec<Diagnostic>) -> Layout {
    if !doc.blocks.iter().any(|b| matches!(b, Block::Toc { .. })) {
        return engine::Engine::new(options, diags).run(doc);
    }
    // 有目次：先排一遍得出各標題在第幾葉，再带着葉碼重排；葉碼变了就再来，最多几遍。
    let per_leaf = options.columns * 2;
    let leaves = options.format == Format::Leaves;
    let mut toc: Vec<TocEntry> = Vec::new();
    for _ in 0..4 {
        let mut scratch = Vec::new();
        let (_, heads) = engine::Engine::new(options.clone(), &mut scratch)
            .with_toc(toc.clone())
            .run_collect(doc);
        let next: Vec<TocEntry> = heads
            .into_iter()
            .map(|(mut e, col)| {
                e.leaf = leaves.then(|| col.div_euclid(per_leaf));
                e
            })
            .collect();
        if next == toc {
            break;
        }
        toc = next;
    }
    engine::Engine::new(options, diags).with_toc(toc).run(doc)
}
