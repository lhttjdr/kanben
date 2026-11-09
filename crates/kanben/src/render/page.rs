//! 版面几何：行号 → 页面上的位置。所有长度以「字」为单位（正文一字 30px 時，1px = 1/30 字）。
//!
//! 兩種版式都照原版排版器：
//! - 長卷：一張長紙，版框（#gaeg）圍住全部行，另加一行空行；
//! - 冊頁：每半葉一頁（原版 切䈎），版心只露一半，與鄰頁拼起来才是整個版心。

use crate::layout::{Format, Options};

/// 一像素（正文一字 30px 時）。
pub const PX1: f32 = 1.0 / 30.0;
/// 版框外的留白。
pub const MARGIN: f32 = 1.0;
/// 冊頁版框下的留白。
const LEAF_FOOT: f32 = 1.65;

#[derive(Clone, Copy, Debug)]
pub struct Geometry {
    pub format: Format,
    /// 每半葉行數。
    pub columns: i32,
    pub cells: i32,
    pub col_w: f32,
    pub head: f32,
    /// 长卷的总行数（只有長卷用）。
    pub scroll_cols: i32,
    /// 長卷紙上要容下的行數（眉批可能伸到版框左邊外）。
    pub scroll_extent: i32,
}

impl Geometry {
    pub fn new(o: &Options) -> Self {
        Geometry {
            format: o.format,
            columns: o.columns,
            cells: o.cells,
            col_w: o.column_width,
            head: o.head,
            scroll_cols: 0,
            scroll_extent: 0,
        }
    }

    /// 字頂。版框外綫在字頂上 15px，内綫在 10px。
    pub fn text_top(&self) -> f32 {
        MARGIN + self.head + 0.3
    }

    pub fn height(&self) -> f32 {
        let end = self.text_top() + self.cells as f32;
        match self.format {
            // 長卷：版框下還有行號（原版字底下 1.4 字以内）。
            Format::Scroll => end + 1.6,
            Format::Leaves => end + 13.0 * PX1 + LEAF_FOOT,
        }
    }

    /// 冊頁共幾頁（半葉數，補成雙數）。
    pub fn leaf_pages(&self, total_cols: i32) -> i32 {
        let n = (total_cols.max(1) + self.columns - 1).div_euclid(self.columns);
        n + n % 2
    }

    /// 冊頁第 `page` 頁：最左一行的左綫離紙邊多遠。
    /// 單頁（右半葉）的版心在左，紙邊切在版心中間；雙頁版心在右，左有外框。
    fn leaf_left(&self, page: i32) -> f32 {
        if page % 2 == 0 {
            23.0 * PX1
        } else {
            34.0 * PX1
        }
    }

    /// 第一行（最右一行）的右綫。
    pub fn first_rule(&self, page: i32) -> f32 {
        match self.format {
            Format::Scroll => self.width(page) - MARGIN,
            Format::Leaves => self.leaf_left(page) + self.columns as f32 * self.col_w,
        }
    }

    pub fn width(&self, page: i32) -> f32 {
        match self.format {
            Format::Scroll => {
                2.0 * MARGIN + self.scroll_cols.max(self.scroll_extent).max(1) as f32 * self.col_w
            }
            Format::Leaves => {
                // 單頁右邊：内綫 1px、内距 3px、外框 2px、紙邊 30px；雙頁右邊露半個版心（24px）。
                let right = if page % 2 == 0 { 36.0 } else { 24.0 };
                self.first_rule(page) + right * PX1
            }
        }
    }

    /// 第 `col` 行在哪一页、中线在哪里。
    pub fn locate(&self, col: i32) -> (i32, f32) {
        let (page, j) = match self.format {
            Format::Scroll => (0, col),
            Format::Leaves => (col.div_euclid(self.columns), col.rem_euclid(self.columns)),
        };
        (page, self.first_rule(page) - (j as f32 + 0.5) * self.col_w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves(columns: i32) -> Geometry {
        Geometry::new(&Options {
            format: Format::Leaves,
            columns,
            column_width: 1.6,
            ..Options::default()
        })
    }

    #[test]
    fn half_leaves_match_original_panels() {
        let g = leaves(9);
        // 原版 切䈎(9)：單頁 491px、雙頁 490px（一字 30px）。
        assert!((g.width(0) * 30.0 - 491.0).abs() < 1e-3);
        assert!((g.width(1) * 30.0 - 490.0).abs() < 1e-3);
        // 半葉數補成雙數。
        assert_eq!(g.leaf_pages(10), 2);
        assert_eq!(g.leaf_pages(19), 4);
        // 第十行在第二頁的第一行。
        assert_eq!(g.locate(9).0, 1);
    }
}
