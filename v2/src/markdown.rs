use crate::renderer::RawMessageWriter;
use aho_corasick::AhoCorasick;
use emojis::Emoji;
use memchr::{memchr_iter, Memchr};
use pulldown_cmark::{
    Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Options, Parser, Tag,
};
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::marker::PhantomData;

type Result<T> = std::result::Result<T, fmt::Error>;
pub type Range = std::ops::Range<usize>;

pub trait ParseResult: Default {
    fn on_text(&mut self, text: &str, range: &Range);
}

impl ParseResult for () {
    fn on_text(&mut self, _text: &str, _range: &Range) {}
}

#[derive(Clone, Copy)]
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
            Self::Normal => unreachable!(),
            Self::MatchOther => "match",
            Self::MatchCurrent => "match-current",
            Self::MatchOtherStart => "match-start",
            Self::MatchCurrentStart => "match-current-start",
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

pub struct MarkdownParser<'a, R: ParseResult, T: TextTokenizer> {
    parser: Parser<'a, 'a>,
    offset: Option<usize>,
    text_tokenizer: T,
    _phantom: PhantomData<R>,
}

impl<'a, R: ParseResult, T: TextTokenizer> MarkdownParser<'a, R, T> {
    pub fn new(source: &'a str, offset: Option<usize>, text_tokenizer: T) -> Self {
        let mut options = Options::empty();
        options.insert(
            Options::ENABLE_STRIKETHROUGH
                | Options::ENABLE_FOOTNOTES
                | Options::ENABLE_TABLES
                | Options::ENABLE_TASKLISTS,
        );
        let parser = Parser::new_ext(source, options);
        Self { parser, offset, text_tokenizer, _phantom: PhantomData }
    }
}

impl<'a, R: ParseResult, T: TextTokenizer> RawMessageWriter for MarkdownParser<'a, R, T> {
    type Output = R;

    fn write_to(self, writer: impl Write) -> Result<Self::Output> {
        let mut ser = RenderTreeSerializer::new(writer, self.offset, self.text_tokenizer);
        ser.out.write_str(r#"{"kind":"render_tree","tree":"#)?;
        ser.push(self.parser)?;
        ser.out.write_char('}')?;
        Ok(ser.parsed)
    }
}

// To know the format of JSON value, see type definitions in web/ipc.ts

enum TableState {
    Head,
    Row,
}

struct RenderTreeSerializer<'a, W: Write, R: ParseResult, T: TextTokenizer> {
    out: W,
    table: TableState,
    is_start: bool,
    ids: HashMap<CowStr<'a>, usize>,
    modified: Option<usize>,
    parsed: R,
    text_tokenizer: T,
    autolinker: Autolinker,
}

impl<'a, W: Write, R: ParseResult, T: TextTokenizer> RenderTreeSerializer<'a, W, R, T> {
    fn new(w: W, modified: Option<usize>, text_tokenizer: T) -> Self {
        Self {
            out: w,
            table: TableState::Head,
            is_start: true,
            ids: HashMap::new(),
            modified,
            parsed: R::default(),
            text_tokenizer,
            autolinker: Autolinker::default(),
        }
    }

    fn push(&mut self, parser: Parser<'a, 'a>) -> Result<()> {
        self.out.write_char('[')?;
        self.events(parser)?;
        // Modified offset was not consumed by any text, it would mean that some non-text parts after any text were
        // modified. As a fallback, set 'modified' marker after the last text.
        if self.modified.is_some() {
            self.tag("modified")?;
            self.out.write_char('}')?;
        }
        self.out.write_char(']')
    }

    fn string_content(&mut self, s: &str) -> Result<()> {
        #[rustfmt::skip]
        const ESCAPE_TABLE: [u8; 128] = [
            0, 0, 0,    0, 0, 0, 0, 0, b'b', b't', b'n', 0, b'f',  b'r', 0, 0, // 16
            0, 0, 0,    0, 0, 0, 0, 0, 0,    0,    0,    0, 0,     0,    0, 0, // 32
            1, 1, b'"', 1, 1, 1, 1, 1, 1,    1,    1,    1, 1,     1,    1, 1, // 48
            1, 1, 1,    1, 1, 1, 1, 1, 1,    1,    1,    1, 1,     1,    1, 1, // 64
            1, 1, 1,    1, 1, 1, 1, 1, 1,    1,    1,    1, 1,     1,    1, 1, // 80
            1, 1, 1,    1, 1, 1, 1, 1, 1,    1,    1,    1, b'\\', 1,    1, 1, // 96
            1, 1, 1,    1, 1, 1, 1, 1, 1,    1,    1,    1, 1,     1,    1, 1, // 112
            1, 1, 1,    1, 1, 1, 1, 1, 1,    1,    1,    1, 1,     1,    1, 0, // 128
        ];

        for c in s.chars() {
            if c < (128 as char) {
                match ESCAPE_TABLE[c as usize] {
                    1 => self.out.write_char(c)?,
                    0 => write!(self.out, "\\u{:04x}", c as u32)?,
                    b => {
                        self.out.write_char('\\')?;
                        self.out.write_char(b as char)?;
                    }
                }
            } else {
                self.out.write_char(c)?;
            }
        }

        Ok(())
    }

    fn string(&mut self, s: &str) -> Result<()> {
        self.out.write_char('"')?;
        self.string_content(s)?;
        self.out.write_char('"')
    }

    fn alignment(&mut self, a: Alignment) -> Result<()> {
        self.out.write_str(match a {
            Alignment::None => "null",
            Alignment::Left => r#""left""#,
            Alignment::Center => r#""center""#,
            Alignment::Right => r#""right""#,
        })
    }

    fn id(&mut self, name: CowStr<'a>) -> usize {
        let new = self.ids.len() + 1;
        *self.ids.entry(name).or_insert(new)
    }

    fn comma(&mut self) -> Result<()> {
        if !self.is_start {
            self.out.write_char(',')?;
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
                    self.children_end()?;
                }
            }
            input = &input[text.len()..];
            range.start += text.len();
        }

        Ok(())
    }

    fn text(&mut self, text: &str, range: Range) -> Result<()> {
        self.parsed.on_text(text, &range);

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
            self.out.write_char('}')?;
            self.text_tokens(text, range)
        } else if end == offset {
            self.text_tokens(text, range)?;
            self.tag("modified")?;
            self.out.write_char('}')
        } else {
            let i = offset - start;
            self.text_tokens(&text[..i], range.start..offset)?;
            self.tag("modified")?;
            self.out.write_char('}')?;
            self.text_tokens(&text[i..], offset..range.end)
        }
    }

    fn emoji_text(&mut self, text: &str, range: Range) -> Result<()> {
        let Range { mut start, end } = range;
        for token in EmojiTokenizer::new(text) {
            match token {
                EmojiToken::Text(text) => {
                    if !text.is_empty() {
                        self.text(text, start..text.len())?;
                        start += text.len();
                    }
                }
                EmojiToken::Emoji(emoji, len) => {
                    self.tag("emoji")?;
                    self.out.write_str(r#","name":"#)?;
                    self.string(emoji.name())?;
                    self.children_begin()?;
                    self.string(emoji.as_str())?;
                    self.children_end()?;
                    start += len;
                }
            }
        }
        debug_assert_eq!(start, end);
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
            self.out.write_str(r#","auto":true,"href":"#)?;
            self.string(url)?;
            self.children_begin()?;
            self.text(url, start + s..start + e)?;
            self.children_end()?;

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
                Start(tag) => self.start_tag(tag)?,
                End(tag) => self.end_tag(tag)?,
                Text(text) => self.autolink_text(&text, range)?,
                Code(text) => {
                    let pad = (range.len() - text.len()) / 2;
                    let inner_range = (range.start + pad)..(range.end - pad);
                    self.tag("code")?;
                    self.children_begin()?;
                    self.text(&text, inner_range)?;
                    self.children_end()?;
                }
                Html(html) => {
                    self.tag("html")?;
                    self.out.write_str(r#","raw":""#)?;
                    self.string_content(&html)?;

                    // Collect all HTML events into one element object
                    while let Some((Html(html), _)) = events.peek() {
                        self.string_content(html)?;
                        events.next();
                    }

                    self.out.write_str(r#""}"#)?;
                }
                SoftBreak => self.text("\n", range)?,
                HardBreak => {
                    self.tag("br")?;
                    self.out.write_char('}')?;
                }
                Rule => {
                    self.tag("hr")?;
                    self.out.write_char('}')?;
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
            }
        }

        Ok(())
    }

    fn children_begin(&mut self) -> Result<()> {
        self.is_start = true;
        self.out.write_str(r#","c":["#)
    }

    fn children_end(&mut self) -> Result<()> {
        self.is_start = false;
        self.out.write_str("]}")
    }

    fn start_tag(&mut self, tag: Tag<'a>) -> Result<()> {
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
                    self.out.write_str(r#","id":"#)?;
                    self.string(id)?;
                }
            }
            Table(alignments) => {
                self.tag("table")?;

                self.out.write_str(r#","align":["#)?;
                let mut alignments = alignments.into_iter();
                if let Some(a) = alignments.next() {
                    self.alignment(a)?;
                }
                for a in alignments {
                    self.out.write_char(',')?;
                    self.alignment(a)?;
                }
                self.out.write_char(']')?;
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
                            self.out.write_str(r#","lang":"#)?;
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
            Item => self.tag("li")?,
            Emphasis => self.tag("em")?,
            Strong => self.tag("strong")?,
            Strikethrough => self.tag("del")?,
            Link(link_type, dest, title) => {
                self.tag("a")?;

                self.out.write_str(r#","href":"#)?;
                match link_type {
                    LinkType::Email => {
                        let mut href = "mailto:".to_string();
                        href.push_str(&dest);
                        self.string(&href)?;
                    }
                    _ => self.string(&dest)?,
                }

                if !title.is_empty() {
                    self.out.write_str(r#","title":"#)?;
                    self.string(&title)?;
                }
            }
            Image(_link_type, dest, title) => {
                self.tag("img")?;

                if !title.is_empty() {
                    self.out.write_str(r#","title":"#)?;
                    self.string(&title)?;
                }

                self.out.write_str(r#","src":"#)?;
                self.string(&dest)?;
            }
            FootnoteDefinition(name) => {
                self.tag("fn-def")?;

                if !name.is_empty() {
                    self.out.write_str(r#","name":"#)?;
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
            | FootnoteDefinition(_) => self.children_end(),
            Table(_) | CodeBlock(_) => {
                self.children_end()?;
                self.children_end()
            }
            TableHead => {
                self.children_end()?;
                self.children_end()?;
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
        let mat = self.0.find(text)?;
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
        if len == 0 {
            return None;
        }
        Some((start, scheme_end + len))
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
            return Some(EmojiToken::Text(""));
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
