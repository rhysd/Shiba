use crate::renderer::RawMessageWriter;
use pulldown_cmark::{
    Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Options, Parser, Tag,
};
use std::collections::HashMap;
use std::fmt::{self, Write};

type Result<T> = std::result::Result<T, fmt::Error>;
type Range = std::ops::Range<usize>;

pub struct MarkdownParser<'a> {
    parser: Parser<'a, 'a>,
    offset: Option<usize>,
}

impl<'a> MarkdownParser<'a> {
    pub fn new(source: &'a str, offset: Option<usize>) -> Self {
        let mut options = Options::empty();
        options.insert(
            Options::ENABLE_STRIKETHROUGH
                | Options::ENABLE_FOOTNOTES
                | Options::ENABLE_TABLES
                | Options::ENABLE_TASKLISTS,
        );
        let parser = Parser::new_ext(source, options);
        Self { parser, offset }
    }
}

impl<'a> RawMessageWriter for MarkdownParser<'a> {
    fn write_to(self, writer: impl Write) -> Result<()> {
        let mut ser = ParseTreeSerializer::new(writer, self.offset);
        ser.out.write_str(r#"{"kind":"parse_tree","tree":"#)?;
        let mut writer = ser.push(self.parser)?;
        writer.write_char('}')
    }
}

// To know the format of JSON value, see type definitions in web/ipc.ts

enum TableState {
    Head,
    Row,
}

struct ParseTreeSerializer<'a, W: Write> {
    out: W,
    table: TableState,
    is_start: bool,
    ids: HashMap<CowStr<'a>, usize>,
    modified: Option<usize>,
}

impl<'a, W: Write> ParseTreeSerializer<'a, W> {
    fn new(w: W, modified: Option<usize>) -> Self {
        Self { out: w, table: TableState::Head, is_start: true, ids: HashMap::new(), modified }
    }

    fn push(mut self, parser: Parser<'a, 'a>) -> Result<W> {
        self.out.write_char('[')?;
        for (event, range) in parser.into_offset_iter() {
            self.event(event, range)?;
        }
        self.out.write_char(']')?;
        Ok(self.out)
    }

    fn string(&mut self, s: &str) -> Result<()> {
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

        self.out.write_char('"')?;
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

    fn text(&mut self, text: &str, range: Range) -> Result<()> {
        let Some(offset) = self.modified else {
            self.comma()?;
            return self.string(text);
        };

        let Range { start, end } = range;
        if end < offset {
            self.comma()?;
            return self.string(text);
        }

        // Handle the last modified offset with this text token
        self.modified = None;
        log::debug!("Handling last modified offset: {:?}", offset);

        if offset <= start {
            self.tag("modified")?;
            self.out.write_str("},")?;
            self.string(text)
        } else if end == offset {
            self.comma()?;
            self.string(text)?;
            self.tag("modified")?;
            self.out.write_char('}')
        } else {
            self.comma()?;
            let i = offset - start;
            self.string(&text[..i])?;
            self.tag("modified")?;
            self.out.write_str("},")?;
            self.string(&text[i..])
        }
    }

    fn event(&mut self, event: Event<'a>, range: Range) -> Result<()> {
        use Event::*;

        match event {
            Start(tag) => self.start_tag(tag),
            End(tag) => self.end_tag(tag),
            Text(text) => self.text(&text, range),
            Code(text) => {
                self.tag("code")?;
                self.children_begin()?;
                self.string(&text)?;
                self.children_end()
            }
            Html(html) => {
                self.tag("html")?;
                self.out.write_str(r#","raw":"#)?;
                self.string(&html)?;
                self.out.write_char('}')
            }
            SoftBreak => {
                self.comma()?;
                self.out.write_str(r#""\n""#)
            }
            HardBreak => {
                self.tag("br")?;
                self.out.write_char('}')
            }
            Rule => {
                self.tag("hr")?;
                self.out.write_char('}')
            }
            FootnoteReference(name) => {
                self.tag("fn-ref")?;
                let id = self.id(name);
                write!(self.out, r#","id":{}}}"#, id)
            }
            TaskListMarker(checked) => {
                self.tag("checkbox")?;
                write!(self.out, r#","checked":{}}}"#, checked)
            }
        }
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
