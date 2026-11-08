//! 源碼裏每條 `re!(…)` 正則都要能編譯（正則是用到時才編譯的，有些分支測試走不到）。

use std::fs;

use kanben_classic::jsre::Re;

/// 從源碼裏取出 `re!(r"…", "旗標")` 的正則源碼與旗標。
fn patterns(src: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(i) = rest.find("re!(") {
        rest = &rest[i + 4..];
        let (pat, after) = match lit(rest.trim_start()) {
            Some(x) => x,
            None => continue,
        };
        let after = after.trim_start();
        let flags = after
            .strip_prefix(',')
            .and_then(|a| lit(a.trim_start()))
            .map(|(f, _)| f)
            .unwrap_or_default();
        out.push((pat, flags));
    }
    out
}

/// 讀一個字符串字面量（`"…"`、`r"…"`、`r#"…"#`），返回內容與其後的文本。
fn lit(s: &str) -> Option<(String, &str)> {
    if let Some(r) = s.strip_prefix("r#\"") {
        let end = r.find("\"#")?;
        return Some((r[..end].to_string(), &r[end + 2..]));
    }
    if let Some(r) = s.strip_prefix("r\"") {
        let end = r.find('"')?;
        return Some((r[..end].to_string(), &r[end + 1..]));
    }
    let r = s.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = r.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((out, &r[i + 1..])),
            '\\' => {
                let (_, n) = chars.next()?;
                match n {
                    'u' => {
                        let rest: String = chars
                            .by_ref()
                            .map(|(_, c)| c)
                            .take_while(|&c| c != '}')
                            .collect();
                        let hex = rest.trim_start_matches('{');
                        out.push(char::from_u32(u32::from_str_radix(hex, 16).ok()?)?);
                    }
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    other => out.push(other),
                }
            }
            c => out.push(c),
        }
    }
    None
}

#[test]
fn every_regex_compiles() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let mut n = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let src = fs::read_to_string(&path).unwrap();
        for (pat, flags) in patterns(&src) {
            if pat.contains('{') && pat.contains("{}") {
                continue; // format! 生成的，不是字面正則
            }
            let r = std::panic::catch_unwind(|| Re::new(&pat, &flags));
            assert!(r.is_ok(), "{}: /{pat}/{flags} 編譯失敗", path.display());
            n += 1;
        }
    }
    assert!(n > 150, "只找到 {n} 條正則，提取可能有誤");
    eprintln!("{n} 條正則都能編譯");
}
