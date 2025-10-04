use std::fmt::Write;
use std::io::Write as IoWrite;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};

const PAGE_WIDTH: f32 = 595.0;
const PAGE_HEIGHT: f32 = 842.0;
const MARGIN_HORIZONTAL: f32 = 40.0;
const MARGIN_TOP: f32 = 50.0;
const MARGIN_BOTTOM: f32 = 40.0;
const BODY_FONT_SIZE: f32 = 10.0;
const HEADING1_FONT_SIZE: f32 = 16.0;
const HEADING2_FONT_SIZE: f32 = 12.0;
const HEADING3_FONT_SIZE: f32 = 11.0;
const LINE_SPACING_FACTOR: f32 = 1.3;
const BULLET_INDENT_POINTS: f32 = 18.0;

/// Get character width for Helvetica font (in 1000 units, scale by font_size/1000)
fn helvetica_char_width(c: char, bold: bool) -> f32 {
    // Helvetica widths in 1000-unit em square
    // Common characters only; unknown chars default to 500 units
    match c {
        ' ' => 278.0,
        '!' => if bold { 333.0 } else { 278.0 },
        '"' => if bold { 474.0 } else { 355.0 },
        '#' => 556.0,
        '$' => 556.0,
        '%' => 889.0,
        '&' => if bold { 722.0 } else { 667.0 },
        '\'' => if bold { 278.0 } else { 191.0 },
        '(' => if bold { 333.0 } else { 333.0 },
        ')' => if bold { 333.0 } else { 333.0 },
        '*' => if bold { 389.0 } else { 389.0 },
        '+' => 584.0,
        ',' => if bold { 278.0 } else { 278.0 },
        '-' => if bold { 333.0 } else { 333.0 },
        '.' => if bold { 278.0 } else { 278.0 },
        '/' => if bold { 278.0 } else { 278.0 },
        '0'..='9' => 556.0,
        ':' => if bold { 333.0 } else { 278.0 },
        ';' => if bold { 333.0 } else { 278.0 },
        '<' => 584.0,
        '=' => 584.0,
        '>' => 584.0,
        '?' => if bold { 611.0 } else { 556.0 },
        '@' => if bold { 975.0 } else { 1015.0 },
        'A' => if bold { 722.0 } else { 667.0 },
        'B' => if bold { 722.0 } else { 667.0 },
        'C' => if bold { 722.0 } else { 722.0 },
        'D' => if bold { 722.0 } else { 722.0 },
        'E' => if bold { 667.0 } else { 667.0 },
        'F' => if bold { 611.0 } else { 611.0 },
        'G' => if bold { 778.0 } else { 778.0 },
        'H' => if bold { 722.0 } else { 722.0 },
        'I' => if bold { 278.0 } else { 278.0 },
        'J' => if bold { 556.0 } else { 500.0 },
        'K' => if bold { 722.0 } else { 667.0 },
        'L' => if bold { 611.0 } else { 556.0 },
        'M' => if bold { 833.0 } else { 833.0 },
        'N' => if bold { 722.0 } else { 722.0 },
        'O' => if bold { 778.0 } else { 778.0 },
        'P' => if bold { 667.0 } else { 667.0 },
        'Q' => if bold { 778.0 } else { 778.0 },
        'R' => if bold { 722.0 } else { 722.0 },
        'S' => if bold { 667.0 } else { 667.0 },
        'T' => if bold { 611.0 } else { 611.0 },
        'U' => if bold { 722.0 } else { 722.0 },
        'V' => if bold { 667.0 } else { 667.0 },
        'W' => if bold { 944.0 } else { 944.0 },
        'X' => if bold { 667.0 } else { 667.0 },
        'Y' => if bold { 667.0 } else { 667.0 },
        'Z' => if bold { 611.0 } else { 611.0 },
        '[' => if bold { 333.0 } else { 278.0 },
        '\\' => if bold { 278.0 } else { 278.0 },
        ']' => if bold { 333.0 } else { 278.0 },
        '^' => if bold { 581.0 } else { 469.0 },
        '_' => if bold { 556.0 } else { 556.0 },
        '`' => if bold { 333.0 } else { 222.0 },
        'a' => if bold { 556.0 } else { 556.0 },
        'b' => if bold { 611.0 } else { 556.0 },
        'c' => if bold { 556.0 } else { 500.0 },
        'd' => if bold { 611.0 } else { 556.0 },
        'e' => if bold { 556.0 } else { 556.0 },
        'f' => if bold { 333.0 } else { 278.0 },
        'g' => if bold { 611.0 } else { 556.0 },
        'h' => if bold { 611.0 } else { 556.0 },
        'i' => if bold { 278.0 } else { 222.0 },
        'j' => if bold { 278.0 } else { 222.0 },
        'k' => if bold { 556.0 } else { 500.0 },
        'l' => if bold { 278.0 } else { 222.0 },
        'm' => if bold { 889.0 } else { 833.0 },
        'n' => if bold { 611.0 } else { 556.0 },
        'o' => if bold { 611.0 } else { 556.0 },
        'p' => if bold { 611.0 } else { 556.0 },
        'q' => if bold { 611.0 } else { 556.0 },
        'r' => if bold { 389.0 } else { 333.0 },
        's' => if bold { 556.0 } else { 500.0 },
        't' => if bold { 333.0 } else { 278.0 },
        'u' => if bold { 611.0 } else { 556.0 },
        'v' => if bold { 556.0 } else { 500.0 },
        'w' => if bold { 778.0 } else { 722.0 },
        'x' => if bold { 556.0 } else { 500.0 },
        'y' => if bold { 556.0 } else { 500.0 },
        'z' => if bold { 500.0 } else { 500.0 },
        '{' => if bold { 389.0 } else { 334.0 },
        '|' => if bold { 280.0 } else { 260.0 },
        '}' => if bold { 389.0 } else { 334.0 },
        '~' => if bold { 584.0 } else { 584.0 },
        '•' => if bold { 350.0 } else { 350.0 },
        _ => if bold { 556.0 } else { 500.0 }, // Default for unknown chars
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
    let mut blocks = Vec::new();
    let parser = Parser::new_ext(
        input,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    );

    let mut inline_stack: Vec<InlineContainer> = Vec::new();
    let mut list_stack: Vec<Vec<Vec<Inline>>> = Vec::new();
    let mut table_stack: Vec<TableState> = Vec::new();
    let mut block_stack: Vec<BlockContext> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    inline_stack.push(InlineContainer::Plain(Vec::new()));
                    block_stack.push(BlockContext::Paragraph);
                }
                Tag::Heading(level, _, _) => {
                    inline_stack.push(InlineContainer::Plain(Vec::new()));
                    block_stack.push(BlockContext::Heading(heading_level_number(level)));
                }
                Tag::List(_) => {
                    list_stack.push(Vec::new());
                    block_stack.push(BlockContext::List);
                }
                Tag::Item => {
                    inline_stack.push(InlineContainer::Plain(Vec::new()));
                    block_stack.push(BlockContext::ListItem);
                }
                Tag::Table(_) => {
                    table_stack.push(TableState::new());
                    block_stack.push(BlockContext::Table);
                }
                Tag::TableHead => {
                    if let Some(state) = table_stack.last_mut() {
                        state.in_head = true;
                        state.start_row(); // Start a row for header cells
                    }
                    block_stack.push(BlockContext::TableHead);
                }
                Tag::TableRow => {
                    if let Some(state) = table_stack.last_mut() {
                        state.start_row();
                    }
                    block_stack.push(BlockContext::TableRow);
                }
                Tag::TableCell => {
                    inline_stack.push(InlineContainer::Plain(Vec::new()));
                    block_stack.push(BlockContext::TableCell);
                }
                Tag::Strong => inline_stack.push(InlineContainer::Strong(Vec::new())),
                Tag::Link(_, _, _) => inline_stack.push(InlineContainer::Link(Vec::new())),
                Tag::Emphasis => inline_stack.push(InlineContainer::Plain(Vec::new())),
                _ => block_stack.push(BlockContext::Ignored),
            },
            Event::End(tag) => match tag {
                Tag::Paragraph => {
                    if let Some(InlineContainer::Plain(inlines)) = inline_stack.pop() {
                        if let Some(BlockContext::Paragraph) = block_stack.pop() {
                            if !is_all_whitespace(&inlines) {
                                blocks.push(Block::Paragraph(inlines));
                            }
                        }
                    }
                }
                Tag::Heading(_, _, _) => {
                    if let Some(InlineContainer::Plain(inlines)) = inline_stack.pop() {
                        if let Some(BlockContext::Heading(level)) = block_stack.pop() {
                            blocks.push(Block::Heading {
                                level,
                                content: inlines,
                            });
                        }
                    }
                }
                Tag::List(_) => {
                    if let Some(items) = list_stack.pop() {
                        if let Some(BlockContext::List) = block_stack.pop() {
                            if !items.is_empty() {
                                blocks.push(Block::BulletList(items));
                            }
                        }
                    }
                }
                Tag::Item => {
                    if let Some(InlineContainer::Plain(inlines)) = inline_stack.pop() {
                        if let Some(BlockContext::ListItem) = block_stack.pop() {
                            if let Some(list) = list_stack.last_mut() {
                                if !is_all_whitespace(&inlines) {
                                    list.push(inlines);
                                }
                            }
                        }
                    }
                }
                Tag::Table(_) => {
                    if let Some(state) = table_stack.pop() {
                        if let Some(BlockContext::Table) = block_stack.pop() {
                            if !state.rows.is_empty() {
                                blocks.push(Block::Table(state.rows));
                            }
                        }
                    }
                }
                Tag::TableHead => {
                    if let Some(BlockContext::TableHead) = block_stack.pop() {
                        if let Some(state) = table_stack.last_mut() {
                            state.finish_row(); // Finish the header row
                            state.in_head = false;
                        }
                    }
                }
                Tag::TableRow => {
                    if let Some(BlockContext::TableRow) = block_stack.pop() {
                        if let Some(state) = table_stack.last_mut() {
                            state.finish_row();
                        }
                    }
                }
                Tag::TableCell => {
                    if let Some(InlineContainer::Plain(inlines)) = inline_stack.pop() {
                        if let Some(BlockContext::TableCell) = block_stack.pop() {
                            if let Some(state) = table_stack.last_mut() {
                                state.push_cell(inlines);
                            }
                        }
                    }
                }
                Tag::Strong => {
                    if let Some(InlineContainer::Strong(content)) = inline_stack.pop() {
                        push_inline(&mut inline_stack, Inline::Strong(content));
                    }
                }
                Tag::Link(_, _, _) => {
                    if let Some(InlineContainer::Link(content)) = inline_stack.pop() {
                        push_inline(&mut inline_stack, Inline::Link(content));
                    }
                }
                Tag::Emphasis => {
                    if let Some(InlineContainer::Plain(content)) = inline_stack.pop() {
                        push_inline(&mut inline_stack, Inline::Strong(content));
                    }
                }
                _ => {
                    block_stack.pop();
                }
            },
            Event::Text(text) => push_text(&mut inline_stack, &text),
            Event::Code(text) => push_inline(&mut inline_stack, Inline::Text(text.into_string())),
            Event::SoftBreak => push_text(&mut inline_stack, " "),
            Event::HardBreak => push_inline(&mut inline_stack, Inline::LineBreak),
            Event::Rule => {
                blocks.push(Block::Paragraph(vec![Inline::Text(String::new())]));
            }
            Event::Html(_) | Event::TaskListMarker(_) | Event::FootnoteReference(_) => {}
        }
    }

    blocks
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
                self.rows.push(row.drain(..).collect());
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

fn push_inline(stack: &mut Vec<InlineContainer>, inline: Inline) {
    if let Some(container) = stack.last_mut() {
        match container {
            InlineContainer::Plain(inlines) | InlineContainer::Strong(inlines) => {
                inlines.push(inline)
            }
            InlineContainer::Link(content) => content.push(inline),
        }
    }
}

fn push_text(stack: &mut Vec<InlineContainer>, text: &str) {
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
        Inline::Strong(children) => is_all_whitespace(children),
        Inline::Link(children) => is_all_whitespace(children),
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
            "BT /{} {} Tf 1 0 0 1 {:.2} {:.2} Tm ({}) Tj ET",
            font_name, size, x, y, escaped
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
            self.cursor_y -= 5.0;
        } else {
            self.cursor_y -= 2.0;
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
        self.cursor_y -= 3.0;
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
        self.cursor_y -= 3.0;
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
        self.cursor_y -= 3.0;
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
            Inline::Strong(children) => collect_plain_text(children, output),
            Inline::Link(children) => collect_plain_text(children, output),
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
    let _ = write!(buffer, "{} 0 obj\n{}\nendobj\n", id, content);
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
        let _ = write!(kids, "{} 0 R ", page_id);
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
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /Font << /F1 {} 0 R /F2 {} 0 R >> >> /Contents {} 0 R >>",
            PAGE_WIDTH, PAGE_HEIGHT, font_regular_id, font_bold_id, content_id
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
    for id in 1..=total_objects {
        let offset = offsets[id];
        let _ = write!(buffer, "{:010} 00000 n \n", offset);
    }

    let _ = write!(
        buffer,
        "trailer<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        total_objects + 1,
        xref_offset
    );

    buffer
}
