//! 版式选项。来自 `\setup[键=值]`、`\book{}` 等，也可以由调用方覆盖。

use crate::syntax::ast::{Color, MetaEntry};
use crate::syntax::span::Diagnostic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// 冊頁：按半葉切分，每葉兩個半葉，中間是版心。
    Leaves,
    /// 長卷：不分葉。
    Scroll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunctStyle {
    /// 句讀：標點不佔格，小字附在字的右下。
    Judou,
    /// 新式：標點佔一格，用直排字形。
    Grid,
    /// 白文：去掉標點。
    Bare,
}

/// 正文里的書名號 `《》` `〈〉` 怎么画。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleMark {
    /// 左側波綫，括號不显示（复制时仍在）。
    Wavy,
    /// 保留括號，当作標點排。
    Bracket,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    /// 每半葉行數。
    pub columns: i32,
    /// 每行字數。
    pub cells: i32,
    pub format: Format,
    pub punct: PunctStyle,
    pub punct_color: Color,
    pub title_mark: TitleMark,
    /// 書名，印在版心。
    pub title: String,
    /// 卷名，印在版心。
    pub volume: String,
    /// 版心下方的堂號。
    pub studio: String,
    /// 行寬，以字為單位。
    pub column_width: f32,
    /// 天頭高度，以字為單位（眉批排在這裏）。
    pub head: f32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            columns: 10,
            cells: 20,
            format: Format::Leaves,
            punct: PunctStyle::Judou,
            punct_color: Color::Red,
            title_mark: TitleMark::Wavy,
            title: String::new(),
            volume: String::new(),
            studio: String::new(),
            column_width: 1.5,
            head: 3.0,
        }
    }
}

/// `\setup` 可用的键，用于报错提示。
pub const KEYS: &str = "columns, cells, format, punct, punct-color, title-mark, column-width, head";

impl Options {
    pub fn from_meta(meta: &[MetaEntry], diags: &mut Vec<Diagnostic>) -> Options {
        let mut o = Options::default();
        for m in meta {
            if let Err(e) = o.set(&m.key, &m.value) {
                diags.push(Diagnostic::error(m.span, e));
            }
        }
        o
    }

    /// 设置一项。出错时返回说明。
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        let value = value.trim();
        let int = |lo: i32, hi: i32| -> Result<i32, String> {
            let n: i32 = value
                .parse()
                .map_err(|_| format!("{key} 應是整數，而不是「{value}」"))?;
            if (lo..=hi).contains(&n) {
                Ok(n)
            } else {
                Err(format!("{key} 應在 {lo} 到 {hi} 之間"))
            }
        };
        let float = |lo: f32, hi: f32| -> Result<f32, String> {
            let n: f32 = value
                .parse()
                .map_err(|_| format!("{key} 應是數字，而不是「{value}」"))?;
            if (lo..=hi).contains(&n) {
                Ok(n)
            } else {
                Err(format!("{key} 應在 {lo} 到 {hi} 之間"))
            }
        };
        let choice = |allowed: &str| format!("{key} 只能是 {allowed}，而不是「{value}」");
        match key.trim() {
            "book" => self.title = value.into(),
            "volume" => self.volume = value.into(),
            "studio" => self.studio = value.into(),
            "columns" => self.columns = int(1, 40)?,
            "cells" => self.cells = int(4, 60)?,
            "format" => {
                self.format = match value {
                    "leaves" => Format::Leaves,
                    "scroll" => Format::Scroll,
                    _ => return Err(choice("leaves, scroll")),
                }
            }
            "punct" => {
                self.punct = match value {
                    "judou" => PunctStyle::Judou,
                    "grid" => PunctStyle::Grid,
                    "bare" => PunctStyle::Bare,
                    _ => return Err(choice("judou, grid, bare")),
                }
            }
            "punct-color" => {
                self.punct_color = Color::from_name(value).ok_or_else(|| choice(Color::NAMES))?
            }
            "title-mark" => {
                self.title_mark = match value {
                    "wavy" => TitleMark::Wavy,
                    "bracket" => TitleMark::Bracket,
                    _ => return Err(choice("wavy, bracket")),
                }
            }
            "column-width" => self.column_width = float(1.0, 3.0)?,
            "head" => self.head = float(0.0, 10.0)?,
            k => return Err(format!("未知的設定 {k}（可用：{KEYS}）")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_values() {
        let mut o = Options::default();
        o.set("columns", "9").unwrap();
        o.set("format", "scroll").unwrap();
        o.set("punct", "bare").unwrap();
        o.set("title-mark", "bracket").unwrap();
        assert_eq!(o.columns, 9);
        assert_eq!(o.format, Format::Scroll);
        assert_eq!(o.punct, PunctStyle::Bare);
        assert_eq!(o.title_mark, TitleMark::Bracket);
        assert!(o.set("cells", "100").is_err());
        assert!(o.set("cells", "九").is_err());
        assert!(o.set("colour", "red").is_err());
    }
}
