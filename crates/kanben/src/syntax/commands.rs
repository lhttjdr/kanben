//! 命令表：把通用的 `\名[參數]{內容}` 解释成具体含义。
//!
//! 解析器只认形式，不认命令；所有命令名、参数都在这里。

use super::ast::{Color, ElementKind, EmphMark, Shape};
use super::span::Span;

/// 方括号里的一个参数：`round` 或 `raise=2`。
#[derive(Clone, Debug, PartialEq)]
pub struct Arg {
    pub key: Option<String>,
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// 有内容的行内命令：`\name{…}`。
    Inline,
    /// 只能写在行首的块级命令：`\heading` `\setup` `\indent` …
    Block,
}

pub fn kind_of(name: &str) -> Option<Kind> {
    Some(match name {
        "note" | "side" | "comment" | "head" | "name" | "title" | "emph" | "stroke" | "inv"
        | "frame" | "del" | "ins" | "color" | "compose" => Kind::Inline,
        "setup" | "book" | "volume" | "studio" | "heading" | "begin" | "end" | "indent"
        | "raise" | "bottom" => Kind::Block,
        _ => return None,
    })
}

pub const INLINE_COMMANDS: &str =
    "note, side, comment, head, name, title, emph, stroke, inv, frame, del, ins, color, compose";

pub struct Resolved {
    pub kind: ElementKind,
    pub warnings: Vec<(Span, String)>,
}

/// 解释行内命令。调用前已确认 `kind_of(name) == Some(Kind::Inline)`。
pub fn resolve_inline(name: &str, args: &[Arg]) -> Resolved {
    let mut warnings = Vec::new();
    // 每个命令只接受一个位置参数，取值来自固定的几个词。
    let mut choose = |allowed: &[&str]| -> Option<String> {
        let mut picked = None;
        for a in args {
            if a.key.is_some() || !allowed.contains(&a.value.as_str()) {
                warnings.push((
                    a.span,
                    format!(
                        "\\{name} 不接受參數「{}」（可用：{}）",
                        show(a),
                        allowed.join(", ")
                    ),
                ));
            } else {
                picked = Some(a.value.clone());
            }
        }
        picked
    };
    let kind = match name {
        "note" => match choose(&["double", "single"]).as_deref() {
            Some("single") => ElementKind::SingleNote,
            _ => ElementKind::DoubleNote,
        },
        "side" => ElementKind::Side {
            left: choose(&["right", "left"]).as_deref() == Some("left"),
        },
        "emph" => ElementKind::Emphasis(
            match choose(&["circle", "dot", "sesame", "comma"]).as_deref() {
                Some("dot") => EmphMark::Dot,
                Some("sesame") => EmphMark::Sesame,
                Some("comma") => EmphMark::Comma,
                _ => EmphMark::Circle,
            },
        ),
        "inv" | "frame" => {
            let shape = match choose(&["square", "round", "hex", "each"]).as_deref() {
                Some("round") => Shape::Round,
                Some("hex") => Shape::Hex,
                Some("each") => Shape::Each,
                _ => Shape::Square,
            };
            if name == "inv" {
                ElementKind::Inverse(shape)
            } else {
                ElementKind::Frame(shape)
            }
        }
        "color" => {
            let names: Vec<&str> = Color::NAMES.split(", ").collect();
            let picked = choose(&names);
            if picked.is_none() && args.is_empty() {
                warnings.push((
                    Span::default(),
                    format!(
                        "\\color 需要指定顏色，如 \\color[red]{{…}}（可用：{}）",
                        Color::NAMES
                    ),
                ));
            }
            ElementKind::Color(
                picked
                    .and_then(|c| Color::from_name(&c))
                    .unwrap_or(Color::Red),
            )
        }
        other => {
            choose(&[]);
            match other {
                "comment" => ElementKind::Comment,
                "head" => ElementKind::HeadNote,
                "name" => ElementKind::Name,
                "title" => ElementKind::Title { brackets: None },
                "stroke" => ElementKind::Stroke,
                "del" => ElementKind::Delete,
                "ins" => ElementKind::Insert,
                "compose" => ElementKind::Compose(String::new()),
                _ => unreachable!("not an inline command: {other}"),
            }
        }
    };
    Resolved { kind, warnings }
}

/// 取唯一的数字参数（`[2]` 或 `[raise=2]` 都行）。
pub fn number_arg(args: &[Arg], key: &str) -> Result<Option<u8>, (Span, String)> {
    match args {
        [] => Ok(None),
        [a] if a.key.as_deref().is_none_or(|k| k == key) => a
            .value
            .parse::<u8>()
            .map(Some)
            .map_err(|_| (a.span, format!("「{}」應是 0–255 的數字", show(a)))),
        [a, ..] => Err((
            a.span,
            format!("這裏只接受一個數字參數，如 [{key}=2] 或 [2]"),
        )),
    }
}

pub fn show(a: &Arg) -> String {
    match &a.key {
        Some(k) => format!("{k}={}", a.value),
        None => a.value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(s: &str) -> Arg {
        match s.split_once('=') {
            Some((k, v)) => Arg {
                key: Some(k.into()),
                value: v.into(),
                span: Span::default(),
            },
            None => Arg {
                key: None,
                value: s.into(),
                span: Span::default(),
            },
        }
    }

    #[test]
    fn variants() {
        assert_eq!(
            resolve_inline("note", &[arg("single")]).kind,
            ElementKind::SingleNote
        );
        assert_eq!(
            resolve_inline("inv", &[arg("round")]).kind,
            ElementKind::Inverse(Shape::Round)
        );
        assert_eq!(
            resolve_inline("frame", &[arg("hex")]).kind,
            ElementKind::Frame(Shape::Hex)
        );
        assert_eq!(
            resolve_inline("color", &[arg("blue")]).kind,
            ElementKind::Color(Color::Blue)
        );
        assert_eq!(
            resolve_inline("side", &[arg("left")]).kind,
            ElementKind::Side { left: true }
        );
    }

    #[test]
    fn bad_args_warn() {
        assert_eq!(resolve_inline("name", &[arg("left")]).warnings.len(), 1);
        assert_eq!(resolve_inline("inv", &[arg("oval")]).warnings.len(), 1);
        assert_eq!(resolve_inline("color", &[]).warnings.len(), 1);
    }

    #[test]
    fn numbers() {
        assert_eq!(number_arg(&[arg("2")], "raise"), Ok(Some(2)));
        assert_eq!(number_arg(&[arg("raise=3")], "raise"), Ok(Some(3)));
        assert_eq!(number_arg(&[], "raise"), Ok(None));
        assert!(number_arg(&[arg("two")], "raise").is_err());
        assert!(number_arg(&[arg("indent=2")], "raise").is_err());
    }
}
