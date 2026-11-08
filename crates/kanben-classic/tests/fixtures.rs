//! 用原版實錄的輸入輸出（tools/classic/record.mjs 生成）逐個檢驗移植的函數。

use std::collections::BTreeMap;
use std::fs;

use kanben_classic::marks;
use kanben_classic::text::{self, Env};
use serde_json::Value;

fn env_from(g: &Value) -> Env {
    let t = &g["調距"];
    let f = |v: &Value| v.as_f64().unwrap();
    Env {
        正文: f(&t["正文"]),
        夾注: f(&t["夾注"]),
        注中注: f(&t["注中注"]),
        注中小字: f(&t["注中小字"]),
        敘: f(&t["敘"]),
        跋: f(&t["跋"]),
        行尾空白長: f(&g["行尾空白長"]),
        系統: g["系統"].as_str().unwrap().to_string(),
    }
}

fn s(v: &Value) -> &str {
    v.as_str().unwrap_or_else(|| panic!("not a string: {v}"))
}

/// 回放一次調用；返回 None 表示該函數尚未移植或不屬純文本。
fn replay(call: &Value, env: &Env) -> Option<(String, String)> {
    let a = &call["args"];
    let ret = |v: String| Some((v, s(&call["ret"]).to_string()));
    match s(&call["fn"]) {
        "寫眉批" => {
            let (body, notes) = text::寫眉批(s(&a[0]));
            let got = format!("{body}\n----\n{}", text::天頭(&notes));
            Some((
                got,
                format!("{}\n----\n{}", s(&call["ret"]), s(&call["tentou"])),
            ))
        }
        "toHTML" => ret(text::to_html(s(&a[0]), env)),
        "數碼" => ret(text::數碼(s(&a[0]))),
        "畫表格" => ret(text::畫表格(s(&a[0]))),
        "分段" => ret(text::分段(s(&a[0]))),
        "分節" => ret(text::分節(s(&a[0]))),
        "引詩文" => ret(text::引詩文(s(&a[0]))),
        "分段單兒" => ret(text::分段單兒(s(&a[0]))),
        "插圖" => ret(text::插圖(s(&a[0]))),
        "尾注" => ret(text::尾注(s(&a[0]))),
        "特殊段" => ret(text::特殊段(s(&a[0]), env)),
        "渲染段" => ret(text::渲染段(s(&a[0]), env)),
        "兼行" => ret(text::兼行(s(&a[0]))),
        "旁字" => ret(text::旁字(s(&a[0]))),
        "提取旁句" => ret(text::提取旁句(s(&a[0]))),
        "字樣" => ret(text::字樣(s(&a[0]), a[1].as_str().unwrap_or("圖"))),
        "縱中橫碼" => ret(text::縱中橫碼(s(&a[0]))),
        "構字" => ret(text::構字(s(&a[0]))),
        "點字下" => ret(marks::點字下(s(&a[0]), env)),
        "陰文和墨圍" => ret(marks::陰文和墨圍(s(&a[0]))),
        "圈點抹" => ret(marks::圈點抹(s(&a[0]))),
        "點行間" => {
            let key = if s(&a[0]) == "tentou" {
                "tentou"
            } else {
                "main"
            };
            Some((
                marks::點行間(s(&call["before"][key])),
                s(&call["after"][key]).to_string(),
            ))
        }
        "行間標點" => {
            let key = if s(&a[0]) == "tentou" {
                "tentou"
            } else {
                "main"
            };
            Some((
                marks::行間標點(s(&call["before"][key]), a[1].as_str()),
                s(&call["after"][key]).to_string(),
            ))
        }
        "punctuateVH" if a[1].as_bool() == Some(true) => {
            let (out, _) = text::punctuate_vh_1(s(&a[0]), s(&a[2]));
            ret(out)
        }
        _ => None,
    }
}

fn first_diff(a: &str, b: &str) -> String {
    let i = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
    let ctx = |s: &str| {
        s.chars()
            .skip(i.saturating_sub(30))
            .take(90)
            .collect::<String>()
    };
    format!("在第 {i} 字\n  移植：{}\n  原版：{}", ctx(a), ctx(b))
}

#[test]
fn pure_functions_match_original() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let globals: Value =
        serde_json::from_str(&fs::read_to_string(format!("{dir}/_globals.json")).unwrap()).unwrap();
    let env = env_from(&globals);
    let mut stats: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut failures = Vec::new();
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with('_') {
            continue;
        }
        let fx: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        for call in fx["calls"].as_array().unwrap() {
            let Some((got, want)) = replay(call, &env) else {
                continue;
            };
            let e = stats
                .entry(call["fn"].as_str().unwrap().to_string())
                .or_default();
            if got == want {
                e.0 += 1;
            } else {
                e.1 += 1;
                if failures.len() < 12 {
                    failures.push(format!(
                        "[{name}] {}：{}",
                        call["fn"],
                        first_diff(&got, &want)
                    ));
                }
            }
        }
    }
    for (f, (ok, bad)) in &stats {
        eprintln!("{f:10} 通過 {ok:4}  不符 {bad}");
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n\n"));
}
