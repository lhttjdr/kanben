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
    pub color: Option<Color>,
    /// 陰文：用纸色写字。
    pub inverse: bool,
    /// 透明：只为复制时文字完整（画成波綫的書名號的括號）。
    pub hidden: bool,
    pub role: Role,
    /// 粗體（標題）。
    pub bold: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Main,
    Note,
    Side,
    Head,
    Punct,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Deco {
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

impl Deco {
    pub fn col(&self) -> i32 {
        match *self {
            Deco::Box { col, .. }
            | Deco::Ring { col, .. }
            | Deco::Line { col, .. }
            | Deco::Mark { col, .. }
            | Deco::Tone { col, .. }
            | Deco::Gap { col, .. } => col,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub options: Options,
    /// 共用了多少行（版面上的列）。
    pub columns: i32,
    pub glyphs: Vec<Glyph>,
    pub decos: Vec<Deco>,
}

/// 有眉批而天頭没指定时，天頭加高到这么多字。
const HEAD_FOR_NOTES: f32 = 3.0;

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
    doc.blocks.iter().any(|b| match b {
        Block::Heading { content, .. } => any(content),
        Block::Paragraph { lines, .. } => lines.iter().any(|l| any(&l.content)),
        _ => false,
    })
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
