//! 葉碼用的中文数字。

const DIGITS: [char; 10] = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];

/// 1 → 一，12 → 十二，105 → 一百〇五，110 → 一百一十。
pub fn hanzi(n: u32) -> String {
    if n == 0 {
        return "〇".into();
    }
    if n >= 10_000 {
        return n
            .to_string()
            .chars()
            .map(|c| DIGITS[c.to_digit(10).unwrap_or(0) as usize])
            .collect();
    }
    let parts = [
        (n / 1000, '千'),
        (n / 100 % 10, '百'),
        (n / 10 % 10, '十'),
        (n % 10, ' '),
    ];
    let mut out = String::new();
    let mut zero = false;
    for (i, (d, unit)) in parts.into_iter().enumerate() {
        if d == 0 {
            zero = !out.is_empty();
            continue;
        }
        if zero {
            out.push('〇');
            zero = false;
        }
        // 「十二」而不是「一十二」
        if !(unit == '十' && d == 1 && out.is_empty()) {
            out.push(DIGITS[d as usize]);
        }
        if i < 3 {
            out.push(unit);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::hanzi;

    #[test]
    fn numbers() {
        assert_eq!(hanzi(1), "一");
        assert_eq!(hanzi(10), "十");
        assert_eq!(hanzi(12), "十二");
        assert_eq!(hanzi(20), "二十");
        assert_eq!(hanzi(105), "一百〇五");
        assert_eq!(hanzi(110), "一百一十");
        assert_eq!(hanzi(1001), "一千〇一");
    }
}
