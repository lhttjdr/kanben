//! 排版器：按字格逐个放字。

use std::collections::HashMap;

use super::flatten::{Flattener, Item, Pic};
use super::token::{Ann, MarkKind, Tok, TokKind, Tone, Unit, group, is_hanging};
use super::{Deco, Figure, Format, Glyph, Layout, Options, Role};
use crate::render::numerals::hanzi;
use crate::syntax::ast::{
    AttrKind, Block, Color, Document, EmphMark, EnvKind, Inline, ParaStyle, Shape, TableHeader,
};
use crate::syntax::span::{Diagnostic, Span};

// 尺寸（单位：字）。
// 雙行夾注：0.8 字号（原版写了 75% 字宽，但常用字体没有窄体，实际渲染就是 0.8 见方），
// 左右两小行的中线在 +0.45 / -0.35，字盒中心比所在格低 0.08。这些都是照原版实测的。
/// 正文字盒的中心比行中綫偏右 1px（原版每行左邊有 1px 界綫）。
const MAIN_X: f32 = 1.0 / 30.0;
/// 陰文、墨圍的字（原版 cite，南監體、方正金陵、仿宋）的內容區相對字心的左右寬（字）。
/// 原版的底色、邊綫都從內容區量起；這是在比對環境裏量得的（隨字體而變）。
const ENCL_L: f32 = 17.0 / 30.0;
const ENCL_R: f32 = 18.0 / 30.0;
/// 圓形、疊燈籠的圓心（底圖從內容區左沿起鋪，圓心在左沿右 0.5 字）。
const ENCL_DX: f32 = 0.5 - ENCL_L;
const NOTE_SIZE: f32 = 0.8;
/// 琵琶譜字的字身高（像素，正文 30px）：字體沒有竪排字寬，瀏覽器取 ascent + descent，各取整。
const PIPA_PX: i32 = 34;
/// 琵琶譜裏單側行一格的高。
const PIPA_LINE_PX: f32 = 33.0;
/// 韻部的顏色（原版 --韵抹0 至 --韵抹9；不標韻部用桃紅）。
const RHYME_COLORS: [&str; 10] = [
    "#9ed900", "#b9dec9", "#f0ada0", "#93d5dc", "#fcb70a", "#d1c2d3", "#f97d1c", "#d2d97a",
    "#c6e6e8", "#e3bd8d",
];
/// 整行插圖寬幾行（原版 figure：行寬 × 9）。
const FIGURE_COLS: i32 = 9;
/// 浮動插圖寬幾行、高幾格（原版 fig-il：16em × 8em）。
const FLOAT_COLS: i32 = 10;
const FLOAT_ROWS: i32 = 8;
/// 字樣字圖邊長（48px）。
const GLYPH_IMG: f32 = 1.6;
/// 手寫目次的字色（原版鏈接色，正青）。
const TOC_COLOR: &str = "rgb(16.27,51.37,147.9)";
/// 集（分段單兒）段首小籤的半寬：字的內容區 15px 加左右內距 6px。
const LIST_TAB_HALF_W: f32 = 21.0 / 30.0;
const NOTE_STRETCH: f32 = 1.0;
const NOTE_R: f32 = 0.45;
const NOTE_L: f32 = -0.35;
const NOTE_DROP: f32 = 0.08;
/// 單行小注：原版 28px（正文 30px）。
const SINGLE_SIZE: f32 = 28.0 / 30.0;
/// 句段旁注：原版 15px（正文一半），逐字相接，排在行的右沿（字盒中心在行中綫右 0.85）。
const SIDE_SIZE: f32 = 0.5;
const SIDE_STEP: f32 = 0.5;
const SIDE_X: f32 = 0.85;
/// 眉批照原版 #tentou：行高為字號的 1.4 倍（字號默認 0.75，每行寬 1.05）。
const HEAD_LEADING: f32 = 1.4;
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
    fn add(&mut self, a: &AttrKind) {
        match *a {
            AttrKind::Indent(n) => self.indent += i32::from(n),
            AttrKind::Raise(n) => self.raise = self.raise.max(i32::from(n)),
            AttrKind::Bottom(n) => self.bottom = Some(i32::from(n)),
            // 原版：撰者靠底，下留二格（padding-bottom: 2em）；目整段下移一格（margin-top: 1em）。
            AttrKind::Style(ParaStyle::Byline) => {
                self.bottom = Some(self.bottom.unwrap_or(0) + 2);
            }
            AttrKind::Style(ParaStyle::Detail) => self.indent += 1,
            // TODO(SVG)：單兒還沒有照原版排。
            AttrKind::Style(_) | AttrKind::List { .. } | AttrKind::Colors(_) => {}
        }
    }

    /// 行首属性：覆盖同类的段落属性。
    fn replace(&mut self, a: &AttrKind) {
        match *a {
            AttrKind::Indent(n) => self.indent = i32::from(n),
            AttrKind::Raise(n) => self.raise = i32::from(n),
            AttrKind::Bottom(n) => self.bottom = Some(i32::from(n)),
            AttrKind::Style(_) | AttrKind::List { .. } | AttrKind::Colors(_) => {}
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
    id: u32,
    kind: MarkKind,
    /// 段內各字（書名波綫要用）。
    text: String,
    col: i32,
    x: f32,
    half_w: f32,
    y0: f32,
    y1: f32,
    /// 所標的字是正文、夾注還是眉批（書名波綫畫法不同）。
    role: Role,
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
    /// 眉批最左到第幾行。
    head_col: i32,
    /// 兼行佔去的格：行 → [起, 止)。
    reserved: HashMap<i32, Vec<(i32, i32)>>,
    /// 目次内容（由上一遍排版得出）。
    toc: Vec<TocEntry>,
    /// 本段的尾注，段後列出。
    endnotes: Vec<(Option<String>, Vec<Tok>)>,
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
            endnotes: Vec::new(),
            side_end: [(-1, 0.0); 2],
            max_col: 0,
            head_col: 0,
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
        self.blocks(&doc.blocks, LineFmt::default(), &mut heads, &mut after_toc);
        (self.finish(), heads)
    }

    fn blocks(
        &mut self,
        blocks: &[Block],
        outer: LineFmt,
        heads: &mut Vec<(TocEntry, i32)>,
        after_toc: &mut bool,
    ) {
        for block in blocks {
            match block {
                Block::Heading { level, content, .. } => {
                    // 原版：n 級標題低 n-1 格。
                    let indent = i32::from(*level) - 1;
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
                        if g.role == Role::Main {
                            g.level = *level;
                        }
                    }
                    let col = self.glyphs.get(before).map_or(self.col, |g| g.col);
                    if *after_toc {
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
                    let mut base = outer;
                    for a in attrs {
                        base.add(&a.kind);
                    }
                    let lines: Vec<(LineFmt, Vec<Item>)> = lines
                        .iter()
                        .enumerate()
                        .map(|(k, l)| {
                            let mut fmt = base;
                            if k > 0 {
                                fmt.raise = 0;
                            }
                            for a in &l.attrs {
                                fmt.replace(&a.kind);
                            }
                            (fmt, self.flatten(&l.content))
                        })
                        .collect();
                    // 整段只有一幅插圖：佔整行（原版 figure）。
                    if let [(_, items)] = lines.as_slice()
                        && let [Item::Figure { images }] = items.as_slice()
                    {
                        let images = images.clone();
                        self.block_figure(&images);
                        continue;
                    }
                    // 單兒：每行一條，照原版（flex，每條至少三分之一行高）。
                    if attrs
                        .iter()
                        .any(|a| matches!(a.kind, AttrKind::List { .. }))
                    {
                        let items: Vec<Vec<Item>> =
                            lines.into_iter().map(|(_, items)| items).collect();
                        self.flex(items, 0.333);
                        continue;
                    }
                    let before = self.glyphs.len();
                    let deco_before = self.decos.len();
                    // 琵琶譜字的字身比一字高（34px / 30px），一行排得少些，再把各格拉開。
                    let pipa = attrs
                        .iter()
                        .any(|a| a.kind == AttrKind::Style(ParaStyle::Pipa));
                    let saved = self.o.cells;
                    if pipa {
                        self.o.cells = (saved * 30 / PIPA_PX).max(1);
                    }
                    self.paragraph(lines);
                    self.widen_pitch(before, deco_before);
                    self.endnote_list();
                    if pipa {
                        self.o.cells = saved;
                        self.stretch_rows(before, deco_before);
                    }
                    for a in attrs {
                        match &a.kind {
                            AttrKind::Colors(names) => self.paint(before, names),
                            AttrKind::Style(s) => {
                                let class = match s {
                                    ParaStyle::Byline => "g-byline",
                                    ParaStyle::Outline => "g-gang",
                                    ParaStyle::Detail => "g-mu",
                                    ParaStyle::Remark => "g-an",
                                    ParaStyle::Pipa => "g-pipa",
                                };
                                for g in &mut self.glyphs[before..] {
                                    if g.role == Role::Main && g.level == 0 && !g.encl {
                                        g.extra = Some(class);
                                    }
                                    // 琵琶譜裏的單側行也用譜字字體。
                                    if g.role == Role::Line && *s == ParaStyle::Pipa {
                                        g.extra = Some("g-pipa-line");
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Block::Blank { count, .. } => {
                    self.start_block();
                    self.col += *count as i32;
                }
                Block::Toc { depth, entries, .. } => {
                    *after_toc = true;
                    if entries.is_empty() {
                        self.toc(*depth);
                    } else {
                        // 手寫的目次：照原版（flex，每條至少半行高），短的兩條共一行；字用正青。
                        let items: Vec<Vec<Item>> =
                            entries.iter().map(|e| self.flatten(&e.content)).collect();
                        let before = self.glyphs.len();
                        self.flex(items, 0.5);
                        for g in &mut self.glyphs[before..] {
                            if g.role != Role::Punct {
                                g.paint = Some(TOC_COLOR);
                                if g.role == Role::Main {
                                    g.extra = Some("g-toc");
                                }
                            }
                        }
                    }
                }
                Block::Env {
                    kind: EnvKind::List { ordered: false },
                    blocks,
                    ..
                } => {
                    // 分段單兒（集）：每條段首上方一塊絲欄色的小籤（原版 li::before）。
                    for b in blocks {
                        let before = self.glyphs.len();
                        self.blocks(std::slice::from_ref(b), outer, heads, after_toc);
                        if matches!(b, Block::Paragraph { .. })
                            && let Some(col) = self.glyphs[before..].iter().map(|g| g.col).min()
                        {
                            self.shape(
                                col,
                                Figure::Rect {
                                    x0: -LIST_TAB_HALF_W,
                                    y0: -0.5,
                                    x1: LIST_TAB_HALF_W,
                                    y1: 0.0,
                                    rx: 0.0,
                                    ry: 0.0,
                                },
                                "f-tab crisp",
                            );
                        }
                    }
                }
                Block::Env {
                    kind: EnvKind::Table { title, header },
                    blocks,
                    ..
                } => self.table(title, *header, blocks),
                Block::Env { kind, blocks, .. } => {
                    // 引文：原版 blockquote 上空二格（margin-top: 2em）。
                    let mut fmt = outer;
                    if *kind == EnvKind::Quote {
                        fmt.indent += 2;
                    }
                    self.blocks(blocks, fmt, heads, after_toc);
                }
            }
        }
    }

    /// 表格（原版 table，竪排）：標題一行（上空一格），表的每一列各佔一行；
    /// 各格長按該欄最長的字數，首格下留半格，其餘上下各留半格；每格之後一條 1px 淡朱綫。
    fn table(&mut self, title: &str, header: TableHeader, blocks: &[Block]) {
        let mut rows: Vec<Vec<Vec<Item>>> = Vec::new();
        for b in blocks {
            if let Block::Paragraph { lines, .. } = b {
                for l in lines {
                    let items = self.flatten(&l.content);
                    rows.push(split_cells(items));
                }
            }
        }
        let ncol = rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut h = vec![0.0f32; ncol];
        for row in &rows {
            for (k, cell) in row.iter().enumerate() {
                let pad = if k == 0 { 0.5 } else { 1.0 };
                h[k] = h[k].max(measure(cell) as f32 + pad);
            }
        }
        let total: f32 = h.iter().sum();
        let cw = self.o.column_width;
        let rule = |e: &mut Self, col: i32, y: f32| {
            e.shape(
                col,
                Figure::Rect {
                    x0: -cw / 2.0,
                    y0: y,
                    x1: cw / 2.0,
                    y1: y + 1.0 / 30.0,
                    rx: 0.0,
                    ry: 0.0,
                },
                "f-tabrule",
            );
        };
        self.start_block();
        let place = |e: &mut Self, items: Vec<Item>, pos: f32, class: &'static str| {
            let g0 = e.glyphs.len();
            let d0 = e.decos.len();
            e.row = pos.floor() as i32;
            e.cont = 0;
            e.pending.clear();
            e.last = None;
            for it in items {
                e.item(it);
            }
            let dy = pos.fract();
            if dy > 0.0 {
                e.remap_y(g0, d0, |_, y| y + dy);
            }
            for g in &mut e.glyphs[g0..] {
                if g.role == Role::Main && !g.encl {
                    g.extra = Some(class);
                }
            }
            e.used = true;
        };
        if !title.is_empty() {
            let items = self.flatten(&[Inline::Text {
                text: title.to_string(),
                span: Span::default(),
            }]);
            place(self, items, 1.0, "g-th");
            let col = self.col;
            rule(self, col, total);
            self.new_column();
        }
        for (r, row) in rows.into_iter().enumerate() {
            let mut y = 0.0;
            for (k, cell) in row.into_iter().enumerate() {
                let th = match header {
                    TableHeader::None => false,
                    TableHeader::Row => r == 0,
                    TableHeader::Column => k == 0,
                    TableHeader::Both => r == 0 || k == 0,
                };
                let pad = if k == 0 { 0.0 } else { 0.5 };
                place(self, cell, y + pad, if th { "g-th" } else { "g-td" });
                y += h[k];
                let col = self.col;
                rule(self, col, y);
            }
            self.used = true;
            self.new_column();
        }
    }

    /// 整行的插圖（原版 figure）：寬九行，從字頂上 9px 到行底，紙色底、左右 1px 絲欄色邊，
    /// 圖按比例放在框內。
    fn block_figure(&mut self, images: &[Pic]) {
        self.start_block();
        let col = self.col;
        let cw = self.o.column_width;
        let (x1, x0) = (cw / 2.0, cw / 2.0 - FIGURE_COLS as f32 * cw);
        let (y0, y1) = (-9.0 * PX, self.cells() as f32 + 0.2);
        self.figure_box(col, x0, x1, y0, y1, "f-figure");
        for (x0, x1) in [(x0, x0 + PX), (x1 - PX, x1)] {
            self.shape(col, rect_fig(x0, x1, y0, y1), "f-tab crisp");
        }
        if let [img] = images {
            self.decos.push(Deco::Image {
                col,
                x0: x0 + PX,
                y0,
                x1: x1 - PX,
                y1,
                href: img.href.clone(),
                fallback: None,
                embed: img.embed.clone(),
            });
        }
        self.col += FIGURE_COLS - 1;
        self.used = true;
    }

    /// 不佔整行的插圖（原版 fig-il，向上浮）：寬十行、高八格，佔去這幾行的上八格，
    /// 本行已排的字一併挪到圖下；右、下 1px 絲欄色邊。各圖自右而左排，高一格框高。
    fn float_figure(&mut self, images: &[Pic]) {
        let col = self.col;
        let cw = self.o.column_width;
        let h = FLOAT_ROWS;
        // 本行已排的字挪到圖下。
        let here: Vec<usize> = (0..self.glyphs.len())
            .filter(|&i| self.glyphs[i].col == col)
            .collect();
        let d0 = self.decos.len();
        for i in here {
            self.glyphs[i].y += h as f32;
        }
        for d in &mut self.decos[..d0] {
            if d.col() == col {
                shift_deco(d, h as f32);
            }
        }
        self.row += h;
        for c in col..col + FLOAT_COLS {
            self.reserved.entry(c).or_default().push((0, h));
        }
        let (x1, x0) = (cw / 2.0, cw / 2.0 - FLOAT_COLS as f32 * cw);
        let (y0, y1) = (-9.0 * PX, h as f32 - 9.0 * PX);
        self.figure_box(col, x0, x1, y0, y1, "f-figure");
        self.shape(col, rect_fig(x1 - PX, x1, y0, y1), "f-tab crisp");
        self.shape(col, rect_fig(x0, x1, y1 - PX, y1), "f-tab crisp");
        self.shape(col, rect_fig(x0 - 0.5 * PX, x0, y0, y1), "f-tab");
        let side = y1 - y0 - PX;
        let mut right = x1 - PX;
        for img in images {
            self.decos.push(Deco::Image {
                col,
                x0: right - side,
                y0,
                x1: right,
                y1: y0 + side,
                href: img.href.clone(),
                fallback: None,
                embed: img.embed.clone(),
            });
            right -= side;
        }
    }

    fn figure_box(&mut self, col: i32, x0: f32, x1: f32, y0: f32, y1: f32, class: &'static str) {
        self.shape(col, rect_fig(x0, x1, y0, y1), class);
    }

    /// 字樣（原版 dfn.字樣）：佔兩格，1.6 字見方的字圖居中；字本身透明，只供複製。
    /// 照原版先取 圖/字.svg，取不到再取 .png。
    fn glyph_image(&mut self, c: &str) {
        if self.row + 2 > self.cells() {
            self.col += 1;
            self.row = self.cont;
        }
        self.ensure_room();
        let (col, y) = (self.col, self.row as f32 + 1.0);
        let half = GLYPH_IMG / 2.0;
        self.decos.push(Deco::Image {
            col,
            x0: -half,
            y0: y - half,
            x1: half,
            y1: y + half,
            href: format!("圖/{c}.svg"),
            fallback: Some(format!("圖/{c}.png")),
            embed: None,
        });
        self.glyphs.push(Glyph {
            col,
            x: MAIN_X,
            y,
            size: 1.0,
            stretch: 1.0,
            text: c.to_string(),
            compose: false,
            rod: false,
            color: None,
            inverse: false,
            hidden: true,
            faded: false,
            encl: false,
            role: Role::Main,
            level: 0,
            paint: None,
            extra: None,
        });
        self.row += 2;
        self.main_anchor_after_note();
    }

    /// 文中的尾注編號（原版 pCmtN）：不佔格，在前一字那格的下半、行中綫左 0.56 字，
    /// 淡綠圓角底、靛青字，半字大。
    fn endnote_marker(&mut self, n: usize) {
        let (col, r) = if self.row > 0 {
            (self.col, self.row as f32)
        } else {
            (self.col - 1, self.cells() as f32)
        };
        let xc = -16.75 / 30.0;
        let (w, h) = (21.0 / 30.0, 0.5);
        self.shape(
            col,
            Figure::Rect {
                x0: xc - w / 2.0,
                y0: r - h,
                x1: xc + w / 2.0,
                y1: r,
                rx: 0.15,
                ry: 0.15,
            },
            "f-pcmtn",
        );
        self.ann_glyph(
            col,
            xc,
            r - h / 2.0,
            0.5,
            crate::render::numerals::heji(n as u32),
            "g-pcmtn",
        );
    }

    /// 段後的校勘記（原版 ol.pCmnt）：先一行「校勘記」（上空一格），每條一行：
    /// 首格是加六角框的編號，其次所注之字（同單行小注），再是注文（同雙行夾注）；折行從第三格起。
    fn endnote_list(&mut self) {
        if self.endnotes.is_empty() {
            return;
        }
        let notes = std::mem::take(&mut self.endnotes);
        self.start_block();
        self.row = 1;
        self.cont = 1;
        let title = self.plain_toks("校勘記");
        let g0 = self.glyphs.len();
        self.item(Item::Main(title));
        for g in &mut self.glyphs[g0..] {
            g.extra = Some("g-td");
        }
        for (k, (anchor, toks)) in notes.into_iter().enumerate() {
            self.used = true;
            self.new_column();
            self.used = true;
            let col = self.col;
            self.afn_box(col, 1.0, 2.0);
            let num = crate::render::numerals::heji(k as u32 + 1);
            self.ann_glyph(col, 0.0, 1.5, 1.0, num, "g-afn");
            self.row = 2;
            self.cont = 2;
            self.pending.clear();
            self.last = None;
            let bare = anchor.is_none();
            if let Some(a) = anchor {
                let t = self.plain_toks(&a);
                self.item(Item::Single(t));
            }
            let (g0, d0) = (self.glyphs.len(), self.decos.len());
            self.item(Item::Double(toks));
            // 逐像素比對：沒有所注之字時，注文比一般夾注高 1px。
            if bare {
                self.remap_y(g0, d0, |_, y| y - PX);
            }
        }
    }

    /// 文字 → 正文字元。
    fn plain_toks(&mut self, text: &str) -> Vec<Tok> {
        let items = self.flatten(&[Inline::Text {
            text: text.to_string(),
            span: Span::default(),
        }]);
        items
            .into_iter()
            .flat_map(|i| match i {
                Item::Main(t) => t,
                _ => Vec::new(),
            })
            .collect()
    }

    /// 不佔格的小字（編號等）。
    fn ann_glyph(
        &mut self,
        col: i32,
        x: f32,
        y: f32,
        size: f32,
        text: String,
        class: &'static str,
    ) {
        self.glyphs.push(Glyph {
            col,
            x,
            y,
            size,
            stretch: 1.0,
            text,
            compose: false,
            rod: false,
            color: None,
            inverse: false,
            hidden: false,
            faded: false,
            encl: false,
            role: Role::Ann,
            level: 0,
            paint: None,
            extra: Some(class),
        });
    }

    /// 校勘記編號的六角框（原版 pCmtAfN，與六角墨圍同畫法）：寬 62px，高一格。
    fn afn_box(&mut self, col: i32, y0: f32, y1: f32) {
        let (x0, x1) = (-31.0 / 30.0, 31.0 / 30.0);
        let rect = |x0: f32, x1: f32, y0: f32, y1: f32| Figure::Rect {
            x0,
            y0,
            x1,
            y1,
            rx: 0.0,
            ry: 0.0,
        };
        let d = 0.2;
        let w = (x1 - x0) * 0.65;
        self.shape(col, rect(-w / 2.0, w / 2.0, y0, y0 + PX), "f-afn");
        self.shape(col, rect(-w / 2.0, w / 2.0, y1 - PX, y1), "f-afn");
        let k = std::f32::consts::SQRT_2;
        for (cx, cy, sx, sy) in [
            (x1, y0, -1.0, 1.0),
            (x1, y1, -1.0, -1.0),
            (x0, y1, 1.0, -1.0),
            (x0, y0, 1.0, 1.0),
        ] {
            let (a, b) = (d * k, (d + PX) * k);
            self.shape(
                col,
                Figure::Polygon(vec![
                    (cx + sx * a, cy),
                    (cx + sx * b, cy),
                    (cx, cy + sy * b),
                    (cx, cy + sy * a),
                ]),
                "f-afn crisp",
            );
        }
    }

    /// 字上的小標，照原版：
    /// - 韻標（ryme）：與字同位同大，畫在字下面，按韻部取色；
    /// - 平仄（rightRuby::after）：半字大，右移 0.75、與字心同高，標點色；
    /// - 樂譜旁字（ruby.mscScr）：0.8 字大，排在右邊另加的一行裏，與字心同高；兩字並排（先右後左）。
    fn annotate(&mut self, a: &Ann, s: &Slot) {
        let push = |e: &mut Self, text: String, x: f32, y: f32, size: f32, class, paint| {
            e.glyphs.push(Glyph {
                col: s.col,
                x,
                y,
                size,
                stretch: 1.0,
                text,
                compose: false,
                rod: false,
                color: None,
                inverse: false,
                hidden: false,
                faded: false,
                encl: false,
                role: Role::Ann,
                level: 0,
                paint,
                extra: Some(class),
            });
        };
        let z = s.size;
        match a {
            Ann::Rhyme { mark, group } => {
                let color = match group {
                    Some(g) => RHYME_COLORS[usize::from(*g) % 10],
                    None => RHYME_COLORS[2],
                };
                push(self, mark.to_string(), s.x, s.y, z, "g-ryme", Some(color));
            }
            Ann::Ruby(text) => {
                // 逐像素比對：再上 1px。
                let y = s.y - 1.0 / 30.0;
                push(
                    self,
                    text.clone(),
                    s.x + 0.75 * z,
                    y,
                    0.5 * z,
                    "g-rruby",
                    None,
                );
            }
            // 原版的左旁字（ruby-position: under）在竪排裏同樣排在右邊一行。
            Ann::Pitch { text, .. } => {
                let cx = s.x + self.o.column_width;
                let chars: Vec<char> = text.chars().collect();
                let n = chars.len() as f32;
                for (i, c) in chars.iter().enumerate() {
                    // 先右後左：第 i 字的中心。
                    let x = cx + 0.8 * z * ((n - 1.0) / 2.0 - i as f32);
                    push(self, c.to_string(), x, s.y, 0.8 * z, "g-pitch", None);
                }
            }
        }
    }

    /// 樂譜旁字所在的行另佔旁邊一行：右旁字的行，正文挪到左邊一行，旁字畫在原位；
    /// 其後各行依次左移。
    fn widen_pitch(&mut self, from: usize, deco_from: usize) {
        use std::collections::BTreeMap;
        // 每行需要在前（右）還是後（左）加一行。
        let mut need: BTreeMap<i32, (bool, bool)> = BTreeMap::new();
        for g in &self.glyphs[from..] {
            if g.extra == Some("g-pitch") {
                let e = need.entry(g.col).or_default();
                if g.x > 0.0 {
                    e.0 = true;
                } else {
                    e.1 = true;
                }
            }
        }
        if need.is_empty() {
            return;
        }
        // 原第 c 行的新位置：之前（含本行右邊）加了幾行。
        let shift = |c: i32| -> i32 {
            need.iter()
                .map(|(&k, &(r, l))| (k < c) as i32 * (r as i32 + l as i32) + (k == c && r) as i32)
                .sum()
        };
        let total: i32 = need.values().map(|&(r, l)| r as i32 + l as i32).sum();
        for g in &mut self.glyphs[from..] {
            g.col += shift(g.col);
        }
        for d in &mut self.decos[deco_from..] {
            let c = d.col();
            d.set_col(c + shift(c));
        }
        self.col += total;
    }

    /// 原版的 flex 單兒、目次：各條順着排，每條至少佔 `frac` 行高（行高含行尾 0.2 字），
    /// 放不下就換行。起點可以落在半格上。
    fn flex(&mut self, entries: Vec<Vec<Item>>, frac: f32) {
        self.start_block();
        let h = self.cells() as f32 + 0.2;
        let mut pos = 0.0f32;
        let mut first = true;
        for items in entries {
            let size = (measure(&items) as f32).max(frac * h);
            if !first && pos + size > h + 1e-3 {
                self.used = true;
                self.new_column();
                pos = 0.0;
            }
            first = false;
            let (g0, d0) = (self.glyphs.len(), self.decos.len());
            self.row = pos.floor() as i32;
            self.cont = 0;
            self.pending.clear();
            self.last = None;
            for it in items {
                self.item(it);
            }
            let dy = pos.fract();
            if dy > 0.0 {
                self.remap_y(g0, d0, |_, y| y + dy);
            }
            self.used = true;
            pos += size;
        }
    }

    /// 把 `glyphs[g0..]`、`decos[d0..]` 的縱坐標按 `f(行, y)` 換掉。
    fn remap_y(&mut self, g0: usize, d0: usize, f: impl Fn(i32, f32) -> f32) {
        for g in &mut self.glyphs[g0..] {
            g.y = f(g.col, g.y);
        }
        for d in &mut self.decos[d0..] {
            match d {
                Deco::Mark { col, y, .. }
                | Deco::Tone { col, y, .. }
                | Deco::Ring { col, y, .. } => *y = f(*col, *y),
                Deco::Line { col, y0, y1, .. }
                | Deco::Wave { col, y0, y1, .. }
                | Deco::Box { col, y0, y1, .. }
                | Deco::Gap { col, y0, y1, .. }
                | Deco::Image { col, y0, y1, .. } => {
                    *y0 = f(*col, *y0);
                    *y1 = f(*col, *y1);
                }
                Deco::Shape { col, figure, .. } => {
                    let c = *col;
                    figure.map_y(|y| f(c, y));
                }
            }
        }
    }

    /// 琵琶譜：按格排好後，把每格拉成譜字的字身高（34px）；只有單側行的格 33px。
    fn stretch_rows(&mut self, from: usize, deco_from: usize) {
        use std::collections::BTreeMap;
        let main = PIPA_PX as f32 / 30.0;
        let line = PIPA_LINE_PX / 30.0;
        // 每行：各格是否只有單側行。
        let mut kinds: BTreeMap<i32, Vec<(i32, bool)>> = BTreeMap::new();
        for g in &self.glyphs[from..] {
            let r = g.y.floor() as i32;
            let v = kinds.entry(g.col).or_default();
            let is_line = g.role == Role::Line;
            match v.iter_mut().find(|(row, _)| *row == r) {
                Some(e) => e.1 &= is_line,
                None => v.push((r, is_line)),
            }
        }
        let map = |col: i32, y: f32| -> f32 {
            let Some(v) = kinds.get(&col) else { return y };
            let r = y.floor() as i32;
            let adv = |k: i32| {
                if v.iter().any(|&(row, l)| row == k && l) {
                    line
                } else {
                    main
                }
            };
            let start: f32 = (0..r).map(adv).sum();
            start + (y - r as f32) * adv(r)
        };
        self.remap_y(from, deco_from, map);
    }

    /// 段落字色：照原版（背景漸變裁成字形），從段首那一行起逐行輪換顏色；標點不變色。
    fn paint(&mut self, from: usize, names: &[String]) {
        let colors: Vec<&'static str> = names
            .iter()
            .filter_map(|n| crate::syntax::palette::lookup(n))
            .collect();
        let Some(first) = self.glyphs[from..].iter().map(|g| g.col).min() else {
            return;
        };
        if colors.is_empty() {
            return;
        }
        for g in &mut self.glyphs[from..] {
            if !matches!(g.role, Role::Punct | Role::Ann) && g.color.is_none() {
                g.paint = Some(colors[(g.col - first) as usize % colors.len()]);
            }
        }
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
                        rod: false,
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
            for (r, mut items) in segments {
                // 原版靠底排（text-align: right）：行末的全角空格挂在行外，不佔格。
                if fmt.bottom.is_some() {
                    while let Some(Item::Main(t)) = items.last_mut() {
                        while t.last().is_some_and(|t| t.kind == TokKind::Space) {
                            t.pop();
                        }
                        if t.is_empty() {
                            items.pop();
                        } else {
                            break;
                        }
                    }
                }
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
                if first < 0 {
                    self.headup(-first);
                }
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
                            x: MAIN_X,
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
                        x: MAIN_X,
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
                        x: MAIN_X,
                        y: self.row as f32 + 0.5,
                        size: SINGLE_SIZE,
                        stretch: 1.0,
                        half_w: SINGLE_SIZE / 2.0,
                        ext: 1.0,
                        role: Role::NoteSingle,
                    };
                    self.place(u, slot, true);
                    self.row += 1;
                }
            }
            Item::Half { toks, left, line } => {
                let g = group(toks);
                self.lead(g.lead);
                for u in g.units {
                    self.ensure_room();
                    let slot = Slot {
                        col: self.col,
                        x: if left { NOTE_L } else { NOTE_R },
                        y: self.row as f32 + 0.5 + NOTE_DROP,
                        size: NOTE_SIZE,
                        stretch: if line { 1.0 } else { NOTE_STRETCH },
                        half_w: NOTE_SIZE / 2.0,
                        ext: 1.0,
                        role: if line { Role::Line } else { Role::Note },
                    };
                    self.place(u, slot, true);
                    self.row += 1;
                }
                self.main_anchor_after_note();
                self.pending.extend(g.tail);
            }
            Item::Pair { right, left } => {
                // 原版 twoLCmt：與側行小注同樣的小字，右段排右半、左段排左半，並起於同一格。
                let start = self.row;
                let mut end = start;
                for (toks, left) in [(right, false), (left, true)] {
                    let g = group(toks);
                    self.lead(g.lead);
                    self.row = start;
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
                    end = end.max(self.row);
                    self.pending.extend(g.tail);
                }
                self.row = end;
                self.main_anchor_after_note();
            }
            Item::Figure { images } => self.float_figure(&images),
            Item::Glyph(c) => self.glyph_image(&c),
            Item::Endnote {
                anchor,
                toks,
                marker,
            } => {
                self.endnotes.push((anchor, toks));
                if marker {
                    self.endnote_marker(self.endnotes.len());
                }
            }
            Item::Span { toks, prev, n, at } => self.span(toks, prev, i32::from(n), at),
            Item::Double(toks) => self.double(toks),
            Item::Side {
                toks,
                left,
                stagger,
            } => self.side(toks, left, stagger),
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
    /// 照原版：夾注之後的正文標點按正文排，附在夾注最後一格（正文字盒的位置）。
    fn main_anchor_after_note(&mut self) {
        if self.row > 0 {
            self.last = Some(Anchor {
                col: self.col,
                x: MAIN_X,
                y: (self.row - 1) as f32 + 0.5,
                size: 1.0,
                posts: 0,
            });
        }
    }

    /// 兼行：照原版，字照常佔本行的格，只是整體向後（前）一行挪半行寬乘 `n`；
    /// 鄰行不讓格，界行也不斷開。
    fn span(&mut self, toks: Vec<Tok>, prev: bool, n: i32, at: Span) {
        let g = group(toks);
        self.lead(g.lead);
        let shift = n as f32 * self.o.column_width / 2.0;
        let x = MAIN_X + if prev { shift } else { -shift };
        let target = if prev { self.col - n } else { self.col + n };
        let half = |c: i32| c.div_euclid(self.o.columns);
        if self.o.format == Format::Leaves && half(target) != half(self.col) {
            self.diags.push(Diagnostic::warning(
                at,
                "兼行跨過了半葉的邊界（版心或版框），請換一行再寫",
            ));
        }
        for u in g.units {
            self.ensure_room();
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
        self.main_anchor_after_note();
        self.pending.extend(g.tail);
    }

    /// 句段旁注：沿行的右侧（或左侧）排小字，不佔正文字格。
    fn side(&mut self, toks: Vec<Tok>, left: bool, stagger: bool) {
        // 錯開（rubyGap）：字距一字，每字佔一格；整段上提半個旁注字。
        let step = if stagger { 2.0 * SIDE_STEP } else { SIDE_STEP };
        let lift = if stagger { SIDE_SIZE / 2.0 } else { 0.0 };
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
        let x = if left { -SIDE_X } else { SIDE_X };
        // 原版旁注底下有紙色底（寬一個旁注字），蓋住界行、版框；每行一段。
        let mut run = (col, y);
        let paper = |e: &mut Self, (c, y0): (i32, f32), y1: f32| {
            if y1 > y0 {
                e.shape(
                    c,
                    Figure::Rect {
                        x0: x - SIDE_SIZE / 2.0,
                        y0,
                        x1: x + SIDE_SIZE / 2.0,
                        y1,
                        rx: 0.0,
                        ry: 0.0,
                    },
                    "paper",
                );
            }
        };
        for u in g.units {
            if y + step > cells + 0.01 {
                paper(self, run, y);
                col += 1;
                y = self.cont as f32;
                run = (col, y);
            }
            let slot = Slot {
                col,
                x,
                y: y + SIDE_STEP / 2.0 - lift,
                size: SIDE_SIZE,
                stretch: 1.0,
                half_w: SIDE_SIZE / 2.0,
                ext: SIDE_STEP,
                role: Role::Side,
            };
            self.place(u, slot, false);
            y += step;
        }
        paper(self, run, y);
        self.side_end[slot_i] = (col, y);
    }

    /// 眉批：排在天頭，从当前行上方开始向左排小字。
    fn head(&mut self, toks: Vec<Tok>) {
        let g = group(toks);
        let cw = self.o.column_width;
        // 天頭從紙邊到版框外綫（字頂上 0.5）：字頂上方 head + 0.3。
        let size = self.o.head_size;
        let cap = (((self.o.head - 0.2) / size).floor() as usize).max(1);
        let step = size * HEAD_LEADING / cw;
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
                y: -self.o.head - 0.3 + (k as f32 + 0.5) * size,
                size,
                stretch: 1.0,
                half_w: size / 2.0,
                ext: size,
                role: Role::Head,
            };
            self.place(unit, slot, false);
        }
        self.head_next = u + step * 1.5;
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
                        x: MAIN_X,
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
        let double = pre.len() == 2 && pre[0].text == "「" && pre[1].text == "『" && s.size >= 0.9;
        for (i, p) in pre.iter().enumerate() {
            // 前引號：照原版，在下一字的位置上移 0.65（夾注裏上移 0.8 個夾注字）。
            // 「『 連用時原版另有樣式（hgbd2Qt1）：「 在右上 (+0.1, -1.35)，『 在 (0, -1.25)。
            let (dx, dy) = match (double, i) {
                (true, 0) => (0.1, -1.35),
                (true, _) => (0.0, -1.25),
                _ if s.size < 0.9 => (0.0, -0.64),
                _ => (0.0, -0.65),
            };
            self.punct(p, s.col, s.x + dx, s.y + dy, s.size);
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
        let c = t.text.chars().next().unwrap_or(' ');
        let (dx, dy) = if (a.size - SINGLE_SIZE).abs() < 1e-3 {
            // 單行小注（28px）：照原版實測。
            (0.35 * a.size, 0.66)
        } else {
            punct_offset(c, a.size < 0.9)
        };
        let (x, y) = (a.x + dx, a.y + dy);
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
            rod: false,
            color: Some(t.style.color.unwrap_or(self.o.punct_color)),
            inverse: false,
            hidden: t.hidden,
            faded: false,
            encl: false,
            role: Role::Punct,
            level: 0,
            paint: None,
            extra: None,
        });
        self.max_col = self.max_col.max(col);
    }

    /// 放一个字。返回是否真的画了字（空格不画）。
    fn put(&mut self, t: &Tok, s: Slot) -> bool {
        if let Some(a) = &t.style.ann {
            self.annotate(a, &s);
        }
        let mut inverse = false;
        let mut faded = false;
        let in_note = matches!(s.role, Role::Note | Role::NoteSingle);
        let z = s.size;
        for m in &t.style.marks {
            match m.kind {
                // 圈點：照原版 strROO / strROF，在字右 0.604 處，半徑 0.096（空心圈內半徑 0.0384）。
                MarkKind::Emphasis(EmphMark::Circle) | MarkKind::Collate(EmphMark::Circle) => self
                    .shape(
                        s.col,
                        Figure::Ring {
                            x: s.x + 0.604 * z,
                            y: s.y,
                            r0: 0.0384 * z,
                            r1: 0.096 * z,
                        },
                        emph_class(m.kind),
                    ),
                MarkKind::Emphasis(EmphMark::Dot) | MarkKind::Collate(_) => self.shape(
                    s.col,
                    Figure::Circle {
                        x: s.x + 0.604 * z,
                        y: s.y,
                        r: 0.096 * z,
                    },
                    emph_class(m.kind),
                ),
                // 旁點（﹅﹆）：原版是字，排在字後、右 0.72、上提 0.84（18px 字）。
                MarkKind::Emphasis(mark) => {
                    // 字盒落點相同，兩個字形在字體裏的高低不同：逐像素比對，﹅ 再上 3px、﹆ 上 2px。
                    let (c, dy) = match mark {
                        EmphMark::Sesame => ("﹅", 0.16 - 3.0 / 30.0),
                        // 方框（琵琶譜）由譜字字體出，字盒高 21px（﹅ 18px），中心低 1.5px。
                        EmphMark::Box => ("ロ", 0.16 - 3.0 / 30.0 + 1.5 / 30.0),
                        _ => ("﹆", 0.16 - 2.0 / 30.0),
                    };
                    self.glyphs.push(Glyph {
                        col: s.col,
                        x: s.x + 0.72 * z,
                        y: s.y + dy * z,
                        size: 0.6 * z,
                        stretch: 1.0,
                        text: c.into(),
                        compose: false,
                        rod: false,
                        color: None,
                        inverse: false,
                        hidden: false,
                        faded: false,
                        encl: false,
                        role: Role::Punct,
                        level: 0,
                        paint: None,
                        extra: None,
                    });
                }
                MarkKind::Inverse(e) | MarkKind::Frame(e) if e.shape.is_each() => {
                    let filled = matches!(m.kind, MarkKind::Inverse(_));
                    inverse |= filled;
                    // 正文裏圓心照原版在內容區中間（比字心偏左）；夾注照舊。
                    let dx = if z < 0.9 { 0.0 } else { ENCL_DX * z };
                    let (x, y) = (s.x + dx, s.y);
                    let fig = match (filled, e.shape == Shape::Lantern) {
                        // 圓形陰文：每字一個實心圓，直徑一字。
                        (true, false) => Figure::Circle { x, y, r: 0.5 * z },
                        // 疊燈籠陰文：半徑 0.6 的圓，裁在本字一格內，上下相接。
                        (true, true) => Figure::Clip(
                            Box::new(Figure::Circle { x, y, r: 0.6 * z }),
                            [x - 0.6 * z, y - 0.5, x + 0.6 * z, y + 0.5],
                        ),
                        // 帶圈字：半徑 0.45–0.5 的圓環。
                        (false, false) => Figure::Ring {
                            x,
                            y,
                            r0: 0.45 * z,
                            r1: 0.5 * z,
                        },
                        // 疊燈籠墨圍：半徑 0.6 加 1px 的圓環，裁在本字一格內。
                        (false, true) => Figure::Clip(
                            Box::new(Figure::Ring {
                                x,
                                y,
                                r0: 0.6 * z,
                                r1: 0.6 * z + PX,
                            }),
                            [x - 0.7 * z, y - 0.5, x + 0.7 * z, y + 0.5],
                        ),
                    };
                    // 原版用 radial-gradient 畫，邊不抗鋸齒。
                    self.shape(
                        s.col,
                        fig,
                        if filled {
                            "f-red crisp"
                        } else {
                            "f-frame crisp"
                        },
                    );
                }
                // 夾注裏的篇名：原版改字色為紫（`<span style="color:purple">`），不畫波綫。
                MarkKind::TitleRight if in_note => {}
                kind => {
                    inverse |= matches!(kind, MarkKind::Inverse(_));
                    faded |= kind == MarkKind::Delete;
                    self.extend_seg(m.id, kind, s, &t.text);
                }
            }
        }
        // 眉批在天頭，不佔行（原版的行數只算正文）。
        if s.role == Role::Head {
            self.head_col = self.head_col.max(s.col);
        } else {
            self.max_col = self.max_col.max(s.col);
        }
        if t.kind == TokKind::Space {
            return false;
        }
        let tone = t.tone;
        // 夾注裏的小注：字號為夾注的 0.625（15px）。
        let size = if t.style.num == Some("g-note-in") {
            s.size * 0.625
        } else {
            s.size
        };
        self.glyphs.push(Glyph {
            col: s.col,
            x: s.x,
            y: s.y,
            size,
            stretch: s.stretch,
            text: t.text.clone(),
            compose: t.compose,
            rod: t.rod,
            color: if in_note && t.style.marks.iter().any(|m| m.kind == MarkKind::TitleRight) {
                Some(Color::Purple)
            } else {
                t.style.color
            },
            inverse,
            hidden: false,
            faded,
            encl: t
                .style
                .marks
                .iter()
                .any(|m| matches!(m.kind, MarkKind::Inverse(_) | MarkKind::Frame(_))),
            role: s.role,
            level: 0,
            paint: None,
            extra: t.style.num,
        });
        // 圈發：原版是「標點符號」字體裏的圈，排在字後，照原版實測的盒位。
        if let Some(tone) = tone {
            let (c, dx, dy) = match tone {
                Tone::Level => ("〪", -0.6, 0.1),
                Tone::Rising => ("〫", -0.6, -0.1),
                Tone::Departing => ("〬", 0.1, -0.6),
                Tone::Entering => ("〭", -0.4, 0.1),
            };
            self.glyphs.push(Glyph {
                col: s.col,
                x: s.x + dx * z,
                y: s.y + dy * z,
                size: if c == "〬" { z } else { 0.8 * z },
                stretch: 1.0,
                text: c.into(),
                compose: false,
                rod: false,
                color: None,
                inverse: false,
                hidden: false,
                faded: false,
                encl: false,
                role: Role::Tone,
                level: 0,
                paint: None,
                extra: None,
            });
        }
        true
    }

    /// 擡頭伸出版框的一截，照原版 .headupN：紙色底蓋住版框上沿，
    /// 頂上 2px 綫、其下 3px 處 1px 綫，左右各 1px 界行，一直到字頂。
    fn headup(&mut self, n: i32) {
        let cw = self.o.column_width;
        let l = -cw / 2.0 + PX;
        let top = -(n as f32) - 15.0 * PX;
        let rect = |x0: f32, x1: f32, y0: f32, y1: f32| Figure::Rect {
            x0,
            y0,
            x1,
            y1,
            rx: 0.0,
            ry: 0.0,
        };
        let col = self.col;
        self.shape(col, rect(l, l + cw, top, 0.0), "paper");
        self.shape(col, rect(l, l + cw, top, top + 2.0 * PX), "f-frame");
        self.shape(
            col,
            rect(l, l + cw, top + 5.0 * PX, top + 6.0 * PX),
            "f-frame",
        );
        self.shape(
            col,
            rect(l + cw - PX, l + cw, top + 2.0 * PX, 0.0),
            "f-frame",
        );
        self.shape(col, rect(l - PX, l, top, 0.0), "f-frame");
    }

    fn shape(&mut self, col: i32, figure: Figure, class: &'static str) {
        self.decos.push(Deco::Shape {
            col,
            figure,
            class,
            over: false,
        });
    }

    fn extend_seg(&mut self, id: u32, kind: MarkKind, s: Slot, text: &str) {
        let key = (id, s.col, (s.x * 1000.0).round() as i32);
        let (y0, y1) = (s.y - s.ext / 2.0, s.y + s.ext / 2.0);
        match self.seg_at.get(&key) {
            Some(&i) => {
                let seg = &mut self.segs[i];
                seg.y0 = seg.y0.min(y0);
                seg.y1 = seg.y1.max(y1);
                seg.text.push_str(text);
            }
            None => {
                self.seg_at.insert(key, self.segs.len());
                self.segs.push(Seg {
                    id,
                    kind,
                    text: text.to_string(),
                    col: s.col,
                    x: s.x,
                    half_w: s.half_w,
                    y0,
                    y1,
                    role: s.role,
                });
            }
        }
    }

    fn finish(mut self) -> Layout {
        // 同一個標記的各段（跨行時每行一段）：專名號只在頭尾兩段縮進。
        let segs = std::mem::take(&mut self.segs);
        let mut first_last: HashMap<u32, (usize, usize)> = HashMap::new();
        for (i, sg) in segs.iter().enumerate() {
            let e = first_last.entry(sg.id).or_insert((i, i));
            if (sg.col, sg.y0) < (segs[e.0].col, segs[e.0].y0) {
                e.0 = i;
            }
            if (sg.col, sg.y1) > (segs[e.1].col, segs[e.1].y1) {
                e.1 = i;
            }
        }
        for (i, sg) in segs.iter().enumerate() {
            let (fi, li) = first_last[&sg.id];
            self.seg_figures(sg, i == fi, i == li);
        }
        let last_col = if self.used { self.col } else { self.col - 1 };
        Layout {
            columns: self.max_col.max(last_col) + 1,
            extent: self.max_col.max(last_col).max(self.head_col) + 1,
            glyphs: self.glyphs,
            decos: self.decos,
            options: self.o,
        }
    }
}

/// 一個 CSS 像素（正文 30px），單位：字。
const PX: f32 = 1.0 / 30.0;

impl Engine<'_> {
    /// 一段標記（同一行裏連續的字）照原版 CSS 畫成的圖形。`first`、`last`：是否整個標記的頭、尾一段。
    fn seg_figures(&mut self, sg: &Seg, first: bool, last: bool) {
        let z = sg.half_w * 2.0;
        let (x, y0, y1) = (sg.x, sg.y0, sg.y1);
        let note = z < 0.9;
        let col = sg.col;
        let rect = |x0: f32, x1: f32, y0: f32, y1: f32| Figure::Rect {
            x0,
            y0,
            x1,
            y1,
            rx: 0.0,
            ry: 0.0,
        };
        match sg.kind {
            // 專名號：原版用 box-shadow 在字盒左沿畫 0.1 寬的朱綫，頭尾各縮 0.15。
            MarkKind::Name if !note => {
                let a = if first { 0.15 } else { 0.0 };
                let b = if last { 0.15 } else { 0.0 };
                self.shape(
                    col,
                    rect(x - 0.55 * z, x - 0.45 * z, y0 + a, y1 - b),
                    "f-red",
                );
            }
            // 夾注裏的專名號：字盒左沿內 0.2 字寬的半透明朱條，上縮 0.15、下縮 0.35（夾注字）。
            MarkKind::Name => {
                let a = if first { 0.15 * z } else { 0.0 };
                let b = if last { 0.35 * z } else { 0.0 };
                self.shape(
                    col,
                    rect(x - 0.5 * z, x - 0.3 * z, y0 + a, y1 - b),
                    "f-red-t",
                );
            }
            MarkKind::Title | MarkKind::TitleRight => self.decos.push(Deco::Wave {
                col,
                x,
                y0,
                y1,
                text: sg.text.clone(),
                right: sg.kind == MarkKind::TitleRight,
                first,
                last,
                size: z,
                role: sg.role,
            }),
            // 抹（原版 colateMS）：字盒中間 60% 寬的橙色底。
            MarkKind::Stroke => self.shape(col, rect(x - 0.3 * z, x + 0.3 * z, y0, y1), "f-strike"),
            // 刪（原版 del）：淺灰底、正赤刪除綫，整體 60% 不透明。
            MarkKind::Delete => {
                self.shape(col, rect(x - 0.5 * z, x + 0.5 * z, y0, y1), "f-del");
                self.decos.push(Deco::Shape {
                    col,
                    // 刪除綫在字形中綫左 1–4px，寬 3px（原版 line-through 照字體的位置、粗細）。
                    figure: rect(x - 4.0 * PX, x - PX, y0, y1),
                    class: "f-delline crisp",
                    over: true,
                });
            }
            // 增（原版 ins）：半透明淺灰底。
            MarkKind::Insert => self.shape(col, rect(x - 0.5 * z, x + 0.5 * z, y0, y1), "f-ins"),
            MarkKind::Inverse(e) => {
                // 陰文：字盒左右各加 (行寬 - 1)/2，正好一行寬（夾注同理用夾注行寬）。
                // 正文：內容區左右各加 (行寬 - 1)/2（原版 padding）；夾注照舊取夾注行寬。
                let half = if note { 0.5 } else { self.o.column_width / 2.0 };
                let (y0, y1) = if note { (y0 - 0.1, y1 - 0.1) } else { (y0, y1) };
                let pad = (self.o.column_width - z) / 2.0;
                let (x0, x1) = if note {
                    (x - half, x + half)
                } else {
                    (x - ENCL_L * z - pad, x + ENCL_R * z + pad)
                };
                let fig = match e.shape {
                    Shape::Round => Figure::Rect {
                        x0,
                        y0,
                        x1,
                        y1,
                        rx: half,
                        ry: half - 0.5 * z,
                    },
                    Shape::Hex => {
                        let c = (if note { 0.14 * z } else { 0.2 }) * std::f32::consts::SQRT_2;
                        cut_corners(x0, y0, x1, y1, c)
                    }
                    _ => rect(x0, x1, y0, y1),
                };
                self.shape(col, fig, "f-red");
            }
            MarkKind::Frame(e) => {
                let half = if e.narrow {
                    0.5 * z + 0.1
                } else {
                    self.o.column_width / 2.0
                };
                // 正文照原版：內容區加 padding（窄式 0.1，其餘 (行寬 - 1)/2）。
                let pad = if e.narrow {
                    0.1
                } else {
                    (self.o.column_width - z) / 2.0
                };
                let (x0, x1) = if note {
                    (x - half, x + half)
                } else {
                    (x - ENCL_L * z - pad, x + ENCL_R * z + pad)
                };
                match (e.shape, e.narrow) {
                    // 窄圓頭（mhHiuanNR）：圓角 0.6/0.3 的框，四邊 1px。
                    (Shape::Round, true) => {
                        let outer = Figure::Rect {
                            x0,
                            y0,
                            x1,
                            y1,
                            rx: 0.6,
                            ry: 0.3,
                        };
                        let inner = Figure::Rect {
                            x0: x0 + PX,
                            y0: y0 + PX,
                            x1: x1 - PX,
                            y1: y1 - PX,
                            rx: 0.6 - PX,
                            ry: 0.3 - PX,
                        };
                        self.shape(
                            col,
                            Figure::Minus(Box::new(outer), Box::new(inner)),
                            "f-frame",
                        );
                    }
                    // 圓頭（mhHiuan）：圓角框的上下沿各 1px（內陰影）。
                    (Shape::Round, false) => {
                        let r = |dy: f32| Figure::Rect {
                            x0,
                            y0: y0 + dy,
                            x1,
                            y1: y1 + dy,
                            rx: half,
                            ry: half - 0.5,
                        };
                        self.shape(
                            col,
                            Figure::Minus(Box::new(r(0.0)), Box::new(r(PX))),
                            "f-frame",
                        );
                        self.shape(
                            col,
                            Figure::Minus(Box::new(r(0.0)), Box::new(r(-PX))),
                            "f-frame",
                        );
                    }
                    // 六角（mh6grug、窄式 mh8grugNR）：上下沿與四角斜邊連成一圈 1px 的綫。
                    // 原版是六塊背景各畫各的，斜邊與上下沿接不上，還伸出界行；這裏畫成一筆。
                    (Shape::Hex, narrow) => {
                        let d = if narrow { 0.14 } else { 0.2 } * z;
                        // 正文的框正好一行寬：斜邊的端點落在左右界行上（界行在行左、右沿往右 1px）。
                        let cw = self.o.column_width;
                        let (xl, xr) = if note || narrow {
                            (x0, x1)
                        } else {
                            (-cw / 2.0, cw / 2.0 + PX)
                        };
                        let c = d * std::f32::consts::SQRT_2;
                        // 內圈：斜邊往裏平移 1px，上下沿各收 1px。
                        let ci = c + PX * (std::f32::consts::SQRT_2 - 1.0);
                        let outer = cut_corners(xl, y0, xr, y1, c);
                        let inner = cut_corners(xl, y0 + PX, xr, y1 - PX, ci);
                        // 去掉左右兩條（界行或窄式的豎綫已有）。
                        self.shape(
                            col,
                            Figure::Clip(
                                Box::new(Figure::Minus(Box::new(outer), Box::new(inner))),
                                [xl, y0, xr, y1],
                            ),
                            "f-frame",
                        );
                        if narrow {
                            // 窄六角（原版 mh8grugNR）：左右豎綫只到斜邊端點，上下各收 0.2；
                            // 六角（內層 span 外擴 0.2）佔滿整段。
                            self.shape(col, rect(x0, x0 + PX, y0 + 0.2, y1 - 0.2), "f-frame");
                            self.shape(col, rect(x1 - PX, x1, y0 + 0.2, y1 - 0.2), "f-frame");
                        }
                    }
                    // 平頭（mhVang）：上下沿各 1px，左右靠界行。
                    _ => {
                        self.shape(col, rect(x0, x1, y0, y0 + PX), "f-frame");
                        self.shape(col, rect(x0, x1, y1 - PX, y1), "f-frame");
                    }
                }
            }
            MarkKind::Emphasis(_) | MarkKind::Collate(_) => {}
        }
    }
}

fn emph_class(k: MarkKind) -> &'static str {
    if matches!(k, MarkKind::Collate(_)) {
        "f-collate crisp"
    } else {
        "f-emph crisp"
    }
}

/// 四角各切去一個等腰直角三角形（直角邊長 `c`）的矩形。
fn cut_corners(x0: f32, y0: f32, x1: f32, y1: f32, c: f32) -> Figure {
    Figure::Polygon(vec![
        (x0 + c, y0),
        (x1 - c, y0),
        (x1, y0 + c),
        (x1, y1 - c),
        (x1 - c, y1),
        (x0 + c, y1),
        (x0, y1 - c),
        (x0, y0 + c),
    ])
}

/// 句讀相對所附字的盒心偏移（字），照原版 CSS：
/// 標點不佔格（margin-bottom: -1em），原位在下一格，再按各類的 left、bottom 挪動。
/// - 正文：，、。．左移 0.35、上提 0.4；：；左移 0.35、上提 0.6；？！左移 0.4、上提 1；後引號上提 0.35。
/// - 夾注（0.8 字號）：，、。．：；上提 0.4 個夾注字；？！左移 0.2、上提 1；後引號上提 0.5。
fn punct_offset(c: char, note: bool) -> (f32, f32) {
    match (note, c) {
        (false, '，' | '、' | '。' | '．') => (0.35, 0.6),
        (false, '：' | '；') => (0.35, 0.4),
        (false, '？' | '！') => (0.4, 0.0),
        (false, '」' | '』' | '”' | '’') => (0.0, 0.65),
        (true, '，' | '、' | '。' | '．' | '：' | '；') => (0.0, 1.0 - 0.4 * 0.8),
        (true, '？' | '！') => (0.2 * 0.8, 1.0 - 0.8),
        (true, '」' | '』' | '”' | '’') => (0.0, 1.0 - 0.5 * 0.8),
        (false, _) => (0.35, 0.6),
        (true, _) => (0.0, 1.0 - 0.4 * 0.8),
    }
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
fn rect_fig(x0: f32, x1: f32, y0: f32, y1: f32) -> Figure {
    Figure::Rect {
        x0,
        y0,
        x1,
        y1,
        rx: 0.0,
        ry: 0.0,
    }
}

/// 整個裝飾縱向挪 `dy`。
fn shift_deco(d: &mut Deco, dy: f32) {
    match d {
        Deco::Mark { y, .. } | Deco::Tone { y, .. } | Deco::Ring { y, .. } => *y += dy,
        Deco::Line { y0, y1, .. }
        | Deco::Wave { y0, y1, .. }
        | Deco::Box { y0, y1, .. }
        | Deco::Gap { y0, y1, .. }
        | Deco::Image { y0, y1, .. } => {
            *y0 += dy;
            *y1 += dy;
        }
        Deco::Shape { figure, .. } => figure.map_y(|y| y + dy),
    }
}

/// 表格一列：按 `&` 切成各格。
fn split_cells(items: Vec<Item>) -> Vec<Vec<Item>> {
    let mut cells: Vec<Vec<Item>> = vec![Vec::new()];
    for it in items {
        match it {
            Item::Main(toks) => {
                let mut cur = Vec::new();
                for t in toks {
                    if t.text == "&" {
                        if !cur.is_empty() {
                            cells.last_mut().expect("non-empty").push(Item::Main(cur));
                        }
                        cur = Vec::new();
                        cells.push(Vec::new());
                    } else {
                        cur.push(t);
                    }
                }
                if !cur.is_empty() {
                    cells.last_mut().expect("non-empty").push(Item::Main(cur));
                }
            }
            other => cells.last_mut().expect("non-empty").push(other),
        }
    }
    cells
}

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
            Item::Pair { right, left } => cells(right).max(cells(left)),
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
        // 專名號照原版畫成每行一條朱色矩形。
        let l = run(r"\name{甲乙丙}", 2);
        let lines = l
            .decos
            .iter()
            .filter(|d| matches!(d, crate::layout::Deco::Shape { class: "f-red", .. }))
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
        assert_eq!(text, "曰：「天繫，戶計翻。地。」");
    }

    #[test]
    fn head_note_goes_above_frame() {
        let l = run(r"甲\head{眉批文字}乙", 10);
        assert!(at(&l, Role::Head).iter().all(|g| g.2 < 0));
    }

    #[test]
    fn chapter_title_in_note_is_purple() {
        // 原版：夾注裏的〈篇名〉改為紫字，不畫波綫；正文裏照畫。
        let l = run("甲\\note{見〈禮運〉篇}〈月令〉", 10);
        let purple: String = l
            .glyphs
            .iter()
            .filter(|g| g.color == Some(crate::syntax::ast::Color::Purple))
            .map(|g| g.text.as_str())
            .collect();
        assert_eq!(purple, "禮運");
        let waves = l
            .decos
            .iter()
            .filter(|d| matches!(d, crate::layout::Deco::Wave { .. }))
            .count();
        assert_eq!(waves, 1, "只有正文的〈月令〉有波綫");
    }

    #[test]
    fn head_note_size() {
        // 天頭 4 字、眉批半字：每行排 (4 − 0.2) / 0.5 = 7 字，行寬 0.7 字。
        let parsed = parse(r"甲\head{一二三四五六七八九十}乙");
        let mut d = parsed.diagnostics;
        let o = Options {
            head: 4.0,
            head_set: true,
            head_size: 0.5,
            ..Options::default()
        };
        let l = layout(&parsed.document, o, &mut d);
        let head: Vec<_> = l.glyphs.iter().filter(|g| g.role == Role::Head).collect();
        assert!(head.iter().all(|g| g.size == 0.5));
        let ys: Vec<f32> = head.iter().map(|g| g.y).collect();
        assert!(ys[6] > ys[7], "第八字另起一行");
        let dx = (head[7].col as f32 * 1.6 - head[7].x) - (head[0].col as f32 * 1.6 - head[0].x);
        assert!((dx - 0.7).abs() < 1e-3, "行寬 {dx}");
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
    fn span_shifts_without_reserving() {
        // 照原版：「明算」整體挪向下一行半行寬，鄰行不讓格。
        let l = run("甲\\span{明算}乙\n丙丁戊", cells(10));
        let g: Vec<(i32, f32, f32, &str)> = l
            .glyphs
            .iter()
            .map(|g| (g.col, g.x, g.y, g.text.as_str()))
            .collect();
        assert_eq!((g[1].0, g[1].2, g[1].3), (0, 1.5, "明"));
        assert!((g[1].1 - (super::MAIN_X - 0.8)).abs() < 1e-6);
        let next: Vec<f32> = g.iter().filter(|g| g.0 == 1).map(|g| g.2).collect();
        assert_eq!(next, [0.5, 1.5, 2.5]);
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
        assert_eq!(notes, [(-0.35, 1.58), (-0.35, 2.58)]);
        assert_eq!(l.glyphs.last().unwrap().y, 3.5);
    }

    #[test]
    fn rods_fill_one_cell_like_original() {
        // 照原版：整串碼子字排在一格裏（字形由「華數碼」字體畫成算籌）。
        let l = run("\\rod{1971}年", cells(10));
        let g: Vec<(&str, f32)> = l.glyphs.iter().map(|g| (g.text.as_str(), g.y)).collect();
        assert_eq!(g, [("〡〩〧〡", 0.5), ("年", 1.5)]);
        assert!(l.glyphs[0].rod);
        let l = run("\\rod{105}", cells(10));
        assert_eq!(l.glyphs[0].text, "〡〇〥");
    }

    #[test]
    fn frames_touch_rules_unless_narrow() {
        use crate::layout::Figure;
        // 平頭墨圍照原版是上下兩條綫，寬是字的內容區（ENCL_L + ENCL_R）加左右 padding：
        // 正文 (行寬 - 1)/2，窄式 0.1。
        let width = |l: &crate::layout::Layout| match &l.decos[0] {
            Deco::Shape {
                figure: Figure::Rect { x0, x1, .. },
                ..
            } => x1 - x0,
            d => panic!("{d:?}"),
        };
        let wide = run("\\frame{甲}", cells(10));
        let narrow = run("\\frame[narrow]{甲}", cells(10));
        let content = super::ENCL_L + super::ENCL_R;
        assert!((width(&wide) - (content + 0.6)).abs() < 1e-4);
        assert!((width(&narrow) - (content + 0.2)).abs() < 1e-4);
        // 疊燈籠陰文：每字一個裁在本格的圓。
        let lantern = run("\\inv[lantern]{甲乙}", cells(10));
        let clips = lantern
            .decos
            .iter()
            .filter(|d| {
                matches!(
                    d,
                    Deco::Shape {
                        figure: Figure::Clip(..),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(clips, 2);
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
