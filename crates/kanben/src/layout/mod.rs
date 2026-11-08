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

use crate::syntax::ast::{Color, Document, EmphMark, Shape};
use crate::syntax::span::Diagnostic;
pub use options::{Format, Options, PunctStyle, TitleMark};

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
    },
    /// 專名號、書名號、抹、刪除綫。
    Line {
        col: i32,
        x: f32,
        y0: f32,
        y1: f32,
        wavy: bool,
        color: Option<Color>,
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
}

impl Deco {
    pub fn col(&self) -> i32 {
        match *self {
            Deco::Box { col, .. }
            | Deco::Ring { col, .. }
            | Deco::Line { col, .. }
            | Deco::Mark { col, .. }
            | Deco::Tone { col, .. } => col,
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

pub fn layout(doc: &Document, options: Options, diags: &mut Vec<Diagnostic>) -> Layout {
    engine::Engine::new(options, diags).run(doc)
}
