use super::sanitizer::{should_rebase_url, Sanitizer, SlashPath};
use crate::renderer::RawMessageWriter;
use aho_corasick::AhoCorasick;
use emojis::Emoji;
use memchr::{memchr_iter, Memchr};
use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Options,
    Parser, Tag, TagEnd,
};
use std::collections::HashMap;
use std::io::{Read, Result, Write};
use std::iter::Peekable;
use std::marker::PhantomData;
use std::path::Path;

pub type Range = std::ops::Range<usize>;

pub trait TextVisitor: Default {
    fn visit(&mut self, text: &str, range: &Range);
}

impl TextVisitor for () {
    fn visit(&mut self, _text: &str, _range: &Range) {}
}

#[derive(Clone, Copy, Debug)]
pub enum TokenKind {
    Normal,
    MatchOther,
    MatchCurrent,
    MatchOtherStart,
    MatchCurrentStart,
}

impl TokenKind {
    fn tag(self) -> &'static str {
        match self {
            Self::MatchOther => "match",
            Self::MatchCurrent => "match-current",
            Self::MatchOtherStart => "match-start",
            Self::MatchCurrentStart => "match-current-start",
            Self::Normal => unreachable!(),
        }
    }
}

pub trait TextTokenizer {
    fn tokenize<'t>(&mut self, text: &'t str, range: &Range) -> (TokenKind, &'t str);
}

impl TextTokenizer for () {
    fn tokenize<'t>(&mut self, text: &'t str, _range: &Range) -> (TokenKind, &'t str) {
        (TokenKind::Normal, text)
    }
}

// TODO: Use `str::floor_char_boundary` when it is stabilized.
// https://doc.rust-lang.org/std/primitive.str.html#method.floor_char_boundary
#[inline]
fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[derive(Default)]
pub struct MarkdownContent {
    source: String,
    base_dir: SlashPath,
}

impl MarkdownContent {
    pub fn new(source: String, base_dir: Option<&Path>) -> Self {
        let base_dir =
            if let Some(path) = base_dir { SlashPath::from(path) } else { SlashPath::default() };
        Self { source, base_dir }
    }

    pub fn modified_utf8_offset(&self, new: &Self) -> Option<usize> {
        let (prev_source, new_source) = (self.source.as_str(), new.source.as_str());
        // Offset must be UTF-8 aware to split text tokens correctly. If finding modified byte offset on a byte-by-byte
        // basis, the offset may point at the middle of UTF-8 character sequence.
        // For example, when a text 'あ' is modified to 'い',
        // - あ: 0xE3 0x81 0x82
        // - い: 0xE3 0x81 0x84
        // The first two bytes are the same. So the byte offset is 2 and it points at the middle of the sequence.
        // `MarkdownParser` will try to split the text at this position and will crash.
        //
        // Note: Iterating UTF-8 character indices with `str::char_indices` is slower than iterating bytes and adjusting
        // the byte offset to the UTF-8 character boundary. In addition, it is 8~10x faster to search 32 bytes chunk
        // index at first then search the byte index within the chunk rather than searching the index byte-by-byte.
        // - Benchmark:  https://github.com/rhysd/misc/tree/master/rust_bench/str_utf8_aware_offset
        // - Discussion: https://users.rust-lang.org/t/how-to-find-common-prefix-of-two-byte-slices-effectively/25815
        const CHUNK_SIZE: usize = 32;
        let prev = prev_source.as_bytes();
        let new = new_source.as_bytes();
        let offset = prev
            .chunks_exact(CHUNK_SIZE)
            .zip(new.chunks_exact(CHUNK_SIZE))
            .take_while(|(x, y)| x == y)
            .count()
            * CHUNK_SIZE;
        let index =
            offset + prev[offset..].iter().zip(&new[offset..]).take_while(|(x, y)| x == y).count();
        let min_len = prev.len().min(new.len());
        if index == min_len {
            return (prev.len() != new.len()).then_some(min_len);
        }
        Some(floor_char_boundary(new_source, index))
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty() && self.base_dir.is_empty()
    }
}

pub struct MarkdownParser<'input, V: TextVisitor, T: TextTokenizer> {
    parser: Parser<'input>,
    base_dir: &'input SlashPath,
    offset: Option<usize>,
    text_tokenizer: T,
    _phantom: PhantomData<V>,
}

impl<'input, V: TextVisitor, T: TextTokenizer> MarkdownParser<'input, V, T> {
    pub fn new(content: &'input MarkdownContent, offset: Option<usize>, text_tokenizer: T) -> Self {
        let mut options = Options::empty();
        options.insert(
            Options::ENABLE_STRIKETHROUGH
                | Options::ENABLE_FOOTNOTES
                | Options::ENABLE_TABLES
                | Options::ENABLE_TASKLISTS
                | Options::ENABLE_MATH
                | Options::ENABLE_GFM,
        );
        let parser = Parser::new_ext(&content.source, options);
        let base_dir = &content.base_dir;
        Self { parser, base_dir, offset, text_tokenizer, _phantom: PhantomData }
    }
}

// Note: Build raw JavaScript expression which is evaluated to the render tree encoded as JSON value.
// This expression will be evaluated via `receive(JSON.parse('{"kind":"render_tree",...}'))` by renderer.
impl<'input, V: TextVisitor, T: TextTokenizer> RawMessageWriter for MarkdownParser<'input, V, T> {
    type Output = V;

    fn write_to(self, writer: impl Write) -> Result<Self::Output> {
        let mut enc =
            RenderTreeEncoder::new(writer, self.base_dir, self.offset, self.text_tokenizer);
        enc.out.write_all(br#"JSON.parse('{"kind":"render_tree","tree":"#)?;
        enc.push(self.parser)?;
        enc.out.write_all(b"}')")?;
        Ok(enc.text_visitor)
    }
}

// To know the format of JSON value, see type definitions in web/ipc.ts

enum TableState {
    Head,
    Row,
}

// Note: Be careful, this function is called in the hot loop on encoding texts
#[inline]
#[allow(clippy::just_underscores_and_digits)]
fn encode_string_byte(mut out: impl Write, b: u8) -> Result<()> {
    const BB: u8 = b'b'; // \x08
    const TT: u8 = b't'; // \x09
    const NN: u8 = b'n'; // \x0a
    const FF: u8 = b'f'; // \x0c
    const RR: u8 = b'r'; // \x0d
    const DQ: u8 = b'"'; // \x22
    const SQ: u8 = b'\''; // \x27
    const BS: u8 = b'\\'; // \x5c
    const XX: u8 = 1; // \x00...\x1f non-printable
    const __: u8 = 0;

    #[rustfmt::skip]
    const ESCAPE_TABLE: [u8; 256] = [
    //   0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        XX, XX, XX, XX, XX, XX, XX, XX, BB, TT, NN, XX, FF, RR, XX, XX, // 0
        XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, // 1
        __, __, DQ, __, __, __, __, SQ, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, XX, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ];

    match ESCAPE_TABLE[b as usize] {
        __ => out.write_all(&[b]),
        BS => out.write_all(br"\\\\"), // Escape twice for JS and JSON (\\\\ → \\ → \)
        SQ => out.write_all(br"\'"), // JSON string will be put in '...' JS string. ' needs to be escaped
        XX => write!(out, r"\\u{:04x}", b),
        b => out.write_all(&[b'\\', b'\\', b]), // Escape \ itself: JSON.parse('\\n')
    }
}

struct StringContentEncoder<W: Write>(W);

impl<W: Write> Write for StringContentEncoder<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for b in buf.iter().copied() {
            encode_string_byte(&mut self.0, b)?;
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

// XXX: Items inside inline HTML are treated as raw texts
// See src/markdown/testdata/inline_items_nested_in_inline_html.md
struct InlineHtmlReader<'events, 'input, I: Iterator> {
    current: CowStr<'input>,
    index: usize,
    events: &'events mut Peekable<I>,
    tag_stack: u32,
    child_stack: u32,
}

impl<'events, 'input, I> InlineHtmlReader<'events, 'input, I>
where
    I: Iterator<Item = (Event<'input>, Range)>,
{
    fn new(current: CowStr<'input>, events: &'events mut Peekable<I>) -> Self {
        let tag_stack = if current.starts_with("</") { 0 } else { 1 };
        Self { current, index: 0, events, tag_stack, child_stack: 0 }
    }

    fn read_byte(&mut self) -> Option<u8> {
        while self.current.len() <= self.index {
            if self.tag_stack == 0 {
                return None;
            }

            match &self.events.peek()?.0 {
                Event::InlineHtml(html) if html.starts_with("</") => {
                    if let Some(popped) = self.tag_stack.checked_sub(1) {
                        self.tag_stack = popped;
                    } else {
                        return None;
                    }
                }
                Event::InlineHtml(_) => {
                    self.tag_stack += 1;
                }
                Event::Start(_) => {
                    self.child_stack += 1;
                }
                Event::End(_) => {
                    if let Some(popped) = self.child_stack.checked_sub(1) {
                        self.child_stack = popped;
                    } else {
                        return None;
                    }
                }
                _ => {}
            }

            self.current = match self.events.next().unwrap().0 {
                Event::InlineHtml(html) => html,
                Event::Text(text) => text,
                Event::Code(text) => text,
                Event::SoftBreak => " ".into(),
                Event::HardBreak => "\n".into(),
                Event::DisplayMath(text) => text,
                Event::InlineMath(text) => text,
                _ => continue,
            };
            self.index = 0;
        }

        let b = self.current.as_bytes()[self.index];
        self.index += 1;
        Some(b)
    }
}

impl<'events, 'input, I> Read for InlineHtmlReader<'events, 'input, I>
where
    I: Iterator<Item = (Event<'input>, Range)>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        for (i, dest) in buf.iter_mut().enumerate() {
            if let Some(b) = self.read_byte() {
                *dest = b;
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }
}

struct HtmlBlockReader<'input, I: Iterator<Item = (Event<'input>, Range)>> {
    current: CowStr<'input>,
    index: usize,
    events: I,
    end: bool,
}

impl<'input, I: Iterator<Item = (Event<'input>, Range)>> HtmlBlockReader<'input, I> {
    fn new(events: I) -> Self {
        Self { current: "".into(), index: 0, events, end: false }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.end {
            return None;
        }

        // Current event was consumed. Fetch next event or return `None` at the end of the block.
        while self.current.len() <= self.index {
            self.current = match self.events.next()?.0 {
                Event::End(TagEnd::HtmlBlock) => {
                    self.end = true;
                    return None;
                }
                Event::Html(html) => html,
                // Text event is emitted when tags are preceded by spaces like " <p></p>"
                Event::Text(text) => text,
                event => unreachable!("unexpected event: {event:?}"),
            };
            self.index = 0;
        }

        let b = self.current.as_bytes()[self.index];
        self.index += 1;
        Some(b)
    }
}

impl<'input, I: Iterator<Item = (Event<'input>, Range)>> Read for HtmlBlockReader<'input, I> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        for (i, dest) in buf.iter_mut().enumerate() {
            if let Some(b) = self.read_byte() {
                *dest = b;
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }
}

struct RenderTreeEncoder<'input, W: Write, V: TextVisitor, T: TextTokenizer> {
    out: W,
    base_dir: &'input SlashPath,
    table: TableState,
    is_start: bool,
    ids: HashMap<CowStr<'input>, usize>,
    modified: Option<usize>,
    text_visitor: V,
    text_tokenizer: T,
    autolinker: Autolinker,
    sanitizer: Sanitizer<'input>,
}

impl<'input, W: Write, V: TextVisitor, T: TextTokenizer> RenderTreeEncoder<'input, W, V, T> {
    fn new(w: W, base_dir: &'input SlashPath, modified: Option<usize>, text_tokenizer: T) -> Self {
        Self {
            out: w,
            base_dir,
            table: TableState::Head,
            is_start: true,
            ids: HashMap::new(),
            modified,
            text_visitor: V::default(),
            text_tokenizer,
            autolinker: Autolinker::default(),
            sanitizer: Sanitizer::new(base_dir),
        }
    }

    fn push(&mut self, parser: Parser<'input>) -> Result<()> {
        self.out.write_all(b"[")?;
        self.events(parser)?;
        // Modified offset was not consumed by any text, it would mean that some non-text parts after any text were
        // modified. As a fallback, set 'modified' marker after the last text.
        if self.modified.is_some() {
            self.tag("modified")?;
            self.out.write_all(b"}")?;
        }
        self.out.write_all(b"]")
    }

    fn string_content(&mut self, s: &str) -> Result<()> {
        for b in s.as_bytes().iter().copied() {
            encode_string_byte(&mut self.out, b)?;
        }
        Ok(())
    }

    fn string(&mut self, s: &str) -> Result<()> {
        self.out.write_all(b"\"")?;
        self.string_content(s)?;
        self.out.write_all(b"\"")
    }

    fn alignment(&mut self, a: Alignment) -> Result<()> {
        self.out.write_all(match a {
            Alignment::None => b"null",
            Alignment::Left => br#""left""#,
            Alignment::Center => br#""center""#,
            Alignment::Right => br#""right""#,
        })
    }

    fn id(&mut self, name: CowStr<'input>) -> usize {
        let new = self.ids.len() + 1;
        *self.ids.entry(name).or_insert(new)
    }

    fn comma(&mut self) -> Result<()> {
        if !self.is_start {
            self.out.write_all(b",")?;
        } else {
            self.is_start = false;
        }
        Ok(())
    }

    fn tag(&mut self, name: &str) -> Result<()> {
        self.comma()?;
        write!(self.out, r#"{{"t":"{}""#, name)
    }

    fn text_tokens(&mut self, mut input: &str, mut range: Range) -> Result<()> {
        use TokenKind::*;

        while !input.is_empty() {
            let (token, text) = self.text_tokenizer.tokenize(input, &range);
            match token {
                Normal => {
                    self.comma()?;
                    self.string(text)?;
                }
                MatchOther | MatchCurrent | MatchOtherStart | MatchCurrentStart => {
                    self.tag(token.tag())?;
                    self.children_begin()?;
                    self.string(text)?;
                    self.tag_end()?;
                }
            }
            input = &input[text.len()..];
            range.start += text.len();
        }

        Ok(())
    }

    fn text(&mut self, text: &str, range: Range) -> Result<()> {
        self.text_visitor.visit(text, &range);

        let Some(offset) = self.modified else {
            return self.text_tokens(text, range);
        };

        let Range { start, end } = range;
        if end < offset {
            return self.text_tokens(text, range);
        }

        // Handle the last modified offset with this text token
        self.modified = None;
        log::debug!("Handling last modified offset: {:?}", offset);

        if offset <= start {
            self.tag("modified")?;
            self.out.write_all(b"}")?;
            self.text_tokens(text, range)
        } else if end == offset {
            self.text_tokens(text, range)?;
            self.tag("modified")?;
            self.out.write_all(b"}")
        } else {
            let i = offset - start;
            self.text_tokens(&text[..i], start..offset)?;
            self.tag("modified")?;
            self.out.write_all(b"}")?;
            self.text_tokens(&text[i..], offset..end)
        }
    }

    fn emoji_text(&mut self, text: &str, range: Range) -> Result<()> {
        let mut start = range.start;
        for token in EmojiTokenizer::new(text) {
            match token {
                EmojiToken::Text(text) => {
                    if !text.is_empty() {
                        self.text(text, start..start + text.len())?;
                        start += text.len();
                    }
                }
                EmojiToken::Emoji(emoji, len) => {
                    self.tag("emoji")?;
                    self.out.write_all(br#","name":"#)?;
                    self.string(emoji.name())?;
                    self.children_begin()?;
                    self.string(emoji.as_str())?;
                    self.tag_end()?;
                    start += len;
                }
            }
        }
        // Note: When some escaped text is included in input like "&amp;", `start == range.end` invariant is violated here.
        // That's OK because pulldown-cmark tokenizes any escaped text as small as possible to reduce extra heap allocation.
        // For instance "foo &amp; bar" is tokenized into three events Text("foo "), Text("&"), Test(" bar"). It means that
        // any escaped charactor is followed by no text within the token.
        Ok(())
    }

    fn autolink_text(&mut self, mut text: &str, range: Range) -> Result<()> {
        let Range { mut start, end } = range;
        while let Some((s, e)) = self.autolinker.find_autolink(text) {
            if s > 0 {
                self.emoji_text(&text[..s], start..start + s)?;
            }

            let url = &text[s..e];
            log::debug!("Auto-linking URL: {}", url);
            self.tag("a")?;
            self.out.write_all(br#","auto":true,"href":"#)?;
            self.string(url)?;
            self.children_begin()?;
            self.text(url, start + s..start + e)?;
            self.tag_end()?;

            text = &text[e..];
            start += e;
        }

        if !text.is_empty() {
            self.emoji_text(text, start..end)?;
        }

        Ok(())
    }

    fn events(&mut self, parser: Parser<'input>) -> Result<()> {
        let mut in_code_block = false;
        let mut in_auto_link = false;
        let mut in_link = false;

        let mut events = parser.into_offset_iter().peekable();
        while let Some((event, range)) = events.next() {
            match event {
                Event::Start(tag) => {
                    use Tag::*;
                    match tag {
                        Paragraph => {
                            self.tag("p")?;
                        }
                        Heading { level, .. } => {
                            self.tag("h")?;

                            let level: u8 = match level {
                                HeadingLevel::H1 => 1,
                                HeadingLevel::H2 => 2,
                                HeadingLevel::H3 => 3,
                                HeadingLevel::H4 => 4,
                                HeadingLevel::H5 => 5,
                                HeadingLevel::H6 => 6,
                            };
                            write!(self.out, r#","level":{}"#, level)?;
                        }
                        Table(alignments) => {
                            self.tag("table")?;

                            self.out.write_all(br#","align":["#)?;
                            let mut alignments = alignments.into_iter();
                            if let Some(a) = alignments.next() {
                                self.alignment(a)?;
                            }
                            for a in alignments {
                                self.out.write_all(b",")?;
                                self.alignment(a)?;
                            }
                            self.out.write_all(b"]")?;
                        }
                        TableHead => {
                            self.table = TableState::Head;
                            self.tag("thead")?;
                            self.children_begin()?;
                            self.tag("tr")?;
                        }
                        TableRow => {
                            self.table = TableState::Row;
                            self.tag("tr")?;
                        }
                        TableCell => {
                            let tag = match self.table {
                                TableState::Head => "th",
                                TableState::Row => "td",
                            };
                            self.tag(tag)?;
                        }
                        BlockQuote(None) => self.tag("blockquote")?,
                        BlockQuote(Some(kind)) => {
                            let kind = match kind {
                                BlockQuoteKind::Warning => "warning",
                                BlockQuoteKind::Important => "important",
                                BlockQuoteKind::Caution => "caution",
                                BlockQuoteKind::Note => "note",
                                BlockQuoteKind::Tip => "tip",
                            };
                            self.tag("alert")?;
                            write!(self.out, r#","kind":"{}""#, kind)?;
                        }
                        CodeBlock(info) => {
                            self.tag("pre")?;
                            self.children_begin()?;
                            self.tag("code")?;
                            if let CodeBlockKind::Fenced(info) = info {
                                if let Some(lang) = info.split(' ').next() {
                                    if !lang.is_empty() {
                                        self.out.write_all(br#","lang":"#)?;
                                        self.string(lang)?;
                                    }
                                }
                            }
                            in_code_block = true;
                        }
                        List(Some(1)) => self.tag("ol")?,
                        List(Some(start)) => {
                            self.tag("ol")?;
                            write!(self.out, r#","start":{}"#, start)?;
                        }
                        List(None) => self.tag("ul")?,
                        Item => {
                            if let Some((Event::TaskListMarker(_), _)) = events.peek() {
                                self.tag("task-list")?;
                            } else {
                                self.tag("li")?;
                            }
                        }
                        Emphasis => self.tag("em")?,
                        Strong => self.tag("strong")?,
                        Strikethrough => self.tag("del")?,
                        Link { link_type: LinkType::Autolink, .. } => {
                            in_auto_link = true;
                            // Ignore autolink since it is linked by `Autolinker`
                            continue;
                        }
                        Link { link_type, dest_url, title, .. } => {
                            self.tag("a")?;

                            self.out.write_all(br#","href":"#)?;
                            match link_type {
                                LinkType::Email => {
                                    self.out.write_all(b"\"mailto:")?;
                                    self.string_content(&dest_url)?;
                                    self.out.write_all(b"\"")?;
                                }
                                _ => self.rebase_link(&dest_url)?,
                            }

                            if !title.is_empty() {
                                self.out.write_all(br#","title":"#)?;
                                self.string(&title)?;
                            }

                            in_link = true;
                        }
                        Image { dest_url, title, .. } => {
                            self.tag("img")?;

                            if !title.is_empty() {
                                self.out.write_all(br#","title":"#)?;
                                self.string(&title)?;
                            }

                            self.out.write_all(br#","src":"#)?;
                            self.rebase_link(&dest_url)?;
                        }
                        HtmlBlock => {
                            self.tag("html")?;
                            self.out.write_all(br#","raw":""#)?;

                            let dst = StringContentEncoder(&mut self.out);
                            let src = HtmlBlockReader::new(&mut events);
                            self.sanitizer.clean(dst, src)?;

                            self.out.write_all(br#""}"#)?;
                            // Unlike other tags, `HtmlBlockReader consumes all events until `TagEnd::HtmlBlock`
                            continue;
                        }
                        FootnoteDefinition(name) => {
                            self.tag("fn-def")?;

                            if !name.is_empty() {
                                self.out.write_all(br#","name":"#)?;
                                self.string(&name)?;
                            }

                            let id = self.id(name);
                            write!(self.out, r#","id":{}"#, id)?;
                        }
                        DefinitionList
                        | DefinitionListDefinition
                        | DefinitionListTitle
                        | MetadataBlock(_) => unreachable!("disabled markdown feature"),
                    }

                    // Tag element must have its children (maybe empty)
                    self.children_begin()?;
                }
                Event::End(tag_end) => {
                    use TagEnd::*;
                    match tag_end {
                        Link if in_auto_link => in_auto_link = false,
                        Link => {
                            in_link = false;
                            self.tag_end()?
                        }
                        Paragraph | Heading(_) | TableRow | TableCell | BlockQuote(_) | List(_)
                        | Item | Emphasis | Strong | Strikethrough | Image | FootnoteDefinition => {
                            self.tag_end()?
                        }
                        CodeBlock => {
                            in_code_block = false;
                            self.tag_end()?;
                            self.tag_end()?;
                        }
                        Table => {
                            self.tag_end()?;
                            self.tag_end()?;
                        }
                        TableHead => {
                            self.tag_end()?;
                            self.tag_end()?;
                            self.tag("tbody")?;
                            self.children_begin()?;
                        }
                        HtmlBlock => unreachable!(), // This event is handled in `Tag::HtmlBlock` event using `HtmlBlockReader`
                        DefinitionList
                        | DefinitionListDefinition
                        | DefinitionListTitle
                        | MetadataBlock(_) => unreachable!("disabled markdown feature"), // This option is not enabled
                    }
                }
                Event::Text(text) if in_code_block || in_link => self.text(&text, range)?,
                Event::Text(text) => self.autolink_text(&text, range)?,
                Event::Code(text) => {
                    let pad = (range.len() - text.len()) / 2;
                    let inner_range = (range.start + pad)..(range.end - pad);
                    self.tag("code")?;
                    self.children_begin()?;
                    self.text(&text, inner_range)?;
                    self.tag_end()?;
                }
                // Inline HTML inside blockquote is emitted as Event::Html event wrongly. Uncomment the below line when
                // the following issue is fixed and the fix is released.
                // https://github.com/pulldown-cmark/pulldown-cmark/issues/960
                //
                // Event::Html(_) => unreachable!(), // This event is handled in `Tag::HtmlBlock` event using `HtmlBlockReader`
                Event::Html(html) | Event::InlineHtml(html) => {
                    self.tag("html")?;
                    self.out.write_all(br#","raw":""#)?;

                    let dst = StringContentEncoder(&mut self.out);
                    let src = InlineHtmlReader::new(html, &mut events);
                    self.sanitizer.clean(dst, src)?;

                    self.out.write_all(br#""}"#)?;
                }
                Event::SoftBreak => {
                    // Soft break consists of \n or \r\n so the length is 1 or 2
                    let range = if range.len() == 1 { range } else { (range.end - 1)..range.end };
                    // Soft break is rendered as a single white space in Markdown. Do not pass through \n or \r\n
                    self.text(" ", range)?
                }
                Event::HardBreak => {
                    self.tag("br")?;
                    self.out.write_all(b"}")?;
                }
                Event::Rule => {
                    self.tag("hr")?;
                    self.out.write_all(b"}")?;
                }
                Event::FootnoteReference(name) => {
                    self.tag("fn-ref")?;
                    let id = self.id(name);
                    write!(self.out, r#","id":{}}}"#, id)?;
                }
                Event::TaskListMarker(checked) => {
                    self.tag("checkbox")?;
                    write!(self.out, r#","checked":{}}}"#, checked)?;
                }
                Event::DisplayMath(text) => {
                    self.tag("math")?;
                    self.out.write_all(br#","inline":false,"expr":"#)?;
                    self.string(&text)?;
                    self.out.write_all(b"}")?;
                }
                Event::InlineMath(text) => {
                    self.tag("math")?;
                    self.out.write_all(br#","inline":true,"expr":"#)?;
                    self.string(&text)?;
                    self.out.write_all(b"}")?;
                }
            }
        }

        Ok(())
    }

    fn rebase_link(&mut self, dest: &str) -> Result<()> {
        if !should_rebase_url(dest) {
            return self.string(dest);
        }

        // Rebase 'foo/bar/' with '/path/to/base' as '/path/to/base/foo/bar'
        self.out.write_all(b"\"")?;
        self.string_content(self.base_dir)?;
        if !dest.starts_with('/') {
            self.out.write_all(b"/")?;
        }
        self.string_content(dest)?;
        self.out.write_all(b"\"")
    }

    fn children_begin(&mut self) -> Result<()> {
        self.is_start = true;
        self.out.write_all(br#","c":["#)
    }

    fn tag_end(&mut self) -> Result<()> {
        self.is_start = false;
        self.out.write_all(b"]}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UrlCharKind {
    Invalid,
    Term,
    NonTerm,
}

impl UrlCharKind {
    fn of(c: char) -> Self {
        // https://www.rfc-editor.org/rfc/rfc3987
        match c {
            '\u{00}'..='\u{1F}'
            | ' '
            | '|'
            | '"'
            | '<'
            | '>'
            | '`'
            | '('
            | ')'
            | '['
            | ']'
            | '\u{7F}'..='\u{9F}' => Self::Invalid,
            '?' | '!' | '.' | ',' | ':' | ';' | '*' | '&' | '\\' | '{' | '}' | '\'' => {
                Self::NonTerm
            }
            _ => Self::Term,
        }
    }
}

struct Autolinker(AhoCorasick);

impl Default for Autolinker {
    fn default() -> Self {
        Self(AhoCorasick::new(["https://", "http://"]).unwrap())
    }
}

impl Autolinker {
    fn find_autolink(&self, text: &str) -> Option<(usize, usize)> {
        for mat in self.0.find_iter(text) {
            let (start, scheme_end) = (mat.start(), mat.end());
            if let Some(c) = text[..start].chars().next_back() {
                if c.is_ascii_alphabetic() {
                    // Note: "foohttp://example.com" is not URL but "123http://example.com" contains URL
                    continue;
                }
            }

            let mut len = 0;
            for (i, c) in text[scheme_end..].char_indices() {
                match UrlCharKind::of(c) {
                    UrlCharKind::Invalid => break,
                    UrlCharKind::Term => {
                        len = i + c.len_utf8();
                    }
                    UrlCharKind::NonTerm => {}
                }
            }
            if len > 0 {
                return Some((start, scheme_end + len));
            }
        }
        None
    }
}

#[derive(Debug)]
enum EmojiToken<'input> {
    Text(&'input str),
    Emoji(&'static Emoji, usize),
}

struct EmojiTokenizer<'input> {
    text: &'input str,
    iter: Memchr<'input>,
    start: usize,
}

impl<'input> EmojiTokenizer<'input> {
    fn new(text: &'input str) -> Self {
        Self { iter: memchr_iter(b':', text.as_bytes()), text, start: 0 }
    }

    fn eat(&mut self, end: usize) -> &'input str {
        let text = &self.text[self.start..end];
        self.start = end;
        text
    }
}

impl<'input> Iterator for EmojiTokenizer<'input> {
    type Item = EmojiToken<'input>;

    // Tokenizing example:
    //   "foo :dog: bar :piyo: wow"
    //   -> ":dog: bar :piyo: wow" (text "foo ")
    //   -> " bar :piyo: wow"      (emoji "dog")
    //   -> ":piyo: wow"           (text " bar ")
    //   -> ": wow"                (text ":piyo")
    //   -> ""                     (text ": wow")
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.text.len() {
            return None;
        }

        let Some(end) = self.iter.next() else {
            return Some(EmojiToken::Text(self.eat(self.text.len()))); // Eat all of rest
        };

        if self.start == end {
            // Edge case: The initial input text starts with ':'
            return self.next();
        }

        if !self.text[self.start..].starts_with(':') {
            return Some(EmojiToken::Text(self.eat(end)));
        }

        // Note:
        //   text[start..end+1] == ":dog:"
        //   text[start+1..end] == "dog"
        //   text[start..end] == ":dog"
        let short = &self.text[self.start + 1..end];
        if let Some(emoji) = emojis::get_by_shortcode(short) {
            self.start = end + 1;
            Some(EmojiToken::Emoji(emoji, short.len() + 2))
        } else {
            Some(EmojiToken::Text(self.eat(end)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn load_data(name: &str) -> String {
        let mut path = PathBuf::from("src");
        path.push("markdown");
        path.push("testdata");
        path.push(format!("{}.md", name));
        match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => panic!("Could not find Markdown test data at {:?}: {}", path, err),
        }
    }

    macro_rules! snapshot_test {
        ($name:ident, $offset:expr, $basedir:expr) => {
            #[test]
            fn $name() {
                let source = load_data(stringify!($name));
                let target = MarkdownContent::new(source, $basedir);
                let parser = MarkdownParser::new(&target, $offset, ());
                let mut buf = Vec::new();
                let () = parser.write_to(&mut buf).unwrap();
                let buf = String::from_utf8(buf).unwrap();
                // Revert extra escape for '...' JavaScript string
                let buf = buf.replace("\\\\", "\\").replace("\\'", "'");
                // Remove the `JSON.parse` call to restore JSON value passed to the function
                let buf = buf.strip_prefix("JSON.parse('").unwrap();
                let buf = buf.strip_suffix("')").unwrap();
                // Check if the written output is in the valid JSON format
                let json: serde_json::Value = match serde_json::from_str(buf) {
                    Ok(value) => value,
                    Err(err) => {
                        panic!("Invalid JSON input with error \"{}\": {}", err, buf);
                    }
                };
                insta::assert_json_snapshot!(json);
            }
        };
        ($name:ident) => {
            snapshot_test!($name, None, None);
        };
        ($name:ident, $offset:expr) => {
            snapshot_test!($name, $offset, None);
        };
    }

    snapshot_test!(paragraph);
    snapshot_test!(blockquote);
    snapshot_test!(list);
    snapshot_test!(headings);
    snapshot_test!(codeblock);
    snapshot_test!(link);
    snapshot_test!(html);
    snapshot_test!(sanitized);
    snapshot_test!(inline_code);
    snapshot_test!(emphasis);
    snapshot_test!(image);
    snapshot_test!(autolink);
    snapshot_test!(emoji);
    snapshot_test!(table);
    snapshot_test!(math);
    snapshot_test!(strikethrough);
    snapshot_test!(tasklist);
    snapshot_test!(footnotes);
    snapshot_test!(highlight);
    snapshot_test!(not_link);
    snapshot_test!(soft_and_hard_break);
    snapshot_test!(all);
    snapshot_test!(unmatched_inline_html);
    snapshot_test!(inline_open_block_close_html);
    snapshot_test!(block_open_inline_close_html);
    snapshot_test!(inline_items_nested_in_inline_html);
    snapshot_test!(escaped_chars_in_text);
    snapshot_test!(alert);
    snapshot_test!(url_inside_link);
    snapshot_test!(inline_html_inside_blockquote);

    // Offset
    snapshot_test!(offset_block, Some(30));
    snapshot_test!(offset_begin, Some(0));
    snapshot_test!(offset_after_end, Some(10000000));
    snapshot_test!(offset_in_emphasis, Some(10));

    // Relative link resolutions
    #[cfg(target_os = "windows")]
    const BASE_DIR: &str = r"\a\b\c\d\e";
    #[cfg(not(target_os = "windows"))]
    const BASE_DIR: &str = "/a/b/c/d/e";
    snapshot_test!(relative_links, None, Some(Path::new(BASE_DIR)));

    // Note: This test cannot be done by snapshot_test! since JSON parser complains the escaped single quote.
    #[test]
    fn escaped_characters_in_text() {
        let source = load_data("escaped_chars_in_text");
        let target = MarkdownContent::new(source, None);
        let parser = MarkdownParser::new(&target, None, ());
        let mut buf = Vec::new();
        let () = parser.write_to(&mut buf).unwrap();
        let buf = String::from_utf8(buf).unwrap();
        insta::assert_snapshot!(buf);
    }

    mod visitor {
        use super::*;
        use crate::markdown::DisplayText;

        macro_rules! snapshot_test {
            ($name:ident) => {
                #[test]
                fn $name() {
                    let source = load_data(stringify!($name));
                    let content = MarkdownContent::new(source, None);
                    let parser = MarkdownParser::new(&content, None, ());
                    let mut buf = Vec::new();
                    let visitor: DisplayText = parser.write_to(&mut buf).unwrap();
                    let text = &visitor.raw_text();
                    let source = &content.source;
                    let mut mapped = vec![];
                    for map in visitor.sourcemap() {
                        let slice = &source[map.clone()];
                        assert!(
                            source.contains(slice),
                            "{:?} does not contain {:?}",
                            source,
                            text,
                        );
                        mapped.push((slice, map.clone()));
                    }
                    insta::assert_debug_snapshot!((text, mapped));
                }
            };
        }

        snapshot_test!(paragraph);
        snapshot_test!(blockquote);
        snapshot_test!(list);
        snapshot_test!(headings);
        snapshot_test!(codeblock);
        snapshot_test!(link);
        snapshot_test!(html);
        snapshot_test!(sanitized);
        snapshot_test!(inline_code);
        snapshot_test!(emphasis);
        snapshot_test!(image);
        snapshot_test!(autolink);
        snapshot_test!(emoji);
        snapshot_test!(table);
        snapshot_test!(math);
        snapshot_test!(strikethrough);
        snapshot_test!(tasklist);
        snapshot_test!(footnotes);
        snapshot_test!(highlight);
        snapshot_test!(not_link);
    }

    #[test]
    fn emoji_tokenizer() {
        #[derive(PartialEq, Eq, Debug)]
        enum Tok {
            T(&'static str),
            E(&'static str, usize),
        }
        for (input, expected) in [
            (":dog:", &[Tok::E("dog face", 5)][..]),
            (":nerd_face:", &[Tok::E("nerd face", 11)][..]),
            (":+1:", &[Tok::E("thumbs up", 4)][..]),
            (":-1:", &[Tok::E("thumbs down", 4)][..]),
            (":dog::cat:", &[Tok::E("dog face", 5), Tok::E("cat face", 5)][..]),
            (":dog: :cat:", &[Tok::E("dog face", 5), Tok::T(" "), Tok::E("cat face", 5)][..]),
            (
                " :dog: :cat: ",
                &[
                    Tok::T(" "),
                    Tok::E("dog face", 5),
                    Tok::T(" "),
                    Tok::E("cat face", 5),
                    Tok::T(" "),
                ][..],
            ),
            (
                "hello :dog: world :cat: nyan",
                &[
                    Tok::T("hello "),
                    Tok::E("dog face", 5),
                    Tok::T(" world "),
                    Tok::E("cat face", 5),
                    Tok::T(" nyan"),
                ][..],
            ),
            ("hello, world", &[Tok::T("hello, world")][..]),
            ("", &[][..]),
            ("dog:", &[Tok::T("dog"), Tok::T(":")][..]),
            (":dog", &[Tok::T(":dog")][..]),
            (":this-is-not-an-emoji:", &[Tok::T(":this-is-not-an-emoji"), Tok::T(":")][..]),
            (
                ":not-emoji:not-emoji:dog:",
                &[Tok::T(":not-emoji"), Tok::T(":not-emoji"), Tok::E("dog face", 5)][..],
            ),
            (
                ":not-emoji:not-emoji:dog:",
                &[Tok::T(":not-emoji"), Tok::T(":not-emoji"), Tok::E("dog face", 5)][..],
            ),
            ("::::", &[Tok::T(":"), Tok::T(":"), Tok::T(":"), Tok::T(":")][..]),
        ] {
            let actual = EmojiTokenizer::new(input)
                .map(|tok| match tok {
                    EmojiToken::Text(text) => Tok::T(text),
                    EmojiToken::Emoji(emoji, len) => Tok::E(emoji.name(), len),
                })
                .collect::<Vec<_>>();
            assert_eq!(expected, actual, "input={:?}", input);
        }
    }

    #[test]
    fn auto_linker() {
        for (input, url) in [
            ("http://example.com", Some("http://example.com")),
            ("https://example.com", Some("https://example.com")),
            ("http://example.com/foo", Some("http://example.com/foo")),
            ("http://example.com/foo/", Some("http://example.com/foo/")),
            ("http://example.com&foo=bar", Some("http://example.com&foo=bar")),
            ("hello http://example.com world", Some("http://example.com")),
            ("[foo](http://example.com)", Some("http://example.com")),
            ("[http://example.com]", Some("http://example.com")),
            ("Nice URL https://example.com!", Some("https://example.com")),
            ("This is URL https://example.com.", Some("https://example.com")),
            ("Is this URL https://example.com?", Some("https://example.com")),
            ("He said 'https://example.com'", Some("https://example.com")),
            ("Open https://example.com, and click button", Some("https://example.com")),
            ("https://example.com&", Some("https://example.com")),
            ("123http://aaa.com", Some("http://aaa.com")),
            ("file:///foo/bar", None),
            ("", None),
            ("hello, world", None),
            ("http:", None),
            ("http://", None),
            ("foohttp://aaa.com", None),
        ] {
            let found = Autolinker::default().find_autolink(input);
            assert_eq!(
                url.is_some(),
                found.is_some(),
                "input={input:?}, found={found:?}, expected={url:?}",
            );
            if let Some(url) = url {
                let (s, e) = found.unwrap();
                assert_eq!(url, &input[s..e]);
            }
        }
    }

    #[test]
    fn utf8_aware_byte_offset() {
        for (before, after, expected) in [
            ("あ", "い", Some(0)),
            ("ああ", "あい", Some(3)),
            ("あああ", "あいう", Some(3)),
            ("あああ", "あいあ", Some(3)),
            ("あいう", "ああうえお", Some(3)),
            ("あいう", "あいえ", Some(6)),
            ("", "あ", Some(0)),
            ("あ", "", Some(0)),
            ("あ", "あい", Some(3)),
            ("あ", "あ", None),
            ("あい", "あい", None),
            ("あいう", "あいう", None),
            ("", "", None),
        ] {
            let prev = MarkdownContent::new(before.into(), None);
            let now = MarkdownContent::new(after.into(), None);
            let offset = prev.modified_utf8_offset(&now);
            assert_eq!(offset, expected, "{before:?}, {after:?}");
        }
    }

    #[test]
    fn text_event_inside_html_block() {
        let target = MarkdownContent::new(" <p>foo</p>".to_string(), None);
        let parser = MarkdownParser::new(&target, None, ());
        let mut buf = Vec::new();
        let () = parser.write_to(&mut buf).unwrap();
        let buf = String::from_utf8(buf).unwrap();
        assert!(
            buf.contains(r#"{"t":"html","raw":" <p>foo</p>"}"#),
            "expected HTML block is not contained: {buf:?}",
        );
    }
}
