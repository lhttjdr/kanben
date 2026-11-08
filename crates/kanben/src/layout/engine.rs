//! 排版器：按字格逐个放字。

use std::collections::HashMap;

use super::flatten::{Flattener, Item};
use super::token::{MarkKind, Tok, TokKind, Unit, group, is_hanging};
use super::{Deco, Format, Glyph, Layout, Options, Role};
use crate::render::numerals::hanzi;
use crate::syntax::ast::{AttrKind, Block, Color, Document, Inline, Shape};
use crate::syntax::span::{Diagnostic, Span};

// 尺寸（单位：字）。
// 雙行夾注：0.8 字号（原版写了 75% 字宽，但常用字体没有窄体，实际渲染就是 0.8 见方），
// 左右两小行的中线在 +0.45 / -0.35，每行比正文字心高 0.062（加上字形本身上移，墨迹比正文高约 0.09）。这些都是照原版实测的。
const NOTE_SIZE: f32 = 0.8;
const NOTE_STRETCH: f32 = 1.0;
const NOTE_R: f32 = 0.45;
const NOTE_L: f32 = -0.35;
const NOTE_DROP: f32 = -0.062;
const SINGLE_SIZE: f32 = 0.62;
const SIDE_SIZE: f32 = 0.36;
const SIDE_STEP: f32 = 0.42;
const HEAD_SIZE: f32 = 0.46;
const HEAD_STEP: f32 = 0.5;
const LINE_GAP: f32 = 0.06;
/// 挂在行末的標點，字心比最后一格低多少（挤进版框留白）。
const HANG: f32 = 0.2;

/// 一行的格式：低几格、擡几格、是否靠底。
#[derive(Clone, Copy, Default)]
struct LineFmt {
    indent: i32,
    raise: i32,
    bottom: Option<i32>,
}

impl LineFmt {
    /// 段落属性：低格累加（容器与段首的会叠在一起）。
    fn add(&mut self, a: AttrKind) {
        match a {
            AttrKind::Indent(n) => self.indent += i32::from(n),
            AttrKind::Raise(n) => self.raise = self.raise.max(i32::from(n)),
            AttrKind::Bottom(n) => self.bottom = Some(i32::from(n)),
        }
    }

    /// 行首属性：覆盖同类的段落属性。
    fn replace(&mut self, a: AttrKind) {
        match a {
            AttrKind::Indent(n) => self.indent = i32::from(n),
            AttrKind::Raise(n) => self.raise = i32::from(n),
            AttrKind::Bottom(n) => self.bottom = Some(i32::from(n)),
        }
    }
}

/// 一个已放好的字，后附標點要靠它定位。
#[derive(Clone, Copy)]
struct Anchor {
    col: i32,
    x: f32,
    y: f32,
    size: f32,
    posts: usize,
}

/// 字的位置与大小。
#[derive(Clone, Copy)]
struct Slot {
    col: i32,
    x: f32,
    y: f32,
    size: f32,
    stretch: f32,
    /// 半宽，用来定綫、框的位置。
    half_w: f32,
    /// 纵向所占的长度。
    ext: f32,
    role: Role,
}

/// 同一个标记在同一行、同一小行里连成的一段。
struct Seg {
    kind: MarkKind,
    col: i32,
    x: f32,
    half_w: f32,
    y0: f32,
    y1: f32,
}

/// 目次的一條。
#[derive(Clone, Debug, PartialEq)]
pub struct TocEntry {
    pub level: u8,
    pub toks: Vec<Tok>,
    /// 在第幾葉；長卷或第一遍排版時没有。
    pub leaf: Option<i32>,
}

pub struct Engine<'d> {
    o: Options,
    diags: &'d mut Vec<Diagnostic>,
    next_id: u32,
    col: i32,
    row: i32,
    /// 折行后从第几格开始（低格）。
    cont: i32,
    /// 当前行是否已经有字。
    used: bool,
    glyphs: Vec<Glyph>,
    decos: Vec<Deco>,
    segs: Vec<Seg>,
    seg_at: HashMap<(u32, i32, i32), usize>,
    last: Option<Anchor>,
    /// 等着附到下一个字上的前引號。
    pending: Vec<Tok>,
    /// 下一条眉批最早从哪里开始（列坐标，可带小数）。
    head_next: f32,
    /// 右、左旁行各自排到了哪里，避免重叠。
    side_end: [(i32, f32); 2],
    max_col: i32,
    /// 兼行佔去的格：行 → [起, 止)。
    reserved: HashMap<i32, Vec<(i32, i32)>>,
    /// 目次内容（由上一遍排版得出）。
    toc: Vec<TocEntry>,
}

impl<'d> Engine<'d> {
    pub fn new(o: Options, diags: &'d mut Vec<Diagnostic>) -> Self {
        Engine {
            o,
            diags,
            next_id: 0,
            col: 0,
            row: 0,
            cont: 0,
            used: false,
            glyphs: Vec::new(),
            decos: Vec::new(),
            segs: Vec::new(),
            seg_at: HashMap::new(),
            last: None,
            pending: Vec::new(),
            head_next: f32::MIN,
            side_end: [(-1, 0.0); 2],
            max_col: 0,
            reserved: HashMap::new(),
            toc: Vec::new(),
        }
    }

    pub fn with_toc(mut self, toc: Vec<TocEntry>) -> Self {
        self.toc = toc;
        self
    }

    fn cells(&self) -> i32 {
        self.o.cells
    }

    pub fn run(self, doc: &Document) -> Layout {
        self.run_collect(doc).0
    }

    /// 排版，并返回排出的標題（含所在的行）。
    pub fn run_collect(mut self, doc: &Document) -> (Layout, Vec<(TocEntry, i32)>) {
        let mut heads = Vec::new();
        // 目次只列它后面的标题。
        let mut after_toc = false;
        for block in &doc.blocks {
            match block {
                Block::Heading { level, content, .. } => {
                    let indent = if *level == 1 { 0 } else { i32::from(*level) };
                    let fmt = LineFmt {
                        indent,
                        ..LineFmt::default()
                    };
                    let items = self.flatten(content);
                    let toks: Vec<Tok> = items
                        .iter()
                        .filter_map(|i| match i {
                            Item::Main(t) => Some(t.iter().filter(|t| !t.hidden).map(plain)),
                            _ => None,
                        })
                        .flatten()
                        .collect();
                    let before = self.glyphs.len();
                    self.paragraph(vec![(fmt, items)]);
                    for g in &mut self.glyphs[before..] {
                        g.bold = g.role == Role::Main;
                    }
                    let col = self.glyphs.get(before).map_or(self.col, |g| g.col);
                    if after_toc {
                        heads.push((
                            TocEntry {
                                level: *level,
                                toks,
                                leaf: None,
                            },
                            col,
                        ));
                    }
                }
                Block::Paragraph { attrs, lines, .. } => {
                    let mut base = LineFmt::default();
                    for a in attrs {
                        base.add(a.kind);
                    }
                    let lines = lines
                        .iter()
                        .enumerate()
                        .map(|(k, l)| {
                            let mut fmt = base;
                            if k > 0 {
                                fmt.raise = 0;
                            }
                            for a in &l.attrs {
                                fmt.replace(a.kind);
                            }
                            (fmt, self.flatten(&l.content))
                        })
                        .collect();
                    self.paragraph(lines);
                }
                Block::Blank { count, .. } => {
                    self.start_block();
                    self.col += *count as i32;
                }
                Block::Toc { depth, .. } => {
                    after_toc = true;
                    self.toc(*depth);
                }
            }
        }
        (self.finish(), heads)
    }

    /// 目次：每個標題一行，按級低格；葉碼靠底。
    fn toc(&mut self, depth: u8) {
        let entries: Vec<TocEntry> = self
            .toc
            .iter()
            .filter(|e| e.level <= depth)
            .cloned()
            .collect();
        let cells = self.cells();
        for e in entries {
            self.start_block();
            let indent = (i32::from(e.level) - 1).clamp(0, cells - 1);
            self.row = indent;
            self.cont = indent;
            self.item(Item::Main(e.toks));
            if let Some(leaf) = e.leaf {
                let num: Vec<Tok> = hanzi(leaf as u32 + 1)
                    .chars()
                    .map(|c| Tok {
                        text: c.to_string(),
                        kind: TokKind::Cell,
                        tone: None,
                        style: Default::default(),
                        compose: false,
                        hidden: false,
                    })
                    .collect();
                let target = cells - num.len() as i32;
                if self.row < target {
                    self.row = target;
                } else {
                    self.row += 1;
                }
                self.item(Item::Main(num));
            }
        }
    }

    fn flatten(&mut self, nodes: &[Inline]) -> Vec<Item> {
        let mut f = Flattener::new(self.o.punct, self.o.title_mark, self.next_id, self.diags);
        let items = f.line(nodes);
        self.next_id = f.next_id;
        items
    }

    /// 段落从新的一行开始。
    fn start_block(&mut self) {
        if self.used {
            self.col += 1;
        }
        self.used = false;
        self.row = 0;
        self.pending.clear();
        self.last = None;
    }

    fn new_column(&mut self) {
        if self.used {
            self.col += 1;
        }
        self.used = false;
    }

    fn paragraph(&mut self, lines: Vec<(LineFmt, Vec<Item>)>) {
        self.start_block();
        let cells = self.cells();
        let mut first_segment = true;
        for (fmt, line) in lines {
            let indent = fmt.indent.clamp(0, cells - 1);
            // 按【擡】把一行再切成几段，每段另起一行。
            let mut segments: Vec<(i32, Vec<Item>)> = vec![(fmt.raise, Vec::new())];
            for item in line {
                match item {
                    Item::Break { raise } => segments.push((i32::from(raise), Vec::new())),
                    other => segments.last_mut().expect("non-empty").1.push(other),
                }
            }
            for (r, items) in segments {
                if !first_segment {
                    self.new_column();
                }
                first_segment = false;
                let mut first = indent - r;
                if let Some(gap) = fmt.bottom {
                    let n = measure(&items);
                    if n + indent + gap <= cells {
                        first = cells - gap - n;
                    }
                }
                self.row = first;
                self.cont = indent;
                for item in items {
                    self.item(item);
                }
            }
        }
    }

    fn item(&mut self, item: Item) {
        match item {
            Item::Main(toks) => {
                let g = group(toks);
                self.lead(g.lead);
                for u in g.units {
                    // 行首禁則：標點挂在上一行末尾，不另起一行。
                    if self.row >= self.cells() && self.used && is_hanging(&u.cell) {
                        let slot = Slot {
                            col: self.col,
                            x: 0.0,
                            y: self.cells() as f32 + HANG,
                            size: 1.0,
                            stretch: 1.0,
                            half_w: 0.5,
                            ext: 1.0,
                            role: Role::Main,
                        };
                        self.place(u, slot, true);
                        continue;
                    }
                    self.ensure_room();
                    let slot = Slot {
                        col: self.col,
                        x: 0.0,
                        y: self.row as f32 + 0.5,
                        size: 1.0,
                        stretch: 1.0,
                        half_w: 0.5,
                        ext: 1.0,
                        role: Role::Main,
                    };
                    self.place(u, slot, true);
                    self.row += 1;
                }
                self.pending.extend(g.tail);
            }
            Item::Single(toks) => {
                let g = group(toks);
                self.lead(g.lead);
                for u in g.units {
                    self.ensure_room();
                    let slot = Slot {
                        col: self.col,
                        x: 0.0,
                        y: self.row as f32 + 0.5,
                        size: SINGLE_SIZE,
                        stretch: 1.0,
                        half_w: SINGLE_SIZE / 2.0,
                        ext: 1.0,
                        role: Role::Note,
                    };
                    self.place(u, slot, true);
                    self.row += 1;
                }
            }
            Item::Half { toks, left } => {
                let g = group(toks);
                self.lead(g.lead);
                for u in g.units {
                    self.ensure_room();
                    let slot = Slot {
                        col: self.col,
                        x: if left { NOTE_L } else { NOTE_R },
                        y: self.row as f32 + 0.5 + NOTE_DROP,
                        size: NOTE_SIZE,
                        stretch: NOTE_STRETCH,
                        half_w: NOTE_SIZE / 2.0,
                        ext: 1.0,
                        role: Role::Note,
                    };
                    self.place(u, slot, true);
                    self.row += 1;
                }
                self.pending.extend(g.tail);
            }
            Item::Span { toks, prev, n, at } => self.span(toks, prev, i32::from(n), at),
            Item::Double(toks) => self.double(toks),
            Item::Side { toks, left } => self.side(toks, left),
            Item::Head(toks) => self.head(toks),
            Item::Break { .. } => unreachable!("breaks are split out in paragraph()"),
        }
    }

    fn ensure_room(&mut self) {
        loop {
            if self.row >= self.cells() {
                self.col += 1;
                self.row = self.cont;
            } else if let Some(end) = self.reserved_end(self.col, self.row) {
                self.row = end;
            } else {
                break;
            }
        }
        self.used = true;
    }

    /// 若 (col, row) 已被兼行佔去，返回佔到哪一格为止。
    fn reserved_end(&self, col: i32, row: i32) -> Option<i32> {
        self.reserved
            .get(&col)?
            .iter()
            .find(|(a, b)| (*a..*b).contains(&row))
            .map(|r| r.1)
    }

    /// 本行从 `row` 起，到哪一格之前都可以连续放字。
    fn room_until(&self, col: i32, row: i32) -> i32 {
        self.reserved
            .get(&col)
            .and_then(|v| v.iter().filter(|r| r.0 >= row).map(|r| r.0).min())
            .unwrap_or(self.cells())
    }

    /// 兼行：字排在本行与后（前）`n` 行的正中，佔去这几行的同样几格。
    fn span(&mut self, toks: Vec<Tok>, prev: bool, n: i32, at: Span) {
        let g = group(toks);
        self.lead(g.lead);
        let len = g.units.len() as i32;
        self.ensure_room();
        if self.row + len > self.room_until(self.col, self.row) && self.row > self.cont {
            self.col += 1;
            self.row = self.cont;
            self.ensure_room();
        }
        let start = self.row;
        let end = start + len;
        let shift = n as f32 * self.o.column_width / 2.0;
        let x = if prev { shift } else { -shift };
        let cols: Vec<i32> = if prev {
            (self.col - n..self.col).collect()
        } else {
            (self.col + 1..=self.col + n).collect()
        };
        let half = |c: i32| c.div_euclid(self.o.columns);
        if self.o.format == Format::Leaves && cols.iter().any(|&c| half(c) != half(self.col)) {
            self.diags.push(Diagnostic::warning(
                at,
                "兼行跨過了半葉的邊界（版心或版框），請換一行再寫",
            ));
        }
        if prev {
            let clash = self.glyphs.iter().any(|gl| {
                cols.contains(&gl.col)
                    && !gl.hidden
                    && gl.y > start as f32
                    && gl.y < end as f32
                    && matches!(gl.role, Role::Main | Role::Note | Role::Punct)
            });
            if clash {
                self.diags.push(Diagnostic::warning(
                    at,
                    "兼行與前面的行重疊：前行在這幾格已經有字",
                ));
            }
        }
        for c in cols {
            self.reserved.entry(c).or_default().push((start, end));
        }
        self.decos.push(Deco::Gap {
            col: self.col,
            x,
            half_w: 0.5,
            y0: start as f32,
            y1: end as f32,
        });
        for u in g.units {
            let slot = Slot {
                col: self.col,
                x,
                y: self.row as f32 + 0.5,
                size: 1.0,
                stretch: 1.0,
                half_w: 0.5,
                ext: 1.0,
                role: Role::Main,
            };
            self.place(u, slot, true);
            self.row += 1;
        }
        self.pending.extend(g.tail);
    }

    /// 雙行夾注：每格两个小字，先右后左；最后一段左右平分。
    fn double(&mut self, toks: Vec<Tok>) {
        let g = group(toks);
        self.lead(g.lead);
        let mut rest: Vec<Unit> = g.units;
        while !rest.is_empty() {
            self.ensure_room();
            let avail = (self.room_until(self.col, self.row) - self.row) as usize;
            let need = rest.len().div_ceil(2);
            let k = need.min(avail);
            let mut take = (2 * k).min(rest.len());
            // 行首禁則：剩下的若只是一两个標點，挂在本行末尾。
            let hang = need > avail
                && rest.len() - take <= 2
                && rest[take..].iter().all(|u| is_hanging(&u.cell));
            if hang {
                take = rest.len();
            }
            let right_n = if need <= avail { take.div_ceil(2) } else { k };
            let tail = rest.split_off(take);
            for (i, u) in rest.into_iter().enumerate() {
                let (offset, x) = if hang && i >= 2 * k {
                    (k, if i == 2 * k { NOTE_R } else { NOTE_L })
                } else if i < right_n {
                    (i, NOTE_R)
                } else {
                    (i - right_n, NOTE_L)
                };
                let y = if hang && i >= 2 * k {
                    self.cells() as f32 + HANG
                } else {
                    (self.row + offset as i32) as f32 + 0.5 + NOTE_DROP
                };
                let slot = Slot {
                    col: self.col,
                    x,
                    y,
                    size: NOTE_SIZE,
                    stretch: NOTE_STRETCH,
                    half_w: NOTE_SIZE / 2.0,
                    ext: 1.0,
                    role: Role::Note,
                };
                self.place(u, slot, true);
            }
            self.row += k as i32;
            rest = tail;
        }
        self.pending.extend(g.tail);
    }

    /// 旁行：沿行的右侧（或左侧）排小字，不佔正文字格。
    fn side(&mut self, toks: Vec<Tok>, left: bool) {
        let g = group(toks);
        let cells = self.cells() as f32;
        let (mut col, mut y) = if self.row >= self.cells() {
            (self.col + 1, self.cont as f32)
        } else {
            (self.col, self.row.max(0) as f32)
        };
        let slot_i = usize::from(left);
        let (ec, ey) = self.side_end[slot_i];
        if ec == col && y < ey {
            y = ey;
        }
        let x = (0.5 + SIDE_SIZE * 0.35) * if left { -1.0 } else { 1.0 };
        for u in g.units {
            if y + SIDE_STEP > cells + 0.01 {
                col += 1;
                y = self.cont as f32;
            }
            let slot = Slot {
                col,
                x,
                y: y + SIDE_STEP / 2.0,
                size: SIDE_SIZE,
                stretch: 1.0,
                half_w: SIDE_SIZE / 2.0,
                ext: SIDE_STEP,
                role: Role::Side,
            };
            self.place(u, slot, false);
            y += SIDE_STEP;
        }
        self.side_end[slot_i] = (col, y);
    }

    /// 眉批：排在天頭，从当前行上方开始向左排小字。
    fn head(&mut self, toks: Vec<Tok>) {
        let g = group(toks);
        let cw = self.o.column_width;
        let cap = (((self.o.head - 0.3) / HEAD_STEP).floor() as usize).max(1);
        let step = HEAD_STEP * 1.1 / cw;
        let mut u = (self.col as f32 - 0.5 + step / 2.0).max(self.head_next);
        for (i, unit) in g.units.into_iter().enumerate() {
            let k = i % cap;
            if i > 0 && k == 0 {
                u += step;
            }
            let col = u.round() as i32;
            let slot = Slot {
                col,
                x: (col as f32 - u) * cw,
                y: -self.o.head + 0.15 + (k as f32 + 0.5) * HEAD_STEP,
                size: HEAD_SIZE,
                stretch: 1.0,
                half_w: HEAD_SIZE / 2.0,
                ext: HEAD_STEP,
                role: Role::Head,
            };
            self.place(unit, slot, false);
        }
        self.head_next = u + step * 1.6;
    }

    /// 开头的後附標點：附到上一个字；没有上一个字就各佔一格。
    fn lead(&mut self, toks: Vec<Tok>) {
        for t in toks {
            match self.last {
                Some(a) => self.post(&t, a),
                None if t.hidden => {}
                None => {
                    self.ensure_room();
                    let mut cell = t;
                    cell.kind = TokKind::Cell;
                    let slot = Slot {
                        col: self.col,
                        x: 0.0,
                        y: self.row as f32 + 0.5,
                        size: 1.0,
                        stretch: 1.0,
                        half_w: 0.5,
                        ext: 1.0,
                        role: Role::Main,
                    };
                    self.place(
                        Unit {
                            pre: Vec::new(),
                            cell,
                            post: Vec::new(),
                        },
                        slot,
                        false,
                    );
                    self.row += 1;
                }
            }
        }
    }

    fn place(&mut self, unit: Unit, s: Slot, take_pending: bool) {
        let mut pre = if take_pending {
            std::mem::take(&mut self.pending)
        } else {
            Vec::new()
        };
        pre.extend(unit.pre);
        // 前引號先于字输出，这样 SVG 里文字的先后就是阅读顺序，复制出来不乱。
        for (i, p) in pre.iter().enumerate() {
            // 前引號：字的右上角，几个就往下错开。
            // 前引號（┐）：照原版，墨迹中心在字心右 0.30、上 0.33。
            let (x, y) = (
                s.x + 0.3 * s.size,
                s.y - 0.33 * s.size + i as f32 * s.size * 0.12,
            );
            self.punct(p, s.col, x, y, s.size);
        }
        let placed = self.put(&unit.cell, s);
        let mut anchor = Anchor {
            col: s.col,
            x: s.x,
            y: s.y,
            size: s.size,
            posts: 0,
        };
        for p in &unit.post {
            self.post(p, anchor);
            anchor.posts += 1;
        }
        if placed {
            self.last = Some(anchor);
        }
    }

    fn post(&mut self, t: &Tok, a: Anchor) {
        // 後引號在字的左下角；句讀在字的右侧、贴着字格下沿，几个就往下错开。
        // 句讀墨迹中心的位置，照原版实测：正文在字心右 0.65、下 0.33；
        // 夾注小字在右 0.2、下 0.54。後引號（└）在左下：左 0.25、下 0.32。
        let close = t.text.chars().next().is_some_and(is_close_quote);
        let note = a.size < 0.9;
        let (x, y) = if close {
            (a.x - 0.25 * a.size, a.y + 0.32 * a.size)
        } else {
            let (dx, dy) = if note { (0.2, 0.542) } else { (0.65, 0.33) };
            (a.x + dx, a.y + dy + a.posts as f32 * 0.22)
        };
        self.punct(t, a.col, x, y, a.size);
        if let Some(last) = &mut self.last
            && last.col == a.col
            && last.y == a.y
            && last.x == a.x
        {
            last.posts = a.posts + 1;
        }
    }

    fn punct(&mut self, t: &Tok, col: i32, x: f32, y: f32, base: f32) {
        self.glyphs.push(Glyph {
            col,
            x,
            y,
            size: base,
            stretch: 1.0,
            text: t.text.clone(),
            compose: false,
            color: Some(t.style.color.unwrap_or(self.o.punct_color)),
            inverse: false,
            hidden: t.hidden,
            role: Role::Punct,
            bold: false,
        });
        self.max_col = self.max_col.max(col);
    }

    /// 放一个字。返回是否真的画了字（空格不画）。
    fn put(&mut self, t: &Tok, s: Slot) -> bool {
        let mut inverse = false;
        for m in &t.style.marks {
            match m.kind {
                MarkKind::Emphasis(mark) => self.decos.push(Deco::Mark {
                    col: s.col,
                    x: s.x + s.half_w + s.size * 0.14,
                    y: s.y,
                    size: s.size,
                    mark,
                    color: Some(self.o.punct_color),
                }),
                MarkKind::Inverse(e) | MarkKind::Frame(e) if e.shape.is_each() => {
                    let filled = matches!(m.kind, MarkKind::Inverse(_));
                    inverse |= filled;
                    let lantern = e.shape == Shape::Lantern;
                    let r = if lantern {
                        s.ext * 0.5 * s.stretch.max(1.0)
                    } else {
                        s.half_w.max(s.ext * 0.5 * s.size) * 0.98
                    };
                    self.decos.push(Deco::Ring {
                        col: s.col,
                        x: s.x,
                        y: s.y,
                        r,
                        filled,
                        hex: lantern,
                    });
                }
                kind => {
                    inverse |= matches!(kind, MarkKind::Inverse(_));
                    self.extend_seg(m.id, kind, s);
                }
            }
        }
        self.max_col = self.max_col.max(s.col);
        if t.kind == TokKind::Space {
            return false;
        }
        if let Some(tone) = t.tone {
            let (dx, dy) = tone.corner();
            self.decos.push(Deco::Tone {
                col: s.col,
                x: s.x + dx * s.half_w * 0.95,
                y: s.y + dy * s.ext * 0.42,
                r: s.size * 0.075,
            });
        }
        self.glyphs.push(Glyph {
            col: s.col,
            x: s.x,
            y: s.y,
            size: s.size,
            stretch: s.stretch,
            text: t.text.clone(),
            compose: t.compose,
            color: t.style.color,
            inverse,
            hidden: false,
            role: s.role,
            bold: false,
        });
        true
    }

    fn extend_seg(&mut self, id: u32, kind: MarkKind, s: Slot) {
        let key = (id, s.col, (s.x * 1000.0).round() as i32);
        let (y0, y1) = (s.y - s.ext / 2.0, s.y + s.ext / 2.0);
        match self.seg_at.get(&key) {
            Some(&i) => {
                let seg = &mut self.segs[i];
                seg.y0 = seg.y0.min(y0);
                seg.y1 = seg.y1.max(y1);
            }
            None => {
                self.seg_at.insert(key, self.segs.len());
                self.segs.push(Seg {
                    kind,
                    col: s.col,
                    x: s.x,
                    half_w: s.half_w,
                    y0,
                    y1,
                });
            }
        }
    }

    fn finish(mut self) -> Layout {
        for s in std::mem::take(&mut self.segs) {
            let deco = match s.kind {
                // 專名號：照原版，朱色粗綫，在字心左 0.485，上缩 0.175、下缩 0.215。
                MarkKind::Name | MarkKind::Title => Deco::Line {
                    col: s.col,
                    x: s.x - s.half_w * 0.97,
                    y0: s.y0 + 0.175 * s.half_w * 2.0,
                    y1: s.y1 - 0.215 * s.half_w * 2.0,
                    wavy: s.kind == MarkKind::Title,
                    color: Some(self.o.punct_color),
                    width: if s.kind == MarkKind::Title {
                        0.03
                    } else {
                        0.098 * s.half_w * 2.0
                    },
                },
                MarkKind::Stroke => Deco::Line {
                    col: s.col,
                    x: s.x + s.half_w + LINE_GAP,
                    y0: s.y0 + 0.04,
                    y1: s.y1 - 0.04,
                    wavy: false,
                    color: Some(Color::Red),
                    width: 0.035,
                },
                MarkKind::Delete => Deco::Line {
                    col: s.col,
                    x: s.x,
                    y0: s.y0 + 0.1,
                    y1: s.y1 - 0.1,
                    wavy: false,
                    color: Some(Color::Red),
                    width: 0.035,
                },
                MarkKind::Inverse(e) | MarkKind::Frame(e) => {
                    // 正文的框默认贴着界行；窄式和小字的框只包住字。
                    let wide = !e.narrow && s.x.abs() < 1e-3 && s.half_w >= 0.5;
                    Deco::Box {
                        col: s.col,
                        x: s.x,
                        half_w: if wide {
                            self.o.column_width / 2.0 - 0.03
                        } else {
                            s.half_w + 0.04
                        },
                        y0: s.y0 + 0.02,
                        y1: s.y1 - 0.02,
                        shape: e.shape,
                        filled: matches!(s.kind, MarkKind::Inverse(_)),
                    }
                }
                MarkKind::Emphasis(_) => continue,
            };
            self.decos.push(deco);
        }
        let last_col = if self.used { self.col } else { self.col - 1 };
        Layout {
            columns: self.max_col.max(last_col) + 1,
            glyphs: self.glyphs,
            decos: self.decos,
            options: self.o,
        }
    }
}

fn is_close_quote(c: char) -> bool {
    matches!(c, '」' | '』' | '”' | '’' | '﹂' | '﹄')
}

/// 目次里只留文字，不带标记。
fn plain(t: &Tok) -> Tok {
    Tok {
        style: Default::default(),
        tone: None,
        ..t.clone()
    }
}

/// 一段要佔多少格（用于靠底排）。
fn measure(items: &[Item]) -> i32 {
    let cells = |t: &[Tok]| {
        t.iter()
            .filter(|t| matches!(t.kind, TokKind::Cell | TokKind::Space))
            .count() as i32
    };
    items
        .iter()
        .map(|i| match i {
            Item::Main(t)
            | Item::Single(t)
            | Item::Half { toks: t, .. }
            | Item::Span { toks: t, .. } => cells(t),
            Item::Double(t) => (cells(t) + 1) / 2,
            _ => 0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use crate::layout::{Options, Role, layout};
    use crate::syntax::parse;

    fn run(src: &str, cells: i32) -> crate::layout::Layout {
        let parsed = parse(src);
        let mut d = parsed.diagnostics;
        let o = Options {
            cells,
            ..Options::default()
        };
        let l = layout(&parsed.document, o, &mut d);
        assert!(d.is_empty(), "{d:?}");
        l
    }

    fn at(l: &crate::layout::Layout, role: Role) -> Vec<(i32, i32, i32, String)> {
        l.glyphs
            .iter()
            .filter(|g| g.role == role)
            .map(|g| {
                (
                    g.col,
                    (g.x * 100.0).round() as i32,
                    (g.y * 2.0).round() as i32,
                    g.text.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn main_text_wraps() {
        let l = run("甲乙丙丁戊", 3);
        let g = at(&l, Role::Main);
        assert_eq!(
            g.iter().map(|g| (g.0, g.2)).collect::<Vec<_>>(),
            [(0, 1), (0, 3), (0, 5), (1, 1), (1, 3)]
        );
        assert_eq!(l.columns, 2);
    }

    #[test]
    fn double_note_balances_last_segment() {
        // 一行 4 格：正文 1 格，夾注 9 字 → 本行 3 格放 6 字（右 3 左 3），下一行 3 字（右 2 左 1）。
        let l = run(r"甲\note{一二三四五六七八九}乙", 4);
        let n = at(&l, Role::Note);
        let right: String = n.iter().filter(|g| g.1 > 0).map(|g| g.3.as_str()).collect();
        let left: String = n.iter().filter(|g| g.1 < 0).map(|g| g.3.as_str()).collect();
        assert_eq!(right, "一二三七八");
        assert_eq!(left, "四五六九");
        let col1: Vec<_> = n
            .iter()
            .filter(|g| g.0 == 1)
            .map(|g| (g.1 > 0, g.2))
            .collect();
        assert_eq!(col1, [(true, 1), (true, 3), (false, 1)]);
        let main = at(&l, Role::Main);
        assert_eq!((main[1].0, main[1].2), (1, 5), "正文接在夾注之后");
    }

    #[test]
    fn indent_and_raise() {
        let l = run(r"\indent[2] \raise[1] 甲乙丙丁", 4);
        let g = at(&l, Role::Main);
        assert_eq!(
            g.iter().map(|g| (g.0, g.2)).collect::<Vec<_>>(),
            [(0, 3), (0, 5), (0, 7), (1, 5)]
        );
    }

    #[test]
    fn bottom_aligned() {
        let l = run(r"\bottom[1] 甲乙", 6);
        let g = at(&l, Role::Main);
        assert_eq!(g.iter().map(|g| g.2).collect::<Vec<_>>(), [7, 9]);
    }

    #[test]
    fn paragraphs_lines_and_blanks() {
        let l = run("甲\n乙\n\n丙\n\n\n丁", 10);
        let g = at(&l, Role::Main);
        assert_eq!(g.iter().map(|g| g.0).collect::<Vec<_>>(), [0, 1, 2, 4]);
    }

    #[test]
    fn punctuation_attaches_without_cells() {
        let l = run("曰：「甲。」乙", 10);
        let main = at(&l, Role::Main);
        assert_eq!(main.iter().map(|g| g.2).collect::<Vec<_>>(), [1, 3, 5]);
        assert_eq!(at(&l, Role::Punct).len(), 4);
    }

    #[test]
    fn marks_become_one_line_per_column() {
        let l = run(r"\name{甲乙丙}", 2);
        let lines = l
            .decos
            .iter()
            .filter(|d| matches!(d, crate::layout::Deco::Line { .. }))
            .count();
        assert_eq!(lines, 2);
    }

    #[test]
    fn grid_punctuation_hangs_instead_of_starting_a_column() {
        let parsed = parse(r"甲乙丙。丁\note{一二三四五六七八。}");
        let mut d = parsed.diagnostics;
        let o = Options {
            cells: 3,
            punct: crate::layout::PunctStyle::Grid,
            ..Options::default()
        };
        let l = layout(&parsed.document, o, &mut d);
        let main = at(&l, Role::Main);
        assert_eq!(
            (main[3].0, main[3].3.as_str()),
            (0, "︒"),
            "句號挂在第一行末"
        );
        assert_eq!((main[4].0, main[4].2), (1, 1), "丁另起一行");
        let notes = at(&l, Role::Note);
        let last = notes.last().unwrap();
        assert_eq!(
            (last.0, last.3.as_str()),
            (2, "︒"),
            "夾注末的句號不單獨佔一行"
        );
        assert_eq!(l.columns, 3);

        // 夾注排满一行后只剩一个句號：挂在本行末，不另起一行。
        let parsed = parse(r"丁\note{一二三四。}");
        let l = layout(
            &parsed.document,
            Options {
                cells: 3,
                punct: crate::layout::PunctStyle::Grid,
                ..Options::default()
            },
            &mut d,
        );
        let last = l.glyphs.last().unwrap();
        assert_eq!((last.col, last.text.as_str()), (0, "︒"));
        assert!(last.y > 3.0 && last.x > 0.0);
        assert_eq!(l.columns, 1);
    }

    #[test]
    fn glyphs_are_in_reading_order() {
        let l = run(r"曰：「天\note{繫，戶計翻。}地。」", 10);
        let text: String = l.glyphs.iter().map(|g| g.text.as_str()).collect();
        assert_eq!(text, "曰：﹁天繫，戶計翻。地。﹂");
    }

    #[test]
    fn head_note_goes_above_frame() {
        let l = run(r"甲\head{眉批文字}乙", 10);
        assert!(at(&l, Role::Head).iter().all(|g| g.2 < 0));
    }
}

#[cfg(test)]
mod container_tests {
    use crate::layout::{Options, layout};
    use crate::syntax::parse;

    #[test]
    fn container_indent() {
        let parsed = parse("\\begin{indent}[2]\n甲\n\n\\indent[1] 乙\n\\end{indent}\n丙");
        let mut d = parsed.diagnostics;
        let l = layout(&parsed.document, Options::default(), &mut d);
        let ys: Vec<f32> = l.glyphs.iter().map(|g| g.y).collect();
        assert_eq!(
            ys,
            [2.5, 3.5, 0.5],
            "容器低二格，段首再低一格累加，容器外不低"
        );
    }
}

#[cfg(test)]
mod feature_tests {
    use crate::layout::{Deco, Options, Role, layout};
    use crate::syntax::ast::Shape;
    use crate::syntax::parse;

    fn run(src: &str, o: Options) -> crate::layout::Layout {
        let parsed = parse(src);
        let mut d = parsed.diagnostics;
        let l = layout(&parsed.document, o, &mut d);
        assert!(d.is_empty(), "{d:?}");
        l
    }

    fn cells(n: i32) -> Options {
        Options {
            cells: n,
            ..Options::default()
        }
    }

    #[test]
    fn span_reserves_next_column() {
        // 「明算」跨到下一行：排在两行正中，下一行的这两格空出来。
        let l = run("甲\\span{明算}乙\n丙丁戊", cells(10));
        let g: Vec<(i32, f32, f32, &str)> = l
            .glyphs
            .iter()
            .map(|g| (g.col, g.x, g.y, g.text.as_str()))
            .collect();
        assert_eq!(g[1], (0, -0.8, 1.5, "明"));
        assert_eq!(g[2], (0, -0.8, 2.5, "算"));
        let next: Vec<f32> = g.iter().filter(|g| g.0 == 1).map(|g| g.2).collect();
        assert_eq!(next, [0.5, 3.5, 4.5], "丙在第一格，丁戊跳过被佔的两格");
    }

    #[test]
    fn half_note_takes_cells_on_one_side() {
        let l = run("甲\\note[left]{乙丙}丁", cells(10));
        let notes: Vec<(f32, f32)> = l
            .glyphs
            .iter()
            .filter(|g| g.role == Role::Note)
            .map(|g| (g.x, g.y))
            .collect();
        assert_eq!(notes, [(-0.35, 1.438), (-0.35, 2.438)]);
        assert_eq!(l.glyphs.last().unwrap().y, 3.5);
    }

    #[test]
    fn rods_alternate_vertical_and_horizontal() {
        let l = run("\\rod{1971}", cells(10));
        let text: String = l.glyphs.iter().map(|g| g.text.as_str()).collect();
        // 千位橫、百位縱、十位橫、個位縱。
        assert_eq!(text, "\u{1D369}\u{1D368}\u{1D36F}\u{1D360}");
        let l = run("\\rod{105}", cells(10));
        let text: String = l.glyphs.iter().map(|g| g.text.as_str()).collect();
        assert_eq!(text, "\u{1D360}〇\u{1D364}");
    }

    #[test]
    fn frames_touch_rules_unless_narrow() {
        let wide = run("\\frame{甲}", cells(10));
        let narrow = run("\\frame[narrow]{甲}", cells(10));
        let hw = |l: &crate::layout::Layout| match l.decos[0] {
            Deco::Box { half_w, .. } => half_w,
            _ => panic!(),
        };
        assert!(hw(&wide) > 0.7 && hw(&narrow) < 0.6);
        let lantern = run("\\inv[lantern]{甲乙}", cells(10));
        assert_eq!(
            lantern
                .decos
                .iter()
                .filter(|d| matches!(d, Deco::Ring { hex: true, .. }))
                .count(),
            2
        );
        let flower = run("\\frame[flower]{甲乙}", cells(10));
        assert!(matches!(
            flower.decos[0],
            Deco::Box {
                shape: Shape::Flower,
                ..
            }
        ));
    }

    #[test]
    fn toc_lists_headings_with_leaf_numbers() {
        // 每半葉 2 行、每葉 4 行：卷二落在第二葉。
        let o = Options {
            cells: 10,
            columns: 2,
            format: crate::layout::Format::Leaves,
            ..Options::default()
        };
        let l = run(
            "\\toc\n\n\\heading{卷一}\n\n甲\n\n乙\n\n\\heading{卷二}\n\n丙",
            o,
        );
        let col0: String = l
            .glyphs
            .iter()
            .filter(|g| g.col == 0)
            .map(|g| g.text.as_str())
            .collect();
        let col1: String = l
            .glyphs
            .iter()
            .filter(|g| g.col == 1)
            .map(|g| g.text.as_str())
            .collect();
        assert_eq!(col0, "卷一一");
        assert_eq!(col1, "卷二二");
        let last = l.glyphs.iter().rfind(|g| g.col == 1).unwrap();
        assert_eq!(last.y, 9.5, "葉碼靠底");
    }
}
