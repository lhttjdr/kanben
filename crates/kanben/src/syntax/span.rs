//! 源码位置与诊断信息。

/// 源码中的字节区间 `[start, end)`。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub fn join(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            span,
            message: message.into(),
        }
    }

    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            span,
            message: message.into(),
        }
    }
}

/// 把字节偏移换算成行号、列号（均从 1 起，列按字符计）。
pub struct LineIndex<'a> {
    src: &'a str,
    starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut starts = vec![0];
        starts.extend(src.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex { src, starts }
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.src.len());
        let line = match self.starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let start = self.starts[line];
        let col = self.src[start..offset].chars().count() + 1;
        (line + 1, col)
    }
}
