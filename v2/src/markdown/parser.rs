use super::sanitizer::{should_rebase_url, Sanitizer, SlashPath};
use crate::renderer::RawMessageWriter;
use aho_corasick::AhoCorasick;
use emojis::Emoji;
use memchr::{memchr_iter, Memchr};
use pulldown_cmark::{
    Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, MathDisplay, Options, Parser,
    Tag,
};
use std::cmp;
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

#[derive(Default)]
pub struct MarkdownParseTarget {
    source: String,
    base_dir: SlashPath,
}

impl MarkdownParseTarget {
    pub fn new(source: String, base_dir: Option<&Path>) -> Self {
        let base_dir =
            if let Some(path) = base_dir { SlashPath::from(path) } else { SlashPath::default() };
        Self { source, base_dir }
    }

    pub fn modified_offset(&self, new: &Self) -> Option<usize> {
        let (prev_source, new_source) = (&self.source, &new.source);
        prev_source
            .as_bytes()
            .iter()
            .zip(new_source.as_bytes().iter())
            .position(|(a, b)| a != b)
            .or_else(|| {
                let (prev_len, new_len) = (prev_source.len(), new_source.len());
                (prev_len != new_len).then_some(cmp::min(prev_len, new_len))
            })
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty() && self.base_dir.is_empty()
    }
}

pub struct MarkdownParser<'a, V: TextVisitor, T: TextTokenizer> {
    parser: Parser<'a, 'a>,
    base_dir: &'a SlashPath,
    offset: Option<usize>,
    text_tokenizer: T,
    _phantom: PhantomData<V>,
}

impl<'a, V: TextVisitor, T: TextTokenizer> MarkdownParser<'a, V, T> {
    pub fn new(target: &'a MarkdownParseTarget, offset: Option<usize>, text_tokenizer: T) -> Self {
        let mut options = Options::empty();
        options.insert(
            Options::ENABLE_STRIKETHROUGH
                | Options::ENABLE_FOOTNOTES
                | Options::ENABLE_TABLES
                | Options::ENABLE_TASKLISTS
                | Options::ENABLE_MATH,
        );
        let parser = Parser::new_ext(&target.source, options);
        let base_dir = &target.base_dir;
        Self { parser, base_dir, offset, text_tokenizer, _phantom: PhantomData }
    }
}

// Note: Build raw JavaScript expression which is evaluated to the render tree encoded as JSON value.
// This expression will be evaluated via `receive(JSON.parse('{"kind":"render_tree",...}'))` by renderer.
impl<'a, V: TextVisitor, T: TextTokenizer> RawMessageWriter for MarkdownParser<'a, V, T> {
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
        BS => out.write_all(br#"\\\\"#), // Escape twice for JS and JSON (\\\\ → \\ → \)
        SQ => out.write_all(br#"\'"#), // JSON string will be put in '...' JS string. ' needs to be escaped
        XX => write!(out, r#"\\u{:04x}"#, b),
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

struct RawHtmlReader<'a, I: Iterator<Item = (Event<'a>, Range)>> {
    current: CowStr<'a>,
    index: usize,
    events: Peekable<I>,
    stack: usize,
}

impl<'a, I: Iterator<Item = (Event<'a>, Range)>> RawHtmlReader<'a, I> {
    fn new(current: CowStr<'a>, events: Peekable<I>) -> Self {
        Self { current, index: 0, events, stack: 1 }
    }

    fn read_byte(&mut self) -> Option<u8> {
        // Current event was consumed. Fetch next event otherwise return `None`.
        while self.current.len() <= self.index {
            if !matches!(self.events.peek(), Some((Event::Html(_) | Event::Text(_), _)))
                || self.stack == 0
            {
                return None;
            }
            self.current = match self.events.next().unwrap().0 {
                Event::Html(html) => {
                    if html.starts_with("</") {
                        self.stack -= 1;
                    } else {
                        self.stack += 1;
                    }
                    html
                }
                Event::Text(text) => text,
                _ => unreachable!(),
            };
            self.index = 0;
        }

        let b = self.current.as_bytes()[self.index];
        self.index += 1;
        Some(b)
    }
}

impl<'a, I: Iterator<Item = (Event<'a>, Range)>> Read for RawHtmlReader<'a, I> {
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

struct RenderTreeEncoder<'a, W: Write, V: TextVisitor, T: TextTokenizer> {
    out: W,
    base_dir: &'a SlashPath,
    table: TableState,
    is_start: bool,
    ids: HashMap<CowStr<'a>, usize>,
    modified: Option<usize>,
    text_visitor: V,
    text_tokenizer: T,
    autolinker: Autolinker,
    sanitizer: Sanitizer<'a>,
}

impl<'a, W: Write, V: TextVisitor, T: TextTokenizer> RenderTreeEncoder<'a, W, V, T> {
    fn new(w: W, base_dir: &'a SlashPath, modified: Option<usize>, text_tokenizer: T) -> Self {
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

    fn push(&mut self, parser: Parser<'a, 'a>) -> Result<()> {
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

    fn id(&mut self, name: CowStr<'a>) -> usize {
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
            self.text_tokens(&text[..i], range.start..offset)?;
            self.tag("modified")?;
            self.out.write_all(b"}")?;
            self.text_tokens(&text[i..], offset..range.end)
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

    fn events(&mut self, parser: Parser<'a, 'a>) -> Result<()> {
        use Event::*;

        let mut events = parser.into_offset_iter().peekable();
        while let Some((event, range)) = events.next() {
            match event {
                Start(tag) => {
                    let next_event = events.peek().map(|(e, _)| e);
                    self.start_tag(tag, next_event)?;
                }
                End(tag) => self.end_tag(tag)?,
                Text(text) => self.autolink_text(&text, range)?,
                Code(text) => {
                    let pad = (range.len() - text.len()) / 2;
                    let inner_range = (range.start + pad)..(range.end - pad);
                    self.tag("code")?;
                    self.children_begin()?;
                    self.text(&text, inner_range)?;
                    self.tag_end()?;
                }
                Html(html) => {
                    self.tag("html")?;
                    self.out.write_all(br#","raw":""#)?;

                    let mut dst = StringContentEncoder(&mut self.out);
                    let mut src = RawHtmlReader::new(html, events);
                    self.sanitizer.clean(&mut dst, &mut src)?;
                    events = src.events;

                    self.out.write_all(br#""}"#)?;
                }
                SoftBreak => self.text("\n", range)?,
                HardBreak => {
                    self.tag("br")?;
                    self.out.write_all(b"}")?;
                }
                Rule => {
                    self.tag("hr")?;
                    self.out.write_all(b"}")?;
                }
                FootnoteReference(name) => {
                    self.tag("fn-ref")?;
                    let id = self.id(name);
                    write!(self.out, r#","id":{}}}"#, id)?;
                }
                TaskListMarker(checked) => {
                    self.tag("checkbox")?;
                    write!(self.out, r#","checked":{}}}"#, checked)?;
                }
                Math(display, text) => {
                    self.tag("math")?;
                    write!(self.out, r#","inline":{},"expr":"#, display == MathDisplay::Inline)?;
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

    fn start_tag(&mut self, tag: Tag<'a>, next: Option<&Event>) -> Result<()> {
        use Tag::*;
        match tag {
            Paragraph => {
                self.tag("p")?;
            }
            Heading(level, id, _) => {
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

                if let Some(id) = id {
                    self.out.write_all(br#","id":"#)?;
                    self.string(id)?;
                }
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
            BlockQuote => {
                self.tag("blockquote")?;
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
            }
            List(Some(1)) => self.tag("ol")?,
            List(Some(start)) => {
                self.tag("ol")?;
                write!(self.out, r#","start":{}"#, start)?;
            }
            List(None) => self.tag("ul")?,
            Item => {
                if let Some(Event::TaskListMarker(_)) = next {
                    self.tag("task-list")?;
                } else {
                    self.tag("li")?;
                }
            }
            Emphasis => self.tag("em")?,
            Strong => self.tag("strong")?,
            Strikethrough => self.tag("del")?,
            Link(LinkType::Autolink, _, _) => return Ok(()), // Ignore autolink since it is linked by `Autolinker`
            Link(link_type, dest, title) => {
                self.tag("a")?;

                self.out.write_all(br#","href":"#)?;
                match link_type {
                    LinkType::Email => {
                        self.out.write_all(b"\"mailto:")?;
                        self.string_content(&dest)?;
                        self.out.write_all(b"\"")?;
                    }
                    _ => self.rebase_link(&dest)?,
                }

                if !title.is_empty() {
                    self.out.write_all(br#","title":"#)?;
                    self.string(&title)?;
                }
            }
            Image(_link_type, dest, title) => {
                self.tag("img")?;

                if !title.is_empty() {
                    self.out.write_all(br#","title":"#)?;
                    self.string(&title)?;
                }

                self.out.write_all(br#","src":"#)?;
                self.rebase_link(&dest)?;
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
        }

        // Tag element must have its children (maybe empty)
        self.children_begin()
    }

    fn end_tag(&mut self, tag: Tag<'a>) -> Result<()> {
        use Tag::*;
        match tag {
            Link(LinkType::Autolink, _, _) => Ok(()), // Ignore autolink since it is linked by `Autolinker`
            Paragraph
            | Heading(_, _, _)
            | TableRow
            | TableCell
            | BlockQuote
            | List(_)
            | Item
            | Emphasis
            | Strong
            | Strikethrough
            | Link(_, _, _)
            | Image(_, _, _)
            | FootnoteDefinition(_) => self.tag_end(),
            Table(_) | CodeBlock(_) => {
                self.tag_end()?;
                self.tag_end()
            }
            TableHead => {
                self.tag_end()?;
                self.tag_end()?;
                self.tag("tbody")?;
                self.children_begin()
            }
        }
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
        Self(AhoCorasick::new(["https://", "http://"]))
    }
}

impl Autolinker {
    fn find_autolink(&self, text: &str) -> Option<(usize, usize)> {
        for mat in self.0.find_iter(text) {
            let (start, scheme_end) = (mat.start(), mat.end());

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
enum EmojiToken<'a> {
    Text(&'a str),
    Emoji(&'static Emoji, usize),
}

struct EmojiTokenizer<'a> {
    text: &'a str,
    iter: Memchr<'a>,
    start: usize,
}

impl<'a> EmojiTokenizer<'a> {
    fn new(text: &'a str) -> Self {
        Self { iter: memchr_iter(b':', text.as_bytes()), text, start: 0 }
    }

    fn eat(&mut self, end: usize) -> &'a str {
        let text = &self.text[self.start..end];
        self.start = end;
        text
    }
}

impl<'a> Iterator for EmojiTokenizer<'a> {
    type Item = EmojiToken<'a>;

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
                let target = MarkdownParseTarget::new(source, $basedir);
                let parser = MarkdownParser::new(&target, $offset, ());
                let mut buf = Vec::new();
                let () = parser.write_to(&mut buf).unwrap();
                let buf = String::from_utf8(buf).unwrap();
                // Revert extra escape for '...' JavaScript string
                let buf = buf.replace("\\\\", "\\");
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

    // Offset
    snapshot_test!(offset_block, Some(30));
    snapshot_test!(offset_begin, Some(0));
    snapshot_test!(offset_after_end, Some(10000000));
    snapshot_test!(offset_in_emphasis, Some(10));

    // Relative link resolutions
    #[cfg(target_os = "windows")]
    const BASE_DIR: &str = r#"\a\b\c\d\e"#;
    #[cfg(not(target_os = "windows"))]
    const BASE_DIR: &str = "/a/b/c/d/e";
    snapshot_test!(relative_links, None, Some(Path::new(BASE_DIR)));

    mod visitor {
        use super::*;
        use crate::markdown::DisplayText;

        macro_rules! snapshot_test {
            ($name:ident) => {
                #[test]
                fn $name() {
                    let source = load_data(stringify!($name));
                    let target = MarkdownParseTarget::new(source, None);
                    let parser = MarkdownParser::new(&target, None, ());
                    let mut buf = Vec::new();
                    let visitor: DisplayText = parser.write_to(&mut buf).unwrap();
                    let text = &visitor.raw_text();
                    let source = &target.source;
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
            ("hello http://example.com world", Some("http://example.com")),
            ("[foo](http://example.com)", Some("http://example.com")),
            ("[http://example.com]", Some("http://example.com")),
            ("Nice URL https://example.com!", Some("https://example.com")),
            ("This is URL https://example.com.", Some("https://example.com")),
            ("He said 'https://example.com'", Some("https://example.com")),
            ("Open https://example.com, and click button", Some("https://example.com")),
            ("https://example.com&", Some("https://example.com")),
            ("file:///foo/bar", None),
            ("", None),
            ("hello, world", None),
            ("http:", None),
            ("http://", None),
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
}
