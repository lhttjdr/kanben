//! 命令行：`kanben 輸入.kb [-o 輸出.html]`
//!
//! 输出 HTML（全部页面）；`--svg 目录` 则每页另存一张 SVG；`--font-loader 網址` 让 HTML 加载网页字體。

use std::path::PathBuf;
use std::process::ExitCode;

use kanben::syntax::span::{LineIndex, Severity};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut input = None;
    let mut out_html = None;
    let mut svg_dir = None;
    let mut font_loader = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "-o" => out_html = args.next().map(PathBuf::from),
            "--svg" => svg_dir = args.next().map(PathBuf::from),
            "--font-loader" => font_loader = args.next(),
            "-h" | "--help" => {
                eprintln!("用法：kanben 輸入.kb [-o 輸出.html] [--svg 目錄] [--font-loader 網址]");
                return ExitCode::SUCCESS;
            }
            _ => input = Some(PathBuf::from(a)),
        }
    }
    let Some(input) = input else {
        eprintln!("用法：kanben 輸入.kb [-o 輸出.html] [--svg 目錄] [--font-loader 網址]");
        return ExitCode::FAILURE;
    };
    let src = match std::fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("讀不了 {}：{e}", input.display());
            return ExitCode::FAILURE;
        }
    };

    let out = kanben::typeset(&src, &[]);
    let index = LineIndex::new(&src);
    let mut errors = 0;
    for d in &out.diagnostics {
        let (line, col) = index.line_col(d.span.start);
        let level = match d.severity {
            Severity::Error => {
                errors += 1;
                "錯誤"
            }
            Severity::Warning => "提示",
        };
        eprintln!("{}:{line}:{col}: {level}：{}", input.display(), d.message);
    }

    let html_path = out_html.unwrap_or_else(|| input.with_extension("html"));
    let html = kanben::render::html(&out.options.title, &out.pages, font_loader.as_deref());
    if let Err(e) = std::fs::write(&html_path, html) {
        eprintln!("寫不了 {}：{e}", html_path.display());
        return ExitCode::FAILURE;
    }
    if let Some(dir) = svg_dir {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("建不了目錄 {}：{e}", dir.display());
            return ExitCode::FAILURE;
        }
        for (i, p) in out.pages.iter().enumerate() {
            let path = dir.join(format!("{:03}.svg", i + 1));
            if let Err(e) = std::fs::write(&path, kanben::render::standalone(p)) {
                eprintln!("寫不了 {}：{e}", path.display());
                return ExitCode::FAILURE;
            }
        }
    }
    eprintln!("{} 頁 → {}", out.pages.len(), html_path.display());
    if errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
