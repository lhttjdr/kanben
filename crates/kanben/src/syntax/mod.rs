//! 刊本语法：源码 → 语法树。
//!
//! 标记只用 ASCII 符号（LaTeX 风格），正文只用书里本来就有的字。
//!
//! 块级（都写在行首）：
//! ```text
//! \book{資治通鑑}  \volume{周紀一}  \studio{刊本}
//! \setup[columns=9, cells=18, format=leaves, punct=judou]
//! \heading{資治通鑑卷第一}   \heading[2]{周紀一}
//! \indent[2] 段落…            段首：整段；段内某行行首：只管该行
//! \begin{indent}[2] … \end{indent}
//! 段内换行 = 平擡；空一行分段；每多空一行，版面多空一行
//! % 注释到行尾
//! ```

pub mod ast;
pub mod commands;
pub mod inline;
pub mod palette;
pub mod span;

use ast::{
    Attr, AttrKind, Block, Document, EnvKind, Inline, Line as ParaLine, MetaEntry, ParaStyle,
    TableHeader,
};
use commands::Kind;
use inline::{CommandParts, collect_text, parse_command_at};
use span::{Diagnostic, Span};

pub struct Parsed {
    pub document: Document,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse(src: &str) -> Parsed {
    let mut diags = Vec::new();
    let mut p = BlockParser::new(&mut diags);
    for line in split_lines(src) {
        p.line(&line);
    }
    let document = p.finish();
    Parsed {
        document,
        diagnostics: diags,
    }
}

#[derive(Clone, Copy)]
struct Line<'a> {
    /// 去掉换行符和注释后的内容。
    text: &'a str,
    /// `text` 在源码中的字节偏移。
    start: usize,
}

impl Line<'_> {
    fn span(&self) -> Span {
        Span::new(self.start, self.start + self.text.len())
    }

    /// 子串在源码中的偏移。`sub` 必须是 `self.text` 的切片。
    fn offset_of(&self, sub: &str) -> usize {
        self.start + (sub.as_ptr() as usize - self.text.as_ptr() as usize)
    }
}

fn split_lines(src: &str) -> Vec<Line<'_>> {
    let body = src.strip_prefix('\u{feff}').unwrap_or(src);
    let mut start = src.len() - body.len();
    let mut out = Vec::new();
    for raw in body.split('\n') {
        let text = raw.strip_suffix('\r').unwrap_or(raw);
        out.push(Line {
            text: strip_comment(text),
            start,
        });
        start += raw.len() + 1;
    }
    out
}

/// 去掉 `%` 起的注释（`\%` 不算）。
fn strip_comment(text: &str) -> &str {
    let mut iter = text.char_indices();
    while let Some((i, c)) = iter.next() {
        match c {
            '\\' => {
                iter.next();
            }
            '%' => return &text[..i],
            _ => {}
        }
    }
    text
}

const SPACE: [char; 2] = [' ', '\t'];

/// 是不是段落属性命令。
fn is_attr(name: &str) -> bool {
    matches!(
        name,
        "indent"
            | "raise"
            | "bottom"
            | "byline"
            | "outline"
            | "detail"
            | "remark"
            | "pipa"
            | "list"
            | "colors"
    )
}

/// 段落属性命令及其参数 → 属性。
fn attr_kind(name: &str, args: &[commands::Arg]) -> Result<AttrKind, (Span, String)> {
    let number = |default| commands::number_arg(args, name).map(|n| n.unwrap_or(default));
    let none = |kind| match args.first() {
        Some(a) => Err((a.span, format!("\\{name} 不帶參數"))),
        None => Ok(kind),
    };
    match name {
        "indent" => number(1).map(AttrKind::Indent),
        "raise" => number(1).map(AttrKind::Raise),
        "bottom" => number(0).map(AttrKind::Bottom),
        "byline" => none(AttrKind::Style(ParaStyle::Byline)),
        "outline" => none(AttrKind::Style(ParaStyle::Outline)),
        "detail" => none(AttrKind::Style(ParaStyle::Detail)),
        "remark" => none(AttrKind::Style(ParaStyle::Remark)),
        "pipa" => none(AttrKind::Style(ParaStyle::Pipa)),
        "list" => list_kind(name, args).map(|ordered| AttrKind::List { ordered }),
        "colors" => {
            let names: Vec<String> = args.iter().map(|a| a.value.clone()).collect();
            if let Some(a) = args.iter().find(|a| palette::lookup(&a.value).is_none()) {
                let all: Vec<&str> = palette::PALETTE.iter().map(|(n, _)| *n).collect();
                return Err((
                    a.span,
                    format!("不認得顏色「{}」（可用：{}）", a.value, all.join("、")),
                ));
            }
            if names.is_empty() || args.iter().any(|a| a.key.is_some()) {
                Err((
                    args.first().map_or(Span::default(), |a| a.span),
                    "寫作 \\colors[孔雀綠, 葛巾紫]：給一種或幾種顏色（幾種就逐行輪換）".into(),
                ))
            } else {
                Ok(AttrKind::Colors(names))
            }
        }
        _ => unreachable!("not an attribute: {name}"),
    }
}

/// `[ordered]`（默认）或 `[unordered]`。
fn list_kind(name: &str, args: &[commands::Arg]) -> Result<bool, (Span, String)> {
    let mut ordered = true;
    for a in args {
        match (a.key.as_deref(), a.value.as_str()) {
            (None, "ordered") => ordered = true,
            (None, "unordered") => ordered = false,
            _ => {
                return Err((
                    a.span,
                    format!(
                        "\\{name} 不接受參數「{}」（可用：ordered, unordered）",
                        commands::show(a)
                    ),
                ));
            }
        }
    }
    Ok(ordered)
}

struct ParaBuilder {
    attrs: Vec<Attr>,
    lines: Vec<ParaLine>,
    span: Span,
}

/// `\begin{…}` 打开的环境。
struct Env {
    name: String,
    /// 段落属性环境（indent 等）给其中每段加的属性。
    attr: Option<Attr>,
    /// 结构环境（quote、list、table）：外层已收的块先存在这里。
    outer: Option<(EnvKind, Vec<Block>)>,
    /// 紧接上一段（中间不空行）。
    attached: bool,
    span: Span,
}

struct BlockParser<'d> {
    diags: &'d mut Vec<Diagnostic>,
    meta: Vec<MetaEntry>,
    blocks: Vec<Block>,
    para: Option<ParaBuilder>,
    envs: Vec<Env>,
    blank_run: usize,
    blank_span: Span,
    /// 刚写了 `\\toc`：紧接着的各行是手写的目次条目。
    toc_open: bool,
}

impl<'d> BlockParser<'d> {
    fn new(diags: &'d mut Vec<Diagnostic>) -> Self {
        BlockParser {
            diags,
            meta: Vec::new(),
            blocks: Vec::new(),
            para: None,
            envs: Vec::new(),
            blank_run: 0,
            blank_span: Span::default(),
            toc_open: false,
        }
    }

    fn finish(mut self) -> Document {
        self.close_para();
        while let Some(env) = self.envs.pop() {
            let msg = format!("\\begin{{{0}}} 沒有對應的 \\end{{{0}}}", env.name);
            self.diags.push(Diagnostic::error(env.span, msg));
            self.close_env(env);
        }
        Document {
            meta: self.meta,
            blocks: self.blocks,
        }
    }

    fn line(&mut self, line: &Line) {
        let t = line.text.trim_matches(SPACE);
        if t.is_empty() {
            self.toc_open = false;
            self.close_para();
            if self.blank_run == 0 {
                self.blank_span = line.span();
            }
            self.blank_run += 1;
            self.blank_span = self.blank_span.join(line.span());
            return;
        }

        // 行首的块级命令（段落属性除外，它们和正文写在同一行）。
        let mut scratch = Vec::new();
        if let Some(cmd) = parse_command_at(t, line.offset_of(t), &mut scratch)
            && commands::kind_of(&cmd.name) == Some(Kind::Block)
            && !is_attr(&cmd.name)
        {
            self.diags.append(&mut scratch);
            let rest = t[cmd.end..].trim_matches(SPACE);
            self.toc_open = false;
            self.block_command(&cmd, line, rest);
            return;
        }

        // 手写的目次条目。
        if self.toc_open
            && let Some(Block::Toc { entries, .. }) = self.blocks.last_mut()
        {
            let content = inline::parse_inline(t, line.offset_of(t), self.diags);
            entries.push(ParaLine {
                attrs: Vec::new(),
                content,
            });
            return;
        }

        self.flush_blanks();
        let (attrs, content) = self.leading_attrs(line, t);
        let line_attrs = match self.para {
            Some(_) => attrs,
            None => {
                let mut all: Vec<Attr> = self.envs.iter().filter_map(|e| e.attr.clone()).collect();
                all.extend(attrs);
                self.para = Some(ParaBuilder {
                    attrs: all,
                    lines: Vec::new(),
                    span: line.span(),
                });
                Vec::new()
            }
        };
        if content.is_empty() {
            if !line_attrs.is_empty() {
                self.diags
                    .push(Diagnostic::warning(line.span(), "行首屬性後面沒有正文"));
            }
        } else {
            let content = inline::parse_inline(content, line.offset_of(content), self.diags);
            let para = self.para.as_mut().expect("paragraph is open");
            para.lines.push(ParaLine {
                attrs: line_attrs,
                content,
            });
        }
        let para = self.para.as_mut().expect("paragraph is open");
        para.span = para.span.join(line.span());
    }

    /// 解析行首的 `\indent[2]` 之类，返回属性和剩下的正文。
    fn leading_attrs<'l>(&mut self, line: &Line, mut s: &'l str) -> (Vec<Attr>, &'l str) {
        let mut out = Vec::new();
        loop {
            let mut scratch = Vec::new();
            let Some(cmd) = parse_command_at(s, line.offset_of(s), &mut scratch) else {
                break;
            };
            if !is_attr(&cmd.name) {
                break;
            }
            self.diags.append(&mut scratch);
            if cmd.body.is_some() {
                let msg = format!("段落屬性不帶內容：寫作 \\{}[2]，後面直接接正文", cmd.name);
                self.diags.push(Diagnostic::error(cmd.span, msg));
            }
            match attr_kind(&cmd.name, &cmd.args) {
                Ok(kind) => out.push(Attr {
                    kind,
                    span: cmd.span,
                }),
                Err((span, msg)) => {
                    let span = if span == Span::default() {
                        cmd.span
                    } else {
                        span
                    };
                    self.diags.push(Diagnostic::error(span, msg))
                }
            }
            s = s[cmd.end..].trim_start_matches(SPACE);
        }
        (out, s)
    }

    fn block_command(&mut self, cmd: &CommandParts, line: &Line, rest: &str) {
        let attached = self.para.is_some();
        self.close_para();
        let name = cmd.name.as_str();
        if name != "heading" && !rest.is_empty() {
            let msg = format!("\\{name} 這一行後面不能再寫正文");
            self.diags.push(Diagnostic::error(
                Span::new(line.offset_of(rest), line.offset_of(rest) + rest.len()),
                msg,
            ));
        }
        match name {
            "setup" => {
                if cmd.body.is_some() || cmd.args.is_empty() {
                    self.diags
                        .push(Diagnostic::error(cmd.span, "寫作 \\setup[鍵=值, …]"));
                }
                for a in &cmd.args {
                    match &a.key {
                        Some(k) => self.meta.push(MetaEntry {
                            key: k.clone(),
                            value: a.value.clone(),
                            span: a.span,
                        }),
                        None => self
                            .diags
                            .push(Diagnostic::error(a.span, "\\setup 的參數應寫成 鍵=值")),
                    }
                }
            }
            "book" | "volume" | "studio" => match self.body_text(cmd) {
                Some(value) => self.meta.push(MetaEntry {
                    key: name.into(),
                    value,
                    span: cmd.span,
                }),
                None => self
                    .diags
                    .push(Diagnostic::error(cmd.span, format!("寫作 \\{name}{{…}}"))),
            },
            "heading" => {
                self.flush_blanks();
                let level = match commands::number_arg(&cmd.args, "level") {
                    Ok(n) => n.unwrap_or(1).clamp(1, 6),
                    Err((span, msg)) => {
                        self.diags.push(Diagnostic::error(span, msg));
                        1
                    }
                };
                let Some(mut content) = cmd.body.clone() else {
                    self.diags.push(Diagnostic::error(
                        cmd.span,
                        "寫作 \\heading{標題} 或 \\heading[2]{標題}",
                    ));
                    return;
                };
                if !rest.is_empty() {
                    content.extend(inline::parse_inline(rest, line.offset_of(rest), self.diags));
                }
                self.blocks.push(Block::Heading {
                    level,
                    content,
                    span: line.span(),
                });
            }
            "begin" => {
                let Some(env) = self.body_text(cmd) else {
                    self.diags
                        .push(Diagnostic::error(cmd.span, "寫作 \\begin{indent}[2]"));
                    return;
                };
                let args = if cmd.post_args.is_empty() {
                    &cmd.args
                } else {
                    &cmd.post_args
                };
                let mut attr = None;
                let mut outer = None;
                match env.as_str() {
                    "quote" | "list" | "table" => match structural_env(&env, args) {
                        Ok(kind) => outer = Some((kind, std::mem::take(&mut self.blocks))),
                        Err((span, msg)) => {
                            let span = if span == Span::default() {
                                cmd.span
                            } else {
                                span
                            };
                            self.diags.push(Diagnostic::error(span, msg));
                        }
                    },
                    "indent" | "raise" | "bottom" => match attr_kind(&env, args) {
                        Ok(kind) => {
                            attr = Some(Attr {
                                kind,
                                span: cmd.span,
                            })
                        }
                        Err((span, msg)) => self.diags.push(Diagnostic::error(span, msg)),
                    },
                    _ => {
                        let msg = format!(
                            "未知的環境「{env}」（可用：indent, raise, bottom, quote, list, table）"
                        );
                        self.diags.push(Diagnostic::error(cmd.span, msg));
                    }
                }
                self.envs.push(Env {
                    name: env,
                    attr,
                    outer,
                    attached,
                    span: cmd.span,
                });
            }
            "toc" => {
                self.flush_blanks();
                let depth = match commands::number_arg(&cmd.args, "depth") {
                    Ok(n) => n.unwrap_or(6).clamp(1, 6),
                    Err((span, msg)) => {
                        self.diags.push(Diagnostic::error(span, msg));
                        6
                    }
                };
                if cmd.body.is_some() {
                    self.diags.push(Diagnostic::warning(
                        cmd.span,
                        "\\toc 不帶內容，寫作 \\toc 或 \\toc[2]",
                    ));
                }
                self.blocks.push(Block::Toc {
                    depth,
                    entries: Vec::new(),
                    span: cmd.span,
                });
                self.toc_open = true;
            }
            "end" => {
                let env = self.body_text(cmd).unwrap_or_default();
                match self.envs.last() {
                    Some(top) if top.name == env => {
                        let top = self.envs.pop().expect("checked");
                        self.close_env(top);
                    }
                    Some(top) => {
                        let msg =
                            format!("應先結束 \\begin{{{}}}，而不是 \\end{{{env}}}", top.name);
                        self.diags.push(Diagnostic::error(cmd.span, msg));
                    }
                    None => self.diags.push(Diagnostic::warning(
                        cmd.span,
                        format!("多餘的 \\end{{{env}}}"),
                    )),
                }
            }
            _ => unreachable!("not a block command: {name}"),
        }
    }

    /// 命令内容里的纯文字（`\book{…}`、`\begin{…}`）。
    fn body_text(&mut self, cmd: &CommandParts) -> Option<String> {
        let body = cmd.body.as_ref()?;
        if body.iter().any(|n| matches!(n, Inline::Element(_))) {
            self.diags.push(Diagnostic::warning(
                cmd.span,
                format!("\\{} 的內容只取文字，其中的標記已忽略", cmd.name),
            ));
        }
        let mut s = String::new();
        body.iter().for_each(|n| collect_text(n, &mut s));
        Some(s.trim().to_string())
    }

    fn close_para(&mut self) {
        if let Some(p) = self.para.take() {
            if p.lines.is_empty() {
                if !p.attrs.is_empty() {
                    self.diags
                        .push(Diagnostic::warning(p.span, "段落屬性後面沒有正文"));
                }
            } else {
                self.blocks.push(Block::Paragraph {
                    attrs: p.attrs,
                    lines: p.lines,
                    span: p.span,
                });
            }
        }
    }

    /// 结构环境结束：其中的块收成一个 `Block::Env`，接回外层。
    fn close_env(&mut self, env: Env) {
        self.close_para();
        if let Some((kind, outer)) = env.outer {
            let inner = std::mem::replace(&mut self.blocks, outer);
            self.blank_run = 0;
            self.blocks.push(Block::Env {
                kind,
                blocks: inner,
                attached: env.attached,
                span: env.span,
            });
        }
    }

    fn flush_blanks(&mut self) {
        if self.blank_run >= 2 && !self.blocks.is_empty() {
            self.blocks.push(Block::Blank {
                count: self.blank_run - 1,
                span: self.blank_span,
            });
        }
        self.blank_run = 0;
    }
}

/// `\\begin{quote}`、`\\begin{list}[unordered]`、`\\begin{table}[標題, header=both]`。
fn structural_env(env: &str, args: &[commands::Arg]) -> Result<EnvKind, (Span, String)> {
    match env {
        "quote" => match args.first() {
            Some(a) => Err((a.span, "\\begin{quote} 不帶參數".into())),
            None => Ok(EnvKind::Quote),
        },
        "list" => list_kind(env, args).map(|ordered| EnvKind::List { ordered }),
        "table" => {
            let mut title = String::new();
            let mut header = TableHeader::None;
            for a in args {
                match (a.key.as_deref(), a.value.as_str()) {
                    (Some("header"), "none") => header = TableHeader::None,
                    (Some("header"), "row") => header = TableHeader::Row,
                    (Some("header"), "column") => header = TableHeader::Column,
                    (Some("header"), "both") => header = TableHeader::Both,
                    (Some("title") | None, v) => title = v.to_string(),
                    _ => {
                        return Err((
                            a.span,
                            format!(
                                "\\begin{{table}} 不接受參數「{}」（可用：標題、header=none|row|column|both）",
                                commands::show(a)
                            ),
                        ));
                    }
                }
            }
            Ok(EnvKind::Table { title, header })
        }
        _ => unreachable!("not a structural environment: {env}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(src: &str) -> Parsed {
        parse(src)
    }

    fn attrs_of(b: &Block) -> Vec<AttrKind> {
        match b {
            Block::Paragraph { attrs, .. } => attrs.iter().map(|a| a.kind.clone()).collect(),
            _ => vec![],
        }
    }

    #[test]
    fn setup_and_blocks() {
        let r = p(
            "\\book{資治通鑑}\n\\setup[columns=9, cells=18]\n\\heading{卷一}\n\n甲乙\n丙\n\n\n\n丁",
        );
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
        let d = r.document;
        let meta: Vec<(&str, &str)> = d
            .meta
            .iter()
            .map(|m| (m.key.as_str(), m.value.as_str()))
            .collect();
        assert_eq!(
            meta,
            [("book", "資治通鑑"), ("columns", "9"), ("cells", "18")]
        );
        assert!(matches!(d.blocks[0], Block::Heading { level: 1, .. }));
        match &d.blocks[1] {
            Block::Paragraph { lines, .. } => assert_eq!(lines.len(), 2),
            b => panic!("{b:?}"),
        }
        assert!(matches!(d.blocks[2], Block::Blank { count: 2, .. }));
        assert!(matches!(d.blocks[3], Block::Paragraph { .. }));
    }

    #[test]
    fn heading_levels_and_trailing_text() {
        let r = p("\\heading[2]{周紀一}\\note{起著雍}");
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
        match &r.document.blocks[0] {
            Block::Heading { level, content, .. } => {
                assert_eq!(*level, 2);
                assert_eq!(content.len(), 2);
            }
            b => panic!("{b:?}"),
        }
    }

    #[test]
    fn attrs_and_environments() {
        let r = p("\\begin{indent}[2]\n\\raise[1] \\bottom 甲\n\n乙\n\\end{indent}\n丙");
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
        let attrs: Vec<Vec<AttrKind>> = r.document.blocks.iter().map(attrs_of).collect();
        assert_eq!(
            attrs,
            vec![
                vec![AttrKind::Indent(2), AttrKind::Raise(1), AttrKind::Bottom(0)],
                vec![AttrKind::Indent(2)],
                vec![],
            ]
        );
    }

    #[test]
    fn line_attrs() {
        let r = p("\\bottom[1] 甲\n\\bottom[2] 乙\n丙");
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
        let Block::Paragraph { attrs, lines, .. } = &r.document.blocks[0] else {
            panic!()
        };
        assert_eq!(
            attrs.iter().map(|a| a.kind.clone()).collect::<Vec<_>>(),
            [AttrKind::Bottom(1)]
        );
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[1]
                .attrs
                .iter()
                .map(|a| a.kind.clone())
                .collect::<Vec<_>>(),
            [AttrKind::Bottom(2)]
        );
        assert!(lines[2].attrs.is_empty());
    }

    #[test]
    fn spans_point_into_source() {
        let src = "\\setup[cells=9]\n甲\\note{乙";
        let r = p(src);
        assert_eq!(r.diagnostics.len(), 1);
        let s = r.diagnostics[0].span;
        assert_eq!(&src[s.start..s.end], "\\note{");
    }

    #[test]
    fn comments() {
        let r = p("甲 % 注释\n乙\\%丙");
        assert!(r.diagnostics.is_empty());
        let Block::Paragraph { lines, .. } = &r.document.blocks[0] else {
            panic!()
        };
        let text = |l: &ParaLine| {
            let mut s = String::new();
            l.content.iter().for_each(|n| collect_text(n, &mut s));
            s
        };
        assert_eq!(text(&lines[0]), "甲");
        assert_eq!(text(&lines[1]), "乙%丙");
    }

    #[test]
    fn environment_errors() {
        let r = p("\\begin{indent}[2]\n甲\n\\end{bottom}");
        assert_eq!(r.diagnostics.len(), 2, "{:?}", r.diagnostics);
        let r = p("\\begin{wide}\n甲\n\\end{wide}");
        assert_eq!(r.diagnostics.len(), 1);
    }

    #[test]
    fn setup_needs_key_values() {
        let r = p("\\setup[nine]");
        assert_eq!(r.diagnostics.len(), 1);
        let r = p("\\book{甲} 乙");
        assert_eq!(r.diagnostics.len(), 1);
    }
}
