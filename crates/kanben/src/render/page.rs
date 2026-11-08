//! 版面几何：行号 → 页面上的位置。所有长度以「字」为单位。

use crate::layout::{Format, Options};

/// 版框外的留白。
pub const MARGIN: f32 = 1.0;
/// 版框到文字的距离。
pub const FRAME_PAD: f32 = 0.3;
/// 地腳。
pub const FOOT: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct Geometry {
    pub format: Format,
    /// 每半葉行數。
    pub columns: i32,
    pub cells: i32,
    pub col_w: f32,
    pub head: f32,
    /// 版心宽。
    pub banxin_w: f32,
    /// 长卷的总行数（只有長卷用）。
    pub scroll_cols: i32,
}

impl Geometry {
    pub fn new(o: &Options) -> Self {
        Geometry {
            format: o.format,
            columns: o.columns,
            cells: o.cells,
            col_w: o.column_width,
            head: o.head,
            banxin_w: o.column_width * 1.3,
            scroll_cols: 0,
        }
    }

    pub fn per_leaf(&self) -> i32 {
        self.columns * 2
    }

    pub fn frame_top(&self) -> f32 {
        MARGIN + self.head
    }

    pub fn text_top(&self) -> f32 {
        self.frame_top() + FRAME_PAD
    }

    pub fn frame_bottom(&self) -> f32 {
        self.text_top() + self.cells as f32 + FRAME_PAD
    }

    pub fn height(&self) -> f32 {
        self.frame_bottom() + FOOT
    }

    pub fn width(&self) -> f32 {
        match self.format {
            Format::Leaves => 2.0 * MARGIN + 2.0 * self.columns as f32 * self.col_w + self.banxin_w,
            Format::Scroll => 2.0 * MARGIN + self.scroll_cols.max(1) as f32 * self.col_w,
        }
    }

    /// 版心左右边界。
    pub fn banxin(&self) -> (f32, f32) {
        let left = MARGIN + self.columns as f32 * self.col_w;
        (left, left + self.banxin_w)
    }

    /// 第 `col` 行在哪一页、中线在哪里。
    pub fn locate(&self, col: i32) -> (i32, f32) {
        match self.format {
            Format::Scroll => (0, self.width() - MARGIN - (col as f32 + 0.5) * self.col_w),
            Format::Leaves => {
                let per = self.per_leaf();
                let page = col.div_euclid(per);
                let local = col.rem_euclid(per);
                let j = (local % self.columns) as f32;
                let x = if local < self.columns {
                    self.width() - MARGIN - (j + 0.5) * self.col_w
                } else {
                    self.banxin().0 - (j + 0.5) * self.col_w
                };
                (page, x)
            }
        }
    }
}
