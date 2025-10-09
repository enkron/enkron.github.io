#![warn(clippy::all, clippy::pedantic)]
use std::fmt::Write;
use std::io::Write as IoWrite;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};

const PAGE_WIDTH: f32 = 595.0;
const PAGE_HEIGHT: f32 = 842.0;
const MARGIN_HORIZONTAL: f32 = 40.0;
const MARGIN_TOP: f32 = 50.0;
const MARGIN_BOTTOM: f32 = 40.0;
const BODY_FONT_SIZE: f32 = 9.0;
const HEADING1_FONT_SIZE: f32 = 16.0;
const HEADING2_FONT_SIZE: f32 = 12.0;
const HEADING3_FONT_SIZE: f32 = 11.0;
const LINE_SPACING_FACTOR: f32 = 1.6;
const BULLET_INDENT_POINTS: f32 = 18.0;

#[derive(Copy, Clone)]
struct HelveticaPair {
    regular: f32,
    bold: f32,
}

const DEFAULT_HELVETICA_WIDTH: HelveticaPair = HelveticaPair {
    regular: 500.0,
    bold: 556.0,
};

const HELVETICA_WIDTHS: &[(char, HelveticaPair)] = &[
    (
        ' ',
        HelveticaPair {
            regular: 278.0,
            bold: 278.0,
        },
    ),
    (
        '!',
        HelveticaPair {
            regular: 278.0,
            bold: 333.0,
        },
    ),
    (
        '"',
        HelveticaPair {
            regular: 355.0,
            bold: 474.0,
        },
    ),
    (
        '#',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '$',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '%',
        HelveticaPair {
            regular: 889.0,
            bold: 889.0,
        },
    ),
    (
        '&',
        HelveticaPair {
            regular: 667.0,
            bold: 722.0,
        },
    ),
    (
        '\'',
        HelveticaPair {
            regular: 191.0,
            bold: 278.0,
        },
    ),
    (
        '(',
        HelveticaPair {
            regular: 333.0,
            bold: 333.0,
        },
    ),
    (
        ')',
        HelveticaPair {
            regular: 333.0,
            bold: 333.0,
        },
    ),
    (
        '*',
        HelveticaPair {
            regular: 389.0,
            bold: 389.0,
        },
    ),
    (
        '+',
        HelveticaPair {
            regular: 584.0,
            bold: 584.0,
        },
    ),
    (
        ',',
        HelveticaPair {
            regular: 278.0,
            bold: 278.0,
        },
    ),
    (
        '-',
        HelveticaPair {
            regular: 333.0,
            bold: 333.0,
        },
    ),
    (
        '.',
        HelveticaPair {
            regular: 278.0,
            bold: 278.0,
        },
    ),
    (
        '/',
        HelveticaPair {
            regular: 278.0,
            bold: 278.0,
        },
    ),
    (
        '0',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '1',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '2',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '3',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '4',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '5',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '6',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '7',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '8',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '9',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        ':',
        HelveticaPair {
            regular: 278.0,
            bold: 333.0,
        },
    ),
    (
        ';',
        HelveticaPair {
            regular: 278.0,
            bold: 333.0,
        },
    ),
    (
        '<',
        HelveticaPair {
            regular: 584.0,
            bold: 584.0,
        },
    ),
    (
        '=',
        HelveticaPair {
            regular: 584.0,
            bold: 584.0,
        },
    ),
    (
        '>',
        HelveticaPair {
            regular: 584.0,
            bold: 584.0,
        },
    ),
    (
        '?',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        '@',
        HelveticaPair {
            regular: 1015.0,
            bold: 975.0,
        },
    ),
    (
        'A',
        HelveticaPair {
            regular: 667.0,
            bold: 722.0,
        },
    ),
    (
        'B',
        HelveticaPair {
            regular: 667.0,
            bold: 722.0,
        },
    ),
    (
        'C',
        HelveticaPair {
            regular: 722.0,
            bold: 722.0,
        },
    ),
    (
        'D',
        HelveticaPair {
            regular: 722.0,
            bold: 722.0,
        },
    ),
    (
        'E',
        HelveticaPair {
            regular: 667.0,
            bold: 667.0,
        },
    ),
    (
        'F',
        HelveticaPair {
            regular: 611.0,
            bold: 611.0,
        },
    ),
    (
        'G',
        HelveticaPair {
            regular: 778.0,
            bold: 778.0,
        },
    ),
    (
        'H',
        HelveticaPair {
            regular: 722.0,
            bold: 722.0,
        },
    ),
    (
        'I',
        HelveticaPair {
            regular: 278.0,
            bold: 278.0,
        },
    ),
    (
        'J',
        HelveticaPair {
            regular: 500.0,
            bold: 556.0,
        },
    ),
    (
        'K',
        HelveticaPair {
            regular: 667.0,
            bold: 722.0,
        },
    ),
    (
        'L',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'M',
        HelveticaPair {
            regular: 833.0,
            bold: 833.0,
        },
    ),
    (
        'N',
        HelveticaPair {
            regular: 722.0,
            bold: 722.0,
        },
    ),
    (
        'O',
        HelveticaPair {
            regular: 778.0,
            bold: 778.0,
        },
    ),
    (
        'P',
        HelveticaPair {
            regular: 667.0,
            bold: 667.0,
        },
    ),
    (
        'Q',
        HelveticaPair {
            regular: 778.0,
            bold: 778.0,
        },
    ),
    (
        'R',
        HelveticaPair {
            regular: 722.0,
            bold: 722.0,
        },
    ),
    (
        'S',
        HelveticaPair {
            regular: 667.0,
            bold: 667.0,
        },
    ),
    (
        'T',
        HelveticaPair {
            regular: 611.0,
            bold: 611.0,
        },
    ),
    (
        'U',
        HelveticaPair {
            regular: 722.0,
            bold: 722.0,
        },
    ),
    (
        'V',
        HelveticaPair {
            regular: 667.0,
            bold: 667.0,
        },
    ),
    (
        'W',
        HelveticaPair {
            regular: 944.0,
            bold: 944.0,
        },
    ),
    (
        'X',
        HelveticaPair {
            regular: 667.0,
            bold: 667.0,
        },
    ),
    (
        'Y',
        HelveticaPair {
            regular: 667.0,
            bold: 667.0,
        },
    ),
    (
        'Z',
        HelveticaPair {
            regular: 611.0,
            bold: 611.0,
        },
    ),
    (
        '[',
        HelveticaPair {
            regular: 278.0,
            bold: 333.0,
        },
    ),
    (
        '\\',
        HelveticaPair {
            regular: 278.0,
            bold: 278.0,
        },
    ),
    (
        ']',
        HelveticaPair {
            regular: 278.0,
            bold: 333.0,
        },
    ),
    (
        '^',
        HelveticaPair {
            regular: 469.0,
            bold: 581.0,
        },
    ),
    (
        '_',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        '`',
        HelveticaPair {
            regular: 222.0,
            bold: 333.0,
        },
    ),
    (
        'a',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        'b',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'c',
        HelveticaPair {
            regular: 500.0,
            bold: 556.0,
        },
    ),
    (
        'd',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'e',
        HelveticaPair {
            regular: 556.0,
            bold: 556.0,
        },
    ),
    (
        'f',
        HelveticaPair {
            regular: 278.0,
            bold: 333.0,
        },
    ),
    (
        'g',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'h',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'i',
        HelveticaPair {
            regular: 222.0,
            bold: 278.0,
        },
    ),
    (
        'j',
        HelveticaPair {
            regular: 222.0,
            bold: 278.0,
        },
    ),
    (
        'k',
        HelveticaPair {
            regular: 500.0,
            bold: 556.0,
        },
    ),
    (
        'l',
        HelveticaPair {
            regular: 222.0,
            bold: 278.0,
        },
    ),
    (
        'm',
        HelveticaPair {
            regular: 833.0,
            bold: 889.0,
        },
    ),
    (
        'n',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'o',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'p',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'q',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'r',
        HelveticaPair {
            regular: 333.0,
            bold: 389.0,
        },
    ),
    (
        's',
        HelveticaPair {
            regular: 500.0,
            bold: 556.0,
        },
    ),
    (
        't',
        HelveticaPair {
            regular: 278.0,
            bold: 333.0,
        },
    ),
    (
        'u',
        HelveticaPair {
            regular: 556.0,
            bold: 611.0,
        },
    ),
    (
        'v',
        HelveticaPair {
            regular: 500.0,
            bold: 556.0,
        },
    ),
    (
        'w',
        HelveticaPair {
            regular: 722.0,
            bold: 778.0,
        },
    ),
    (
        'x',
        HelveticaPair {
            regular: 500.0,
            bold: 556.0,
        },
    ),
    (
        'y',
        HelveticaPair {
            regular: 500.0,
            bold: 556.0,
        },
    ),
    (
        'z',
        HelveticaPair {
            regular: 500.0,
            bold: 500.0,
        },
    ),
    (
        '{',
        HelveticaPair {
            regular: 334.0,
            bold: 389.0,
        },
    ),
    (
        '|',
        HelveticaPair {
            regular: 260.0,
            bold: 280.0,
        },
    ),
    (
        '}',
        HelveticaPair {
            regular: 334.0,
            bold: 389.0,
        },
    ),
    (
        '~',
        HelveticaPair {
            regular: 584.0,
            bold: 584.0,
        },
    ),
    (
        '•',
        HelveticaPair {
            regular: 350.0,
            bold: 350.0,
        },
    ),
];
/// Get character width for Helvetica font (in 1000 units, scale by `font_size`/1000)
fn helvetica_char_width(c: char, bold: bool) -> f32 {
    // Helvetica widths in 1000-unit em square
    let metrics = HELVETICA_WIDTHS
        .iter()
        .find(|(ch, _)| *ch == c)
        .map_or(DEFAULT_HELVETICA_WIDTH, |(_, metrics)| *metrics);

    if bold {
        metrics.bold
    } else {
        metrics.regular
    }
}

/// Calculate text width in points for given string
fn text_width(text: &str, font_size: f32, bold: bool) -> f32 {
    let width_units: f32 = text.chars().map(|c| helvetica_char_width(c, bold)).sum();
    width_units * font_size / 1000.0
}

pub fn render(markdown: &str) -> Vec<u8> {
    let blocks = parse_markdown(markdown);

    let mut composer = PdfComposer::new();
    composer.render(&blocks);
    let pages = composer.finish();

    write_pdf(&pages)
}

#[derive(Debug, Clone)]
enum Inline {
    Text(String),
    Strong(Vec<Inline>),
    Link(Vec<Inline>),
    LineBreak,
}

#[derive(Debug, Clone)]
enum Block {
    Heading { level: u32, content: Vec<Inline> },
    Paragraph(Vec<Inline>),
    BulletList(Vec<Vec<Inline>>),
    Table(Vec<Vec<Vec<Inline>>>),
}

fn parse_markdown(input: &str) -> Vec<Block> {
    let parser = Parser::new_ext(
        input,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    );

    let mut state = MarkdownParser::new();
    for event in parser {
        state.handle_event(event);
    }
    state.finish()
}

#[derive(Debug)]
enum InlineContainer {
    Plain(Vec<Inline>),
    Strong(Vec<Inline>),
    Link(Vec<Inline>),
}

#[derive(Debug)]
struct TableState {
    rows: Vec<Vec<Vec<Inline>>>,
    current_row: Option<Vec<Vec<Inline>>>,
    in_head: bool,
}

impl TableState {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            current_row: None,
            in_head: false,
        }
    }

    fn start_row(&mut self) {
        self.current_row = Some(Vec::new());
    }

    fn push_cell(&mut self, cell: Vec<Inline>) {
        if let Some(row) = self.current_row.as_mut() {
            row.push(cell);
        }
    }

    fn finish_row(&mut self) {
        if let Some(mut row) = self.current_row.take() {
            // Include all non-empty rows (both header and body rows)
            if row.iter().any(|cell| !is_all_whitespace(cell)) {
                self.rows.push(std::mem::take(&mut row));
            }
        }
    }
}

#[derive(Debug)]
enum BlockContext {
    Paragraph,
    Heading(u32),
    List,
    ListItem,
    Table,
    TableHead,
    TableRow,
    TableCell,
    Ignored,
}

struct MarkdownParser {
    blocks: Vec<Block>,
    inline_stack: Vec<InlineContainer>,
    list_stack: Vec<Vec<Vec<Inline>>>,
    table_stack: Vec<TableState>,
    block_stack: Vec<BlockContext>,
}

impl MarkdownParser {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
            inline_stack: Vec::new(),
            list_stack: Vec::new(),
            table_stack: Vec::new(),
            block_stack: Vec::new(),
        }
    }

    fn handle_event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.handle_start_tag(&tag),
            Event::End(tag) => self.handle_end_tag(&tag),
            Event::Text(text) => push_text(&mut self.inline_stack, text.as_ref()),
            Event::Code(text) => {
                push_inline(&mut self.inline_stack, Inline::Text(text.into_string()));
            }
            Event::SoftBreak => push_text(&mut self.inline_stack, " "),
            Event::HardBreak => push_inline(&mut self.inline_stack, Inline::LineBreak),
            Event::Rule => self
                .blocks
                .push(Block::Paragraph(vec![Inline::Text(String::new())])),
            Event::Html(_) | Event::TaskListMarker(_) | Event::FootnoteReference(_) => {}
        }
    }

    fn handle_start_tag(&mut self, tag: &Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                self.inline_stack.push(InlineContainer::Plain(Vec::new()));
                self.block_stack.push(BlockContext::Paragraph);
            }
            Tag::Heading(level, _, _) => {
                self.inline_stack.push(InlineContainer::Plain(Vec::new()));
                self.block_stack
                    .push(BlockContext::Heading(heading_level_number(*level)));
            }
            Tag::List(_) => {
                self.list_stack.push(Vec::new());
                self.block_stack.push(BlockContext::List);
            }
            Tag::Item => {
                self.inline_stack.push(InlineContainer::Plain(Vec::new()));
                self.block_stack.push(BlockContext::ListItem);
            }
            Tag::Table(_) => {
                self.table_stack.push(TableState::new());
                self.block_stack.push(BlockContext::Table);
            }
            Tag::TableHead => {
                if let Some(state) = self.table_stack.last_mut() {
                    state.in_head = true;
                    state.start_row();
                }
                self.block_stack.push(BlockContext::TableHead);
            }
            Tag::TableRow => {
                if let Some(state) = self.table_stack.last_mut() {
                    state.start_row();
                }
                self.block_stack.push(BlockContext::TableRow);
            }
            Tag::TableCell => {
                self.inline_stack.push(InlineContainer::Plain(Vec::new()));
                self.block_stack.push(BlockContext::TableCell);
            }
            Tag::Strong => self.inline_stack.push(InlineContainer::Strong(Vec::new())),
            Tag::Link(_, _, _) => self.inline_stack.push(InlineContainer::Link(Vec::new())),
            Tag::Emphasis => self.inline_stack.push(InlineContainer::Plain(Vec::new())),
            _ => self.block_stack.push(BlockContext::Ignored),
        }
    }

    fn handle_end_tag(&mut self, tag: &Tag<'_>) {
        match tag {
            Tag::Paragraph => self.finish_paragraph(),
            Tag::Heading(_, _, _) => self.finish_heading(),
            Tag::List(_) => self.finish_list(),
            Tag::Item => self.finish_list_item(),
            Tag::Table(_) => self.finish_table(),
            Tag::TableHead => self.finish_table_head(),
            Tag::TableRow => self.finish_table_row(),
            Tag::TableCell => self.finish_table_cell(),
            Tag::Strong => self.finish_strong(),
            Tag::Link(_, _, _) => self.finish_link(),
            Tag::Emphasis => self.finish_emphasis(),
            _ => {
                self.block_stack.pop();
            }
        }
    }

    fn finish(self) -> Vec<Block> {
        self.blocks
    }

    fn finish_paragraph(&mut self) {
        if let (Some(InlineContainer::Plain(inlines)), Some(BlockContext::Paragraph)) =
            (self.inline_stack.pop(), self.block_stack.pop())
        {
            if !is_all_whitespace(&inlines) {
                self.blocks.push(Block::Paragraph(inlines));
            }
        }
    }

    fn finish_heading(&mut self) {
        if let (Some(InlineContainer::Plain(inlines)), Some(BlockContext::Heading(level))) =
            (self.inline_stack.pop(), self.block_stack.pop())
        {
            self.blocks.push(Block::Heading {
                level,
                content: inlines,
            });
        }
    }

    fn finish_list(&mut self) {
        if let Some(items) = self.list_stack.pop() {
            if matches!(self.block_stack.pop(), Some(BlockContext::List)) && !items.is_empty() {
                self.blocks.push(Block::BulletList(items));
            }
        }
    }

    fn finish_list_item(&mut self) {
        if let (Some(InlineContainer::Plain(inlines)), Some(BlockContext::ListItem)) =
            (self.inline_stack.pop(), self.block_stack.pop())
        {
            if let Some(list) = self.list_stack.last_mut() {
                if !is_all_whitespace(&inlines) {
                    list.push(inlines);
                }
            }
        }
    }

    fn finish_table(&mut self) {
        if let Some(state) = self.table_stack.pop() {
            if matches!(self.block_stack.pop(), Some(BlockContext::Table)) && !state.rows.is_empty()
            {
                self.blocks.push(Block::Table(state.rows));
            }
        }
    }

    fn finish_table_head(&mut self) {
        if matches!(self.block_stack.pop(), Some(BlockContext::TableHead)) {
            if let Some(state) = self.table_stack.last_mut() {
                state.finish_row();
                state.in_head = false;
            }
        }
    }

    fn finish_table_row(&mut self) {
        if matches!(self.block_stack.pop(), Some(BlockContext::TableRow)) {
            if let Some(state) = self.table_stack.last_mut() {
                state.finish_row();
            }
        }
    }

    fn finish_table_cell(&mut self) {
        if let (Some(InlineContainer::Plain(inlines)), Some(BlockContext::TableCell)) =
            (self.inline_stack.pop(), self.block_stack.pop())
        {
            if let Some(state) = self.table_stack.last_mut() {
                state.push_cell(inlines);
            }
        }
    }

    fn finish_strong(&mut self) {
        if let Some(InlineContainer::Strong(content)) = self.inline_stack.pop() {
            push_inline(&mut self.inline_stack, Inline::Strong(content));
        }
    }

    fn finish_link(&mut self) {
        if let Some(InlineContainer::Link(content)) = self.inline_stack.pop() {
            push_inline(&mut self.inline_stack, Inline::Link(content));
        }
    }

    fn finish_emphasis(&mut self) {
        if let Some(InlineContainer::Plain(content)) = self.inline_stack.pop() {
            push_inline(&mut self.inline_stack, Inline::Strong(content));
        }
    }
}

fn heading_level_number(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn push_inline(stack: &mut [InlineContainer], inline: Inline) {
    if let Some(container) = stack.last_mut() {
        match container {
            InlineContainer::Plain(inlines) | InlineContainer::Strong(inlines) => {
                inlines.push(inline);
            }
            InlineContainer::Link(content) => content.push(inline),
        }
    }
}

fn push_text(stack: &mut [InlineContainer], text: &str) {
    if let Some(container) = stack.last_mut() {
        match container {
            InlineContainer::Plain(inlines)
            | InlineContainer::Strong(inlines)
            | InlineContainer::Link(inlines) => {
                if let Some(Inline::Text(existing)) = inlines.last_mut() {
                    existing.push_str(text);
                } else {
                    inlines.push(Inline::Text(text.to_string()));
                }
            }
        }
    }
}

fn is_all_whitespace(inlines: &[Inline]) -> bool {
    inlines.iter().all(|inline| match inline {
        Inline::Text(text) => text.trim().is_empty(),
        Inline::LineBreak => true,
        Inline::Strong(children) | Inline::Link(children) => is_all_whitespace(children),
    })
}

#[derive(Debug, Clone)]
struct Segment {
    text: String,
    bold: bool,
}

#[derive(Debug, Clone)]
struct Line {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
struct PdfPage {
    content: String,
}

impl PdfPage {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn write_text(&mut self, x: f32, y: f32, font: FontFace, size: f32, text: &str) {
        if text.is_empty() {
            return;
        }

        let font_name = match font {
            FontFace::Regular => "F1",
            FontFace::Bold => "F2",
        };

        let escaped = escape_pdf_text(text);
        let _ = writeln!(
            self.content,
            "BT /{font_name} {size} Tf 1 0 0 1 {x:.2} {y:.2} Tm ({escaped}) Tj ET"
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FontFace {
    Regular,
    Bold,
}

struct PdfComposer {
    pages: Vec<PdfPage>,
    current: PdfPage,
    cursor_y: f32,
}

impl PdfComposer {
    fn new() -> Self {
        Self {
            pages: Vec::new(),
            current: PdfPage::new(),
            cursor_y: PAGE_HEIGHT - MARGIN_TOP,
        }
    }

    fn render(&mut self, blocks: &[Block]) {
        for block in blocks {
            match block {
                Block::Heading { level, content } => self.render_heading(*level, content),
                Block::Paragraph(content) => self.render_paragraph(content),
                Block::BulletList(items) => self.render_list(items),
                Block::Table(rows) => self.render_table(rows),
            }
        }
    }

    fn render_heading(&mut self, level: u32, content: &[Inline]) {
        let (size, spacing) = match level {
            1 => (HEADING1_FONT_SIZE, HEADING1_FONT_SIZE * LINE_SPACING_FACTOR),
            2 => (HEADING2_FONT_SIZE, HEADING2_FONT_SIZE * LINE_SPACING_FACTOR),
            _ => (HEADING3_FONT_SIZE, HEADING3_FONT_SIZE * LINE_SPACING_FACTOR),
        };

        self.ensure_space(spacing + 4.0);
        let text = plain_text(content);
        let text_w = text_width(&text, size, true);
        let x = if level == 1 {
            ((PAGE_WIDTH - text_w) / 2.0).max(MARGIN_HORIZONTAL)
        } else {
            MARGIN_HORIZONTAL
        };
        let y = self.cursor_y;
        self.current.write_text(x, y, FontFace::Bold, size, &text);
        self.cursor_y -= spacing;
        if level == 1 {
            self.cursor_y -= 8.0;
        } else if level == 2 {
            self.cursor_y -= 4.0;
        } else {
            self.cursor_y -= 3.0;
        }
    }

    fn render_paragraph(&mut self, content: &[Inline]) {
        let tokens = tokenize(content, false);
        let max_width = PAGE_WIDTH - 2.0 * MARGIN_HORIZONTAL;
        let lines = wrap_tokens(&tokens, max_width, BODY_FONT_SIZE);

        if lines.is_empty() {
            return;
        }

        for line in lines {
            let line_height = BODY_FONT_SIZE * LINE_SPACING_FACTOR;
            self.ensure_space(line_height);
            let y = self.cursor_y;
            self.write_line(&line, MARGIN_HORIZONTAL, y, BODY_FONT_SIZE);
            self.cursor_y -= line_height;
        }
        self.cursor_y -= 8.0;
    }

    fn render_list(&mut self, items: &[Vec<Inline>]) {
        for item in items {
            let tokens = tokenize(item, false);
            let available_width = PAGE_WIDTH - 2.0 * MARGIN_HORIZONTAL - BULLET_INDENT_POINTS;
            let lines = wrap_tokens(&tokens, available_width, BODY_FONT_SIZE);
            if lines.is_empty() {
                continue;
            }

            let line_height = BODY_FONT_SIZE * LINE_SPACING_FACTOR;
            for (idx, line) in lines.iter().enumerate() {
                self.ensure_space(line_height);
                let y = self.cursor_y;
                if idx == 0 {
                    self.current.write_text(
                        MARGIN_HORIZONTAL,
                        y,
                        FontFace::Regular,
                        BODY_FONT_SIZE,
                        "•",
                    );
                }
                self.write_line(
                    line,
                    MARGIN_HORIZONTAL + BULLET_INDENT_POINTS,
                    y,
                    BODY_FONT_SIZE,
                );
                self.cursor_y -= line_height;
            }
            self.cursor_y -= 2.0;
        }
        self.cursor_y -= 8.0;
    }

    fn render_table(&mut self, rows: &[Vec<Vec<Inline>>]) {
        if rows.is_empty() {
            return;
        }

        for row in rows {
            if row.len() < 2 {
                continue;
            }

            let left_text = plain_text(&row[0]);
            let right_text = plain_text(&row[1]);
            if left_text.trim().is_empty() && right_text.trim().is_empty() {
                continue;
            }

            let line_height = BODY_FONT_SIZE * LINE_SPACING_FACTOR;
            self.ensure_space(line_height);
            let y = self.cursor_y;

            let left_face = if contains_strong(&row[0]) {
                FontFace::Bold
            } else {
                FontFace::Regular
            };
            let right_face = if contains_strong(&row[1]) {
                FontFace::Bold
            } else {
                FontFace::Regular
            };

            self.current
                .write_text(MARGIN_HORIZONTAL, y, left_face, BODY_FONT_SIZE, &left_text);

            let right_width = text_width(&right_text, BODY_FONT_SIZE, right_face == FontFace::Bold);
            let right_x = (PAGE_WIDTH - MARGIN_HORIZONTAL - right_width).max(MARGIN_HORIZONTAL);
            self.current
                .write_text(right_x, y, right_face, BODY_FONT_SIZE, &right_text);

            self.cursor_y -= line_height;
        }
        self.cursor_y -= 8.0;
    }

    fn ensure_space(&mut self, required: f32) {
        if self.cursor_y - required < MARGIN_BOTTOM {
            self.finish_page();
        }
    }

    fn write_line(&mut self, line: &Line, start_x: f32, y: f32, size: f32) {
        let mut x = start_x;
        for segment in &line.segments {
            let font = if segment.bold {
                FontFace::Bold
            } else {
                FontFace::Regular
            };
            self.current.write_text(x, y, font, size, &segment.text);
            let advance = text_width(&segment.text, size, segment.bold);
            x += advance;
        }
    }

    fn finish_page(&mut self) {
        let old_page = std::mem::replace(&mut self.current, PdfPage::new());
        self.pages.push(old_page);
        self.cursor_y = PAGE_HEIGHT - MARGIN_TOP;
    }

    fn finish(mut self) -> Vec<PdfPage> {
        if !self.current.content.trim().is_empty() || self.pages.is_empty() {
            self.pages.push(self.current);
        }
        self.pages
    }
}

fn tokenize(inlines: &[Inline], bold_parent: bool) -> Vec<Token> {
    let mut tokens = Vec::new();
    collect_tokens(inlines, bold_parent, &mut tokens);
    tokens
}

fn collect_tokens(inlines: &[Inline], bold_parent: bool, tokens: &mut Vec<Token>) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                let mut buffer = String::new();
                for ch in text.chars() {
                    match ch {
                        ' ' | '\t' => {
                            if !buffer.is_empty() {
                                tokens.push(Token::Word {
                                    text: buffer.clone(),
                                    bold: bold_parent,
                                });
                                buffer.clear();
                            }
                            tokens.push(Token::Space);
                        }
                        '\n' => {
                            if !buffer.is_empty() {
                                tokens.push(Token::Word {
                                    text: buffer.clone(),
                                    bold: bold_parent,
                                });
                                buffer.clear();
                            }
                            tokens.push(Token::HardBreak);
                        }
                        _ => buffer.push(ch),
                    }
                }
                if !buffer.is_empty() {
                    tokens.push(Token::Word {
                        text: buffer,
                        bold: bold_parent,
                    });
                }
            }
            Inline::Strong(children) => collect_tokens(children, true, tokens),
            Inline::Link(children) => collect_tokens(children, bold_parent, tokens),
            Inline::LineBreak => tokens.push(Token::HardBreak),
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Word { text: String, bold: bool },
    Space,
    HardBreak,
}

fn wrap_tokens(tokens: &[Token], max_width: f32, font_size: f32) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut current_segments: Vec<Segment> = Vec::new();
    let mut current_width = 0.0f32;
    let mut pending_space = false;

    let mut push_line = |segments: &mut Vec<Segment>, width: &mut f32| {
        if !segments.is_empty() {
            lines.push(Line {
                segments: std::mem::take(segments),
            });
            *width = 0.0;
        }
    };

    for token in tokens {
        match token {
            Token::Space => {
                if current_width > 0.0 {
                    pending_space = true;
                }
            }
            Token::HardBreak => {
                push_line(&mut current_segments, &mut current_width);
                pending_space = false;
            }
            Token::Word { text, bold } => {
                let word_width = text_width(text, font_size, *bold);
                let space_width = if pending_space {
                    text_width(" ", font_size, false)
                } else {
                    0.0
                };
                let additional = word_width + space_width;

                if current_width > 0.0 && current_width + additional > max_width {
                    push_line(&mut current_segments, &mut current_width);
                    pending_space = false;
                }

                if pending_space && !current_segments.is_empty() {
                    append_segment(&mut current_segments, " ", false);
                    current_width += text_width(" ", font_size, false);
                    pending_space = false;
                }

                append_segment(&mut current_segments, text, *bold);
                current_width += word_width;
            }
        }
    }

    if !current_segments.is_empty() {
        lines.push(Line {
            segments: current_segments,
        });
    }

    lines
}

fn append_segment(segments: &mut Vec<Segment>, text: &str, bold: bool) {
    if text.is_empty() {
        return;
    }

    if let Some(last) = segments.last_mut() {
        if last.bold == bold {
            last.text.push_str(text);
            return;
        }
    }

    segments.push(Segment {
        text: text.to_string(),
        bold,
    });
}

fn plain_text(inlines: &[Inline]) -> String {
    let mut result = String::new();
    collect_plain_text(inlines, &mut result);
    result
}

fn contains_strong(inlines: &[Inline]) -> bool {
    inlines.iter().any(|inline| match inline {
        Inline::Strong(_) => true,
        Inline::Link(children) => contains_strong(children),
        _ => false,
    })
}

fn collect_plain_text(inlines: &[Inline], output: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => output.push_str(text),
            Inline::Strong(children) | Inline::Link(children) => {
                collect_plain_text(children, output);
            }
            Inline::LineBreak => output.push('\n'),
        }
    }
}

fn escape_pdf_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '(' | ')' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '\r' => {
                escaped.push(' ');
            }
            '•' => {
                // Bullet character: use octal escape for StandardEncoding position
                escaped.push_str("\\267");
            }
            _ => {
                // For non-ASCII characters, try to preserve them or use space
                if ch.is_ascii() || ch as u32 <= 255 {
                    escaped.push(ch);
                } else {
                    escaped.push('?');
                }
            }
        }
    }
    escaped
}

fn write_object(buffer: &mut Vec<u8>, offsets: &mut [usize], id: usize, content: &str) {
    offsets[id] = buffer.len();
    let _ = write!(buffer, "{id} 0 obj\n{content}\nendobj\n");
}

fn write_stream(buffer: &mut Vec<u8>, offsets: &mut [usize], id: usize, data: &str) {
    offsets[id] = buffer.len();
    let _ = write!(
        buffer,
        "{} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        id,
        data.len(),
        data
    );
}

fn write_pdf(pages: &[PdfPage]) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"%PDF-1.4\n");

    let page_count = pages.len().max(1);
    let base_objects = 2 + page_count;
    let font_regular_id = base_objects + 1;
    let font_bold_id = base_objects + 2;
    let content_start_id = font_bold_id + 1;

    let total_objects = base_objects + 2 + page_count;
    let mut offsets = vec![0usize; total_objects + 1];

    write_object(
        &mut buffer,
        &mut offsets,
        1,
        "<< /Type /Catalog /Pages 2 0 R >>",
    );

    let mut kids = String::new();
    for i in 0..page_count {
        let page_id = 3 + i;
        let _ = write!(kids, "{page_id} 0 R ");
    }
    write_object(
        &mut buffer,
        &mut offsets,
        2,
        &format!(
            "<< /Type /Pages /Count {} /Kids [{}] >>",
            page_count,
            kids.trim()
        ),
    );

    let mut content_ids = Vec::with_capacity(page_count);
    let mut next_content_id = content_start_id;

    for (index, _) in pages.iter().enumerate() {
        let page_id = 3 + index;
        let content_id = next_content_id;
        next_content_id += 1;
        content_ids.push(content_id);

        let page_dict = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.2} {PAGE_HEIGHT:.2}] /Resources << /Font << /F1 {font_regular_id} 0 R /F2 {font_bold_id} 0 R >> >> /Contents {content_id} 0 R >>"
        );
        write_object(&mut buffer, &mut offsets, page_id, &page_dict);
    }

    write_object(
        &mut buffer,
        &mut offsets,
        font_regular_id,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    );
    write_object(
        &mut buffer,
        &mut offsets,
        font_bold_id,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>",
    );

    for (page, content_id) in pages.iter().zip(content_ids.iter()) {
        write_stream(&mut buffer, &mut offsets, *content_id, &page.content);
    }

    let xref_offset = buffer.len();
    let _ = write!(buffer, "xref\n0 {}\n", total_objects + 1);
    buffer.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1).take(total_objects) {
        let _ = writeln!(buffer, "{offset:010} 00000 n ");
    }

    let _ = write!(
        buffer,
        "trailer<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        total_objects + 1,
        xref_offset
    );

    buffer
}
