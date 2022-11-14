use crate::config::SearchMatcher;
use crate::markdown::{ParseResult, Range, TextTokenizer, TokenKind};
use aho_corasick::AhoCorasickBuilder;

#[derive(Default)]
pub struct Text {
    text: String,
    maps: Vec<Range>,
}

impl ParseResult for Text {
    fn on_text(&mut self, text: &str, range: &Range) {
        self.text.push_str(text);
        self.maps.push(range.clone());
    }
}

struct SourceMapper<'a> {
    head: &'a Range,
    tail: &'a [Range],
    offset: usize,
}

impl<'a> SourceMapper<'a> {
    fn new(maps: &'a [Range]) -> Option<Self> {
        let (head, tail) = maps.split_first()?;
        Some(Self { head, tail, offset: 0 })
    }

    fn next(&mut self) -> bool {
        let Some((h, t)) = self.tail.split_first() else {
            return false;
        };
        self.offset += self.head.len();
        self.head = h;
        self.tail = t;
        true
    }

    fn map_inclusive(&mut self, index: usize) -> Option<usize> {
        while self.offset + self.head.len() <= index {
            if !self.next() {
                return None;
            }
        }
        Some(self.head.start + (index - self.offset))
    }

    fn map_exclusive(&mut self, index: usize) -> Option<usize> {
        while self.offset + self.head.len() < index {
            if !self.next() {
                return None;
            }
        }
        Some(self.head.start + (index - self.offset))
    }
}

impl Text {
    pub fn search(&self, query: &str, matcher: SearchMatcher) -> SearchMatches {
        let ignore_case = match matcher {
            SearchMatcher::SmartCase => !query.chars().any(|c| c.is_ascii_uppercase()),
            SearchMatcher::CaseInsensitive => true,
            SearchMatcher::CaseSensitive => false,
            SearchMatcher::CaseSensitiveRegex => {
                log::error!("CaseSensitiveRegex matcher is not supported yet");
                false
            }
        };
        let ac = AhoCorasickBuilder::new().ascii_case_insensitive(ignore_case).build([query]);

        let Some(mut mapper) = SourceMapper::new(&self.maps) else {
            return SearchMatches::default();
        };
        let mut matches = vec![];
        for mat in ac.find_iter(&self.text) {
            let Some(start) = mapper.map_inclusive(mat.start()) else {
                break;
            };
            let Some(end) = mapper.map_exclusive(mat.end()) else {
                break;
            };
            matches.push(start..end);
        }

        SearchMatches(matches)
    }
}

#[derive(Default)]
pub struct SearchMatches(Vec<Range>);

impl SearchMatches {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn tokenizer(&self, index: Option<usize>) -> Option<MatchTokenizer<'_>> {
        let (head, tail) = self.0.split_first()?;
        Some(MatchTokenizer { head, tail, current: 0, index })
    }
}

pub struct MatchTokenizer<'a> {
    head: &'a Range,
    tail: &'a [Range],
    current: usize,
    index: Option<usize>,
}

impl<'a> MatchTokenizer<'a> {
    fn next(&mut self) -> bool {
        let Some((h, t)) = self.tail.split_first() else {
            return false;
        };
        self.head = h;
        self.tail = t;
        self.current += 1;
        true
    }

    fn match_token(&self) -> TokenKind {
        match self.index {
            Some(idx) if idx == self.current => TokenKind::MatchCurrent,
            _ => TokenKind::MatchOther,
        }
    }
}

impl<'a> TextTokenizer for MatchTokenizer<'a> {
    fn tokenize<'t>(&mut self, text: &'t str, range: &Range) -> (TokenKind, &'t str) {
        debug_assert_eq!(text.len(), range.len());
        let Range { start, end } = *range;

        while self.head.end <= start {
            if !self.next() {
                return (TokenKind::Normal, text);
            }
        }

        if self.head.start <= start {
            let token = self.match_token();
            if self.head.end < end {
                (token, &text[..self.head.end - start])
            } else {
                (token, text)
            }
        } else if self.head.start < end {
            (TokenKind::Normal, &text[..self.head.start - start])
        } else {
            (TokenKind::Normal, text)
        }
    }
}
