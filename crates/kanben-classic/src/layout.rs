//! 需要量頁面的幾步裏的計算部分：頁面量出位置，這裏算出要設的類名、尺寸與裁切形狀。
//! 算式、數值格式都照原版（dlNote.js 的 雙行夾注、調整雙行夾注）。

use crate::jsre::u16len;
use crate::re;
use crate::text::num;

/// `雙行夾注` 裏的字數：去掉直排標點、標記符、腳注號與標籤後的 UTF-16 長度。
pub fn 夾注字數(inner: &str) -> usize {
    let 乙 = re!(
        r"[︐︑︒︓︔︖︕︽︾︿﹀･・﹁﹂﹃﹄︻︼\u{E794}\u{E795}︵︶﹇﹈¯ˇˋˆ︳﹤﹥\(\)\[\]\{\}︹︺〘〙]",
        "g"
    )
    .replace(inner, "");
    let 乙 = re!(r#"<small class="(pCmtN|gCmtN).+?<\/small>"#, "g").replace(&乙, "");
    let 乙 = re!(r"<.+?>", "g").replace(&乙, "");
    u16len(&乙)
}

/// 一條雙行夾注的排法。
#[derive(Clone, Debug, PartialEq)]
pub enum 夾注排法 {
    /// 兩字以內：不分行，`ig_gd`。
    不分,
    /// 本行排得下：`sht_gd`，高 `height`。
    本行 { height: String },
    /// 要跨到後面的行：`long_gd`。
    跨行 {
        width: String,
        /// 夾注本身的 `clip-path` 與 `shape-outside`。
        clip: String,
        /// 頭部佔位的高。
        head_height: String,
        /// 尾部浮動塊的 `clip-path` 與 `shape-outside`。
        tail_clip: String,
        /// 注後佔位的高（夾注後面還有內容時才用）。
        after_height: String,
    },
}

/// 原版 `雙行夾注` 的算式。`丙` 字數，`丁` 起始位置（第幾格，0 起），`壬` 段首縮進格數，`每行格數`。
pub fn 夾注排法(丙: usize, 丁: i64, 壬: i64, 每行格數: i64) -> 夾注排法 {
    if 丙 <= 2 {
        return 夾注排法::不分;
    }
    let 半 = 丙.div_ceil(2) as i64;
    let 可用 = 每行格數 - if 丁 >= 每行格數 { 0 } else { 丁 };
    if 丙 as i64 <= 可用 * 2 {
        return 夾注排法::本行 {
            height: format!("{}em", num(1.25 * 半 as f64 + 0.125)),
        };
    }
    長夾注(半, if 丁 == 0 { 每行格數 } else { 丁 }, 壬, 每行格數)
}

/// 原版 `調整雙行夾注`（改窗口重排時）：已知佔幾大格，判斷改成本行或跨行。
/// 返回 None 表示不用動。
pub fn 調整排法(
    佔大格數: i64,
    丁: i64,
    壬: i64,
    每行格數: i64,
    現為跨行: bool,
) -> Option<夾注排法> {
    let 可用 = 每行格數 - if 丁 >= 每行格數 { 0 } else { 丁 };
    if 佔大格數 <= 可用 && 現為跨行 {
        Some(夾注排法::本行 {
            height: format!("{}em", num(1.25 * 佔大格數 as f64 + 0.125)),
        })
    } else if 佔大格數 > 可用 {
        Some(長夾注(
            佔大格數,
            if 丁 == 0 { 每行格數 } else { 丁 },
            壬,
            每行格數,
        ))
    } else {
        None
    }
}

fn 長夾注(半: i64, 丁: i64, 壬: i64, 每行格數: i64) -> 夾注排法 {
    let 辛 = 半 - (每行格數 - 丁);
    let 癸 = 辛 as f64 / (每行格數 - 壬) as f64;
    let 焉 = 癸.floor() as i64;
    let 哉 = 癸.ceil() as i64 + if 每行格數 < 丁 { 0 } else { 1 };
    let mut 乎 = 辛 - 焉 * (每行格數 - 壬);
    if 乎 == 0 {
        乎 = 每行格數 - 壬;
    }
    let 庚 = format!("{}em", num((丁 - 壬) as f64 * 1.25));
    let 乎f = 乎 as f64;
    let 也 = format!(
        "polygon(0 {a}em,calc(var(--跨註行寬)*1.9) {a}em,calc(var(--跨註行寬)*1.9) 100%,0 100%)",
        a = num(乎f * 1.25 + 0.15)
    );
    let 於 = format!(
        "polygon(0 0,0 {b}em,calc(var(--跨註行寬)*2) {b}em,calc(var(--跨註行寬)*2) 100%,calc(var(--跨註行寬)*{c}) 100%,calc(var(--跨註行寬)*{c}) {庚},calc(var(--跨註行寬)*{d}) {庚},calc(var(--跨註行寬)*{d}) 0)",
        b = num(乎f * 1.25 + 0.1),
        c = 哉 * 2,
        d = 哉 * 2 - 2
    );
    夾注排法::跨行 {
        width: format!("calc(var(--跨註行寬)*{})", 哉 * 2),
        clip: 於,
        head_height: 庚,
        tail_clip: 也,
        after_height: format!("{乎}em"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_like_original() {
        assert_eq!(夾注字數("分︐扶問翻︒"), 4);
        assert_eq!(夾注字數(r#"甲<span class="x">乙</span>"#), 2);
    }

    #[test]
    fn short_and_long() {
        assert_eq!(夾注排法(2, 0, 0, 22), 夾注排法::不分);
        assert_eq!(
            夾注排法(7, 3, 0, 22),
            夾注排法::本行 {
                height: "5.125em".into()
            }
        );
        let 夾注排法::跨行 {
            width,
            after_height,
            ..
        } = 夾注排法(30, 20, 0, 22)
        else {
            panic!()
        };
        assert_eq!(width, "calc(var(--跨註行寬)*4)");
        assert_eq!(after_height, "13em");
    }
}
