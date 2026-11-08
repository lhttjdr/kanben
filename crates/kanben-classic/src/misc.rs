//! 原版 main.js 裏不碰頁面的計算：讀數、析數、算籌樣式、辨認瀏覽器與系統。

use crate::jsre::u16len;
use crate::re;
use crate::text::num;

const 字: [&str; 10] = ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"];

/// `數.讀十萬以內`
pub fn 讀十萬以內(mut an: u64) -> String {
    const 級名: [&str; 5] = ["", "十", "百", "千", "萬"];
    let mut 讀法 = String::new();
    let mut 位 = 0;
    while an > 0 {
        let d = (an % 10) as usize;
        if d != 0 {
            let 數字 = if 位 == 1 && d == 1 { "" } else { 字[d] };
            讀法 = format!("{數字}{}{讀法}", 級名.get(位).copied().unwrap_or(""));
        }
        an /= 10;
        位 += 1;
    }
    讀法
}

/// `析整數`：漢字數（可帶 負、前、歬、正）→ 整數。照原版的寫法，包括它的局限。
pub fn 析整數(讀法: &str) -> f64 {
    let first = 讀法.chars().next().map(String::from).unwrap_or_default();
    let 正負 = if ["負", "前", "歬"].contains(&first.as_str()) {
        -1.0
    } else {
        1.0
    };
    let mut s = re!("^[負前歬正]", "g").replace(讀法, "");
    s = re!("亖", "g").replace(&s, "四");
    s = re!("[廿卄]", "g").replace(&s, "二十");
    s = re!("[卅丗]", "g").replace(&s, "三十");
    s = re!("卌", "g").replace(&s, "四十");
    s = re!("皕", "g").replace(&s, "二百");
    s = re!("元", "g").replace(&s, "一");
    const 十進級: [&str; 3] = ["十", "百", "千"];
    const 大數級: [&str; 11] = [
        "萬", "億", "兆", "京", "垓", "秭", "穰", "溝", "澗", "正", "載",
    ];
    // 原版按 UTF-16 單元逐個取字（slice(i, i+1)）；這裏的字都在 BMP。
    let cs: Vec<String> = s.chars().map(String::from).collect();
    let mut 值 = 0.0;
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i].as_str();
        let 位 = 字.iter().position(|&d| d == c);
        match 位 {
            Some(p) if p > 0 => {
                if i == cs.len() - 1 {
                    值 += p as f64;
                } else {
                    let next = cs[i + 1].as_str();
                    let 級 = 十進級.iter().position(|&d| d == next).map_or(0, |k| k + 1);
                    值 += p as f64 * 10f64.powi(級 as i32);
                    if !大數級.contains(&next) {
                        i += 1;
                    }
                }
            }
            _ => {
                if let Some(k) = 十進級.iter().position(|&d| d == c) {
                    值 += 10f64.powi(k as i32 + 1);
                } else if 大數級.contains(&c) && 值 != 0.0 {
                    值 *= 10000.0;
                }
            }
        }
        i += 1;
    }
    正負 * 值
}

/// `整數籌` 的結果。
#[derive(Clone, Debug, PartialEq)]
pub struct 籌 {
    /// 數字（算籌碼子字串或十進制數字串）。
    pub 數: String,
    pub 類: String,
    pub 寬窄: f64,
    pub 負號: String,
}

/// `整數籌`，輸入是數值（如行號）。原版此時把負號記成 `＼`，照錄。
pub fn 整數籌_數(n: f64, 个位縱: bool) -> 籌 {
    let 數 = if n.is_nan() {
        "0".to_string()
    } else {
        num(n.abs())
    };
    let 負號 = if n.is_nan() {
        String::new()
    } else {
        "＼".to_string()
    };
    籌樣式(數, 个位縱, 負號)
}

/// `整數籌`，輸入是字串（碼子字或數字）。
pub fn 整數籌_字(s: &str, 个位縱: bool) -> 籌 {
    if re!("[〇〡-〩]").test(s) {
        let (數, 負號) = match s.strip_prefix('＼') {
            Some(rest) => (rest.to_string(), "＼".to_string()),
            None => (s.to_string(), String::new()),
        };
        籌樣式(數, 个位縱, 負號)
    } else {
        整數籌_數(s.trim().parse::<f64>().unwrap_or(f64::NAN), 个位縱)
    }
}

fn 籌樣式(數: String, 个位縱: bool, 負號: String) -> 籌 {
    let 位數 = u16len(&數);
    let mut 類 = ["cnumDrV", "cnumDrH"][(usize::from(个位縱) + 位數) % 2].to_string();
    if 位數 == 1 {
        類 += " num1d";
    }
    let 寬窄 = if 位數 > 1 {
        120.0 / 位數 as f64
    } else {
        80.0
    };
    籌 {
        數, 類, 寬窄, 負號
    }
}

/// `getBrowserInfo`：由 userAgent 認瀏覽器，返回（名, 版本）。
pub fn 瀏覽器(ua: &str) -> (String, String) {
    let agent = ua.to_lowercase();
    let ver = |re: &str| -> String {
        // 原版帶 i 旗標；agent 已轉小寫，模式也都是小寫，不用 i（省去 Unicode 大小寫表）。
        crate::jsre::Re::new(re, "g")
            .match_all(&agent)
            .map(|v| v.join(","))
            .map(|s| re!(r"[^0-9.]", "g").replace(&s, ""))
            .unwrap_or_default()
    };
    let has = |s: &str| agent.find(s).is_some_and(|i| i > 0);
    if has("msie") {
        ("IE".into(), ver(r"msie [\d.]+;"))
    } else if has(".net") {
        ("IE".into(), ver(r"rv\:[\d.]+"))
    } else if agent.contains("micromessenger") {
        ("微信".into(), "未知".into())
    } else if re!(r"\sqq").test(&agent) {
        ("QQ".into(), "未知".into())
    } else if has("firefox") {
        ("火狐".into(), ver(r"firefox\/[\d.]+"))
    } else if has("chrome") {
        ("鉻".into(), ver(r"chrome\/[\d.]+"))
    } else if has("safari") && !agent.contains("chrome") {
        ("Safari".into(), ver(r"safari\/[\d.]+"))
    } else {
        ("未知".into(), "未知".into())
    }
}

/// `detectOS`
pub fn 系統(ua: &str, platform: &str) -> &'static str {
    // 原版用 /…/i；這裏把兩邊都轉成小寫再比，結果相同。
    let (ua_l, pf_l) = (ua.to_lowercase(), platform.to_lowercase());
    let ua_has = |p: &str| ua_l.contains(&p.to_lowercase());
    let pf_has = |p: &str| pf_l.contains(&p.to_lowercase());
    if ua_has("OpenHarmony") {
        "鴻蒙"
    } else if ua_has("Android") || platform == "Linux armv7l" {
        "Android"
    } else if ua_has("iPhone") {
        "iPhone"
    } else if ua_has("iPad") || pf_has("iPad") {
        "iPad"
    } else if ua_has("Linux") || pf_has("Linux") {
        "Linux"
    } else if ua_has("Windows") || pf_has("Win") {
        "Windows"
    } else {
        "其他"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readings() {
        assert_eq!(讀十萬以內(105), "一百五");
        assert_eq!(讀十萬以內(12), "十二");
        assert_eq!(讀十萬以內(20), "二十");
        assert_eq!(析整數("三百廿一"), 321.0);
        assert_eq!(析整數("負十五"), -15.0);
    }

    #[test]
    fn rods() {
        let r = 整數籌_數(12.0, false);
        assert_eq!(
            (r.數.as_str(), r.類.as_str(), r.寬窄),
            ("12", "cnumDrV", 60.0)
        );
        assert_eq!(整數籌_數(3.0, false).類, "cnumDrH num1d");
        assert_eq!(整數籌_字("＼〡〢", true).負號, "＼");
    }

    #[test]
    fn browsers() {
        let chrome = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36";
        assert_eq!(瀏覽器(chrome), ("鉻".into(), "146.0.0.0".into()));
        assert_eq!(系統(chrome, "Linux x86_64"), "Linux");
    }
}
