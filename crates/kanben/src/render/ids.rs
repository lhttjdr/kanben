//! IDS 组字：把 `⿰木鬼` 之类拆成若干部件，算出每个部件在字格里的位置。

/// 字格内的矩形，取值 0..1。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const FULL: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };

    fn sub(self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {
            x: self.x + x * self.w,
            y: self.y + y * self.h,
            w: w * self.w,
            h: h * self.h,
        }
    }
}

enum Node {
    Leaf(String),
    Op(char, Vec<Node>),
}

fn arity(c: char) -> Option<usize> {
    match c {
        '⿲' | '⿳' => Some(3),
        '\u{2FF0}'..='\u{2FFB}' => Some(2),
        _ => None,
    }
}

fn parse(chars: &[char], i: &mut usize) -> Option<Node> {
    let c = *chars.get(*i)?;
    *i += 1;
    match arity(c) {
        Some(n) => {
            let kids = (0..n)
                .map(|_| parse(chars, i))
                .collect::<Option<Vec<_>>>()?;
            Some(Node::Op(c, kids))
        }
        None => {
            let mut leaf = c.to_string();
            // 异体字选择符跟着前一个部件。
            while let Some(&v) = chars.get(*i) {
                if matches!(v as u32, 0xFE00..=0xFE0F | 0xE0100..=0xE01EF) {
                    leaf.push(v);
                    *i += 1;
                } else {
                    break;
                }
            }
            Some(Node::Leaf(leaf))
        }
    }
}

/// 拆分 IDS。格式不对时返回 `None`。
pub fn components(ids: &str) -> Option<Vec<(String, Rect)>> {
    let chars: Vec<char> = ids.chars().collect();
    let mut i = 0;
    let root = parse(&chars, &mut i)?;
    if i != chars.len() {
        return None;
    }
    let mut out = Vec::new();
    place(&root, Rect::FULL, &mut out);
    Some(out)
}

fn place(node: &Node, r: Rect, out: &mut Vec<(String, Rect)>) {
    let kids = match node {
        Node::Leaf(s) => {
            out.push((s.clone(), r));
            return;
        }
        Node::Op(op, kids) => (op, kids),
    };
    let (op, kids) = kids;
    let rects: Vec<Rect> = match op {
        '⿰' => vec![r.sub(0.0, 0.0, 0.48, 1.0), r.sub(0.48, 0.0, 0.52, 1.0)],
        '⿱' => vec![r.sub(0.0, 0.0, 1.0, 0.48), r.sub(0.0, 0.48, 1.0, 0.52)],
        '⿲' => vec![
            r.sub(0.0, 0.0, 0.33, 1.0),
            r.sub(0.33, 0.0, 0.34, 1.0),
            r.sub(0.67, 0.0, 0.33, 1.0),
        ],
        '⿳' => vec![
            r.sub(0.0, 0.0, 1.0, 0.33),
            r.sub(0.0, 0.33, 1.0, 0.34),
            r.sub(0.0, 0.67, 1.0, 0.33),
        ],
        '⿴' => vec![r, r.sub(0.25, 0.25, 0.5, 0.5)],
        '⿵' => vec![r, r.sub(0.2, 0.35, 0.6, 0.6)],
        '⿶' => vec![r, r.sub(0.2, 0.05, 0.6, 0.6)],
        '⿷' => vec![r, r.sub(0.35, 0.2, 0.6, 0.6)],
        '⿸' => vec![r, r.sub(0.35, 0.35, 0.63, 0.63)],
        '⿹' => vec![r, r.sub(0.02, 0.35, 0.63, 0.63)],
        '⿺' => vec![r, r.sub(0.35, 0.02, 0.63, 0.63)],
        _ => vec![r, r],
    };
    for (k, rect) in kids.iter().zip(rects) {
        place(k, rect, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_right() {
        let c = components("⿰木鬼").unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].0, "木");
        assert!(c[0].1.w < 0.5 && c[1].1.x > 0.4);
    }

    #[test]
    fn nested() {
        let c = components("⿱艹⿰氵去").unwrap();
        assert_eq!(c.iter().map(|c| c.0.as_str()).collect::<String>(), "艹氵去");
    }

    #[test]
    fn invalid() {
        assert!(components("⿰木").is_none());
        assert!(components("木鬼").is_none());
    }
}
