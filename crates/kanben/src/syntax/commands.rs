//! 命令表：把通用的 `\名[參數]{內容}` 解释成具体含义。
//!
//! 解析器只认形式，不认命令；所有命令名、参数都在这里。

use super::ast::{Color, ElementKind, EmphMark, Encl, Shape};
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
        | "frame" | "del" | "ins" | "color" | "compose" | "span" | "rod" => Kind::Inline,
        "setup" | "book" | "volume" | "studio" | "heading" | "begin" | "end" | "indent"
        | "raise" | "bottom" | "toc" => Kind::Block,
        _ => return None,
    })
}

pub const INLINE_COMMANDS: &str = "note, side, comment, head, name, title, emph, stroke, inv, frame, del, ins, color, compose, span, rod";

pub struct Resolved {
    pub kind: ElementKind,
    pub warnings: Vec<(Span, String)>,
}

/// 解释行内命令。调用前已确认 `kind_of(name) == Some(Kind::Inline)`。
pub fn resolve_inline(name: &str, args: &[Arg]) -> Resolved {
    let mut warnings = Vec::new();
    // 每个命令只接受一个位置参数，取值来自固定的几个词。
    let choose = |allowed: &[&str], warnings: &mut Vec<(Span, String)>| {
        choose_from(name, args, allowed, warnings)
    };
    let kind = match name {
        "note" => match choose(&["double", "single", "right", "left"], &mut warnings).as_deref() {
            Some("single") => ElementKind::SingleNote,
            Some("right") => ElementKind::HalfNote { left: false },
            Some("left") => ElementKind::HalfNote { left: true },
            _ => ElementKind::DoubleNote,
        },
        "side" => ElementKind::Side {
            left: choose(&["right", "left"], &mut warnings).as_deref() == Some("left"),
        },
        "emph" => ElementKind::Emphasis(
            match choose(&["circle", "dot", "sesame", "comma"], &mut warnings).as_deref() {
                Some("dot") => EmphMark::Dot,
                Some("sesame") => EmphMark::Sesame,
                Some("comma") => EmphMark::Comma,
                _ => EmphMark::Circle,
            },
        ),
        "inv" | "frame" => {
            // `narrow` 可以和形狀並用：`\frame[hex, narrow]`。
            let narrow = args.iter().any(|a| a.key.is_none() && a.value == "narrow");
            let rest: Vec<Arg> = args
                .iter()
                .filter(|a| !(a.key.is_none() && a.value == "narrow"))
                .cloned()
                .collect();
            let shape = match choose_from(name, &rest, SHAPES, &mut warnings).as_deref() {
                Some("round") => Shape::Round,
                Some("hex") => Shape::Hex,
                Some("flower") => Shape::Flower,
                Some("each") => Shape::Each,
                Some("lantern") => Shape::Lantern,
                _ => Shape::Square,
            };
            let encl = Encl { shape, narrow };
            if name == "inv" {
                ElementKind::Inverse(encl)
            } else {
                ElementKind::Frame(encl)
            }
        }
        "span" => {
            let mut prev = false;
            let mut n = 1;
            for a in args {
                match (a.key.as_deref(), a.value.as_str()) {
                    (None, "next") => prev = false,
                    (None, "prev") => prev = true,
                    (None | Some("n"), v)
                        if v.parse::<u8>().is_ok_and(|v| (1..=3).contains(&v)) =>
                    {
                        n = v.parse().expect("checked");
                    }
                    _ => warnings.push((
                        a.span,
                        format!("\\span 不接受參數「{}」（可用：next, prev, 1–3）", show(a)),
                    )),
                }
            }
            ElementKind::Span { prev, n }
        }
        "color" => {
            let names: Vec<&str> = Color::NAMES.split(", ").collect();
            let picked = choose(&names, &mut warnings);
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
            choose(&[], &mut warnings);
            match other {
                "comment" => ElementKind::Comment,
                "head" => ElementKind::HeadNote,
                "name" => ElementKind::Name,
                "title" => ElementKind::Title { brackets: None },
                "stroke" => ElementKind::Stroke,
                "del" => ElementKind::Delete,
                "ins" => ElementKind::Insert,
                "compose" => ElementKind::Compose(String::new()),
                "rod" => ElementKind::Rod(String::new()),
                _ => unreachable!("not an inline command: {other}"),
            }
        }
    };
    Resolved { kind, warnings }
}

const SHAPES: &[&str] = &[
    "square", "round", "hex", "flower", "each", "lantern", "narrow",
];

/// 从固定的几个词里取一个位置参数，其余的记为提示。
fn choose_from(
    name: &str,
    args: &[Arg],
    allowed: &[&str],
    warnings: &mut Vec<(Span, String)>,
) -> Option<String> {
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
            ElementKind::Inverse(Encl {
                shape: Shape::Round,
                narrow: false
            })
        );
        assert_eq!(
            resolve_inline("frame", &[arg("hex"), arg("narrow")]).kind,
            ElementKind::Frame(Encl {
                shape: Shape::Hex,
                narrow: true
            })
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
