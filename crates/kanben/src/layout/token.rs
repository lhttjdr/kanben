//! 排版用的最小单位：字（Tok）与字格（Unit）。

use super::options::PunctStyle;
use crate::syntax::ast::{Color, EmphMark, Encl};

#[derive(Clone, Debug, PartialEq)]
pub struct Tok {
    /// 要显示的文字（组字时是 IDS 串）。
    pub text: String,
    pub kind: TokKind,
    pub tone: Option<Tone>,
    pub style: Style,
    pub compose: bool,
    /// 算籌數碼：`text` 是碼子字（〇〡…〩），整串排在一格裏。
    pub rod: bool,
    /// 不显示、但复制时要有的字（画成波綫的書名號的括號）。
    pub hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokKind {
    /// 佔一格的字。
    Cell,
    /// 佔一格的空白（全角空格）。
    Space,
    /// 不佔格，附在前一字之後（句讀、後引號）。
    After,
    /// 不佔格，附在後一字之前（前引號）。
    Before,
}

/// 四聲圈發，畫在字的四角。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// 平 〪 左下
    Level,
    /// 上 〫 左上
    Rising,
    /// 去 〬 右上
    Departing,
    /// 入 〭 右下
    Entering,
}

impl Tone {
    pub fn from_char(c: char) -> Option<Tone> {
        Some(match c {
            '〪' => Tone::Level,
            '〫' => Tone::Rising,
            '〬' => Tone::Departing,
            '〭' => Tone::Entering,
            _ => return None,
        })
    }

    /// 相对字心的方向：(x, y)，x 向右为正，y 向下为正。
    pub fn corner(self) -> (f32, f32) {
        match self {
            Tone::Level => (-1.0, 1.0),
            Tone::Rising => (-1.0, -1.0),
            Tone::Departing => (1.0, -1.0),
            Tone::Entering => (1.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub marks: Vec<Mark>,
    pub color: Option<Color>,
}

/// 加在一串字上的标记。同一个 `id` 的字连成一段来画（綫、框）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Mark {
    pub id: u32,
    pub kind: MarkKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarkKind {
    Name,
    Title,
    /// 篇名（〈〉）：原版的波綫在字的右側。
    TitleRight,
    Emphasis(EmphMark),
    Stroke,
    Inverse(Encl),
    Frame(Encl),
    Delete,
    Insert,
}

/// 是否是附在前一个字上的组合字符（异体字选择符、ZWJ 等）。
pub fn is_combining(c: char) -> bool {
    matches!(c as u32, 0xFE00..=0xFE0F | 0xE0100..=0xE01EF | 0x200D | 0x20E3)
}

/// 判定一个字符怎样排。返回 `None` 表示不排（被丢弃）。
pub fn classify(c: char, style: PunctStyle) -> Option<(TokKind, char)> {
    if c == '　' {
        return Some((TokKind::Space, c));
    }
    if c.is_whitespace() || c.is_control() {
        return None;
    }
    let vertical = vertical_form(c);
    let class = punct_class(c);
    Some(match (style, class) {
        (_, None) => (TokKind::Cell, vertical.unwrap_or(c)),
        (PunctStyle::Bare, Some(_)) => return None,
        (PunctStyle::Grid, Some(_)) => (TokKind::Cell, vertical.unwrap_or(c)),
        (PunctStyle::Judou, Some(PunctClass::After)) => (TokKind::After, c),
        // 引號保持原字：SVG 文字竪排，瀏覽器自會換成直排字形（與原版相同）。
        (PunctStyle::Judou, Some(PunctClass::CloseQuote)) => (TokKind::After, c),
        (PunctStyle::Judou, Some(PunctClass::OpenQuote)) => (TokKind::Before, c),
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PunctClass {
    After,
    OpenQuote,
    CloseQuote,
}

fn punct_class(c: char) -> Option<PunctClass> {
    Some(match c {
        '，' | '、' | '。' | '．' | '；' | '：' | '？' | '！' | '·' => PunctClass::After,
        '「' | '『' | '“' | '‘' | '《' | '〈' | '【' => PunctClass::OpenQuote,
        '」' | '』' | '”' | '’' | '》' | '〉' | '】' => PunctClass::CloseQuote,
        _ => return None,
    })
}

/// 直排字形（括號、省略號等即使在句讀式下也佔格）。
fn vertical_form(c: char) -> Option<char> {
    Some(match c {
        '，' => '︐',
        '、' => '︑',
        '。' => '︒',
        '：' => '︓',
        '；' => '︔',
        '！' => '︕',
        '？' => '︖',
        '「' | '“' => '﹁',
        '」' | '”' => '﹂',
        '『' | '‘' => '﹃',
        '』' | '’' => '﹄',
        '《' => '︽',
        '》' => '︾',
        '〈' => '︿',
        '〉' => '﹀',
        '【' => '︻',
        '】' => '︼',
        '（' => '︵',
        '）' => '︶',
        '〔' => '︹',
        '〕' => '︺',
        '［' => '﹇',
        '］' => '﹈',
        '…' => '︙',
        '—' => '︱',
        _ => return None,
    })
}

/// 把直排字形还原成普通标点（复制文字时用）。
pub fn horizontal_form(c: char) -> char {
    match c {
        '︐' => '，',
        '︑' => '、',
        '︒' => '。',
        '︓' => '：',
        '︔' => '；',
        '︕' => '！',
        '︖' => '？',
        '﹁' => '「',
        '﹂' => '」',
        '﹃' => '『',
        '﹄' => '』',
        '︽' => '《',
        '︾' => '》',
        '︿' => '〈',
        '﹀' => '〉',
        '︻' => '【',
        '︼' => '】',
        '︵' => '（',
        '︶' => '）',
        '︹' => '〔',
        '︺' => '〕',
        '﹇' => '［',
        '﹈' => '］',
        '︙' => '…',
        '︱' => '—',
        c => c,
    }
}

/// 佔格的標點里不能放在行首的（行首禁則），排不下时挂在上一行末尾。
pub fn is_hanging(t: &Tok) -> bool {
    t.kind == TokKind::Cell
        && matches!(
            t.text.as_str(),
            "︐" | "︑"
                | "︒"
                | "︓"
                | "︔"
                | "︕"
                | "︖"
                | "﹂"
                | "﹄"
                | "︶"
                | "︺"
                | "﹈"
                | "．"
                | "·"
        )
}

/// 一格：格中的字，以及附在它前后、不佔格的標點。
#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub pre: Vec<Tok>,
    pub cell: Tok,
    pub post: Vec<Tok>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Grouped {
    /// 开头就出现、前面没有字可附的「後附」標點（附到上一段的最后一字）。
    pub lead: Vec<Tok>,
    pub units: Vec<Unit>,
    /// 结尾的「前附」標點（留给下一段的第一个字）。
    pub tail: Vec<Tok>,
}

/// 把一串字分成格。
pub fn group(toks: Vec<Tok>) -> Grouped {
    let mut g = Grouped::default();
    let mut pre = Vec::new();
    for t in toks {
        match t.kind {
            TokKind::Cell | TokKind::Space => g.units.push(Unit {
                pre: std::mem::take(&mut pre),
                cell: t,
                post: Vec::new(),
            }),
            TokKind::Before => pre.push(t),
            TokKind::After => match g.units.last_mut() {
                // 前附標點后面紧跟後附標點（如「」」），没有字可附，就给它们各自一格。
                Some(u) if pre.is_empty() => u.post.push(t),
                None if pre.is_empty() => g.lead.push(t),
                // 隐藏的書名號不佔格，没处附就丢掉（只影响复制）。
                _ if t.hidden => {}
                _ => {
                    for p in std::mem::take(&mut pre).into_iter().filter(|p| !p.hidden) {
                        g.units.push(Unit {
                            pre: Vec::new(),
                            cell: p,
                            post: Vec::new(),
                        });
                    }
                    g.units.push(Unit {
                        pre: Vec::new(),
                        cell: t,
                        post: Vec::new(),
                    });
                }
            },
        }
    }
    g.tail = pre;
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str, style: PunctStyle) -> Vec<Tok> {
        s.chars()
            .filter_map(|c| classify(c, style))
            .map(|(kind, c)| Tok {
                text: c.to_string(),
                kind,
                tone: None,
                style: Style::default(),
                compose: false,
                rod: false,
                hidden: false,
            })
            .collect()
    }

    #[test]
    fn judou_attaches_punctuation() {
        let g = group(toks("曰：「天尊地卑。」", PunctStyle::Judou));
        let cells: String = g.units.iter().map(|u| u.cell.text.as_str()).collect();
        assert_eq!(cells, "曰天尊地卑");
        assert_eq!(g.units[0].post[0].text, "：");
        assert_eq!(g.units[1].pre[0].text, "「");
        assert_eq!(
            g.units[4]
                .post
                .iter()
                .map(|t| t.text.as_str())
                .collect::<String>(),
            "。」"
        );
    }

    #[test]
    fn grid_takes_cells() {
        let g = group(toks("曰：「天」", PunctStyle::Grid));
        let cells: String = g.units.iter().map(|u| u.cell.text.as_str()).collect();
        assert_eq!(cells, "曰︓﹁天﹂");
    }

    #[test]
    fn bare_drops_punctuation() {
        let g = group(toks("曰：「天（地）」", PunctStyle::Bare));
        let cells: String = g.units.iter().map(|u| u.cell.text.as_str()).collect();
        assert_eq!(cells, "曰天︵地︶");
    }

    #[test]
    fn vertical_forms_round_trip() {
        for c in "，、。：；！？「」『』《》〈〉【】（）〔〕［］…—".chars()
        {
            assert_eq!(horizontal_form(vertical_form(c).unwrap()), c);
        }
    }

    #[test]
    fn leading_and_trailing() {
        let g = group(toks("。甲「", PunctStyle::Judou));
        assert_eq!(g.lead.len(), 1);
        assert_eq!(g.units.len(), 1);
        assert_eq!(g.tail.len(), 1);
    }
}
