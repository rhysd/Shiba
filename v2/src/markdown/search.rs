use super::parser::{Range, TextTokenizer, TextVisitor, TokenKind};
use crate::config::SearchMatcher;
use aho_corasick::{
    AhoCorasick, AhoCorasickBuilder, FindIter as AhoCorasickFindIter, Match as AhoCorasickMatch,
};
use anyhow::Result;
use regex::{Match as RegexMatch, Matches as RegexMatches, Regex, RegexBuilder};

trait MatchPosition {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

impl MatchPosition for AhoCorasickMatch {
    fn start(&self) -> usize {
        self.start()
    }
    fn end(&self) -> usize {
        self.end()
    }
}

impl<'text> MatchPosition for RegexMatch<'text> {
    fn start(&self) -> usize {
        self.start()
    }
    fn end(&self) -> usize {
        self.end()
    }
}

trait Searcher: Sized {
    type Match<'text>: MatchPosition;
    type Iter<'me, 'text>: Iterator<Item = Self::Match<'text>>
    where
        Self: 'me;
    fn new(query: &str, ignore_case: bool) -> Result<Self>;
    fn find_iter<'me, 'text>(&'me self, text: &'text str) -> Self::Iter<'me, 'text>;
}

impl Searcher for AhoCorasick {
    type Match<'text> = AhoCorasickMatch;
    type Iter<'me, 'text> = AhoCorasickFindIter<'me, 'text, usize>;

    fn new(query: &str, ignore_case: bool) -> Result<Self> {
        Ok(AhoCorasickBuilder::new().ascii_case_insensitive(ignore_case).build([query]))
    }

    fn find_iter<'me, 'text>(&'me self, text: &'text str) -> Self::Iter<'me, 'text> {
        self.find_iter(text)
    }
}

impl Searcher for Regex {
    type Match<'text> = RegexMatch<'text>;
    type Iter<'me, 'text> = RegexMatches<'me, 'text>;

    fn new(query: &str, ignore_case: bool) -> Result<Self> {
        Ok(RegexBuilder::new(query).case_insensitive(ignore_case).build()?)
    }
    fn find_iter<'me, 'text>(&'me self, text: &'text str) -> Self::Iter<'me, 'text> {
        self.find_iter(text)
    }
}

#[derive(Default)]
pub struct DisplayText {
    text: String,
    srcmap: Vec<Range>,
}

impl TextVisitor for DisplayText {
    fn visit(&mut self, text: &str, range: &Range) {
        self.text.push_str(text);
        match self.srcmap.last_mut() {
            Some(last) if last.end == range.start => {
                last.end = range.end;
            }
            _ => self.srcmap.push(range.clone()),
        }
    }
}

impl DisplayText {
    fn search_with<S: Searcher>(&self, query: &str, ignore_case: bool) -> Result<SearchMatches> {
        let searcher = S::new(query, ignore_case)?;

        let Some(mut mapper) = SourceMapper::new(&self.srcmap) else {
            return Ok(SearchMatches::default());
        };
        let mut matches = vec![];
        for mat in searcher.find_iter(&self.text) {
            let Some(start) = mapper.map_inclusive(mat.start()) else {
                break;
            };
            let Some(end) = mapper.map_exclusive(mat.end()) else {
                break;
            };
            matches.push(start..end);
        }

        Ok(SearchMatches(matches))
    }

    pub fn search(&self, query: &str, matcher: SearchMatcher) -> Result<SearchMatches> {
        use SearchMatcher::*;

        let ignore_case = match matcher {
            SmartCase => !query.chars().any(|c| c.is_ascii_uppercase()),
            CaseInsensitive => true,
            CaseSensitive | CaseSensitiveRegex => false,
        };

        match matcher {
            SmartCase | CaseInsensitive | CaseSensitive => {
                self.search_with::<AhoCorasick>(query, ignore_case)
            }
            CaseSensitiveRegex => self.search_with::<Regex>(query, ignore_case),
        }
    }

    #[cfg(test)]
    pub fn raw_text(&self) -> &'_ str {
        &self.text
    }

    #[cfg(test)]
    pub fn sourcemap(&self) -> &'_ [Range] {
        &self.srcmap
    }
}

struct SourceMapper<'a> {
    head: &'a Range,
    tail: &'a [Range],
    offset: usize,
}

impl<'a> SourceMapper<'a> {
    fn new(srcmap: &'a [Range]) -> Option<Self> {
        let (head, tail) = srcmap.split_first()?;
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

#[derive(Default)]
pub struct SearchMatches(Vec<Range>);

impl SearchMatches {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn tokenizer(&self, index: Option<usize>) -> Option<MatchTokenizer<'_>> {
        let (head, tail) = self.0.split_first()?;
        Some(MatchTokenizer { head, tail, current: 0, is_start: true, index })
    }
}

pub struct MatchTokenizer<'a> {
    head: &'a Range,
    tail: &'a [Range],
    current: usize,
    is_start: bool,
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
        self.is_start = true;
        true
    }

    fn match_token(&mut self) -> TokenKind {
        match self.index {
            Some(idx) if idx == self.current => {
                if self.is_start {
                    self.is_start = false;
                    TokenKind::MatchCurrentStart
                } else {
                    TokenKind::MatchCurrent
                }
            }
            _ if self.is_start => {
                self.is_start = false;
                TokenKind::MatchOtherStart
            }
            _ => TokenKind::MatchOther,
        }
    }
}

impl<'a> TextTokenizer for MatchTokenizer<'a> {
    fn tokenize<'t>(&mut self, text: &'t str, range: &Range) -> (TokenKind, &'t str) {
        debug_assert_eq!(text.len(), range.len(), "text={text:?} range={range:?}");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_text(input: &str, maps: &[Range]) -> DisplayText {
        let mut text = DisplayText::default();
        for m in maps {
            text.visit(&input[m.clone()], m);
        }
        text
    }

    #[test]
    fn collect_text_with_source_maps() {
        //           ab  ef hijkl ;
        let input = "abcdefghijkl";
        let maps = [0..2, 4..6, 7..12];
        let text = build_text(input, &maps);
        assert_eq!(text.raw_text(), "abefhijkl");
        assert_eq!(text.sourcemap(), &maps);
    }

    #[test]
    fn merge_contiguous_mappings() {
        let input = "abcdefghijkl";
        let maps = [0..2, 2..4, 4..6, 6..7, 7..12];
        let text = build_text(input, &maps);
        assert_eq!(text.raw_text(), "abcdefghijkl");
        assert_eq!(text.sourcemap(), [0..12]);
    }

    #[test]
    fn search_case_sensitive() {
        //           ab  ef  JK  ab  Gh
        let input = "abcdefghJKababefGh";
        let text = build_text(input, &[0..2, 4..6, 8..10, 12..14, 16..18]);

        let matches = text.search("ab", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[0..2, 12..14]);
        let matches = text.search("ef", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[4..6]);
        let matches = text.search("Gh", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[16..18]);
        let matches = text.search("be", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[1..5]);
        let matches = text.search("befJ", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[1..9]);
        let matches = text.search("abefJKabGh", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[0..18]);
        let matches = text.search("cd", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[]);
        let matches = text.search("abefJKabGhab", SearchMatcher::CaseSensitive).unwrap();
        assert_eq!(&matches.0, &[]);
    }

    #[test]
    fn search_case_insensitive() {
        //           ab  ef  JK  ab  Gh
        let input = "abcdefghJKababefGh";
        let text = build_text(input, &[0..2, 4..6, 8..10, 12..14, 16..18]);

        let matches = text.search("AB", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[0..2, 12..14]);
        let matches = text.search("EF", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[4..6]);
        let matches = text.search("gh", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[16..18]);
        let matches = text.search("GH", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[16..18]);
        let matches = text.search("BE", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[1..5]);
        let matches = text.search("BEFj", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[1..9]);
        let matches = text.search("abefjkabgh", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[0..18]);
        let matches = text.search("cd", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[]);
        let matches = text.search("abefjkabghab", SearchMatcher::CaseInsensitive).unwrap();
        assert_eq!(&matches.0, &[]);
    }

    #[test]
    fn search_smart_case() {
        let input = "ab aB Ab AB";
        let text = build_text(input, &[0..2, 3..5, 6..8, 9..11]);

        let matches = text.search("ab", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[0..2, 3..5, 6..8, 9..11]);
        let matches = text.search("aB", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[3..5]);
        let matches = text.search("Ab", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[6..8]);
        let matches = text.search("AB", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[9..11]);

        let matches = text.search("ba", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[1..4, 4..7, 7..10]);
        let matches = text.search("BA", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[4..7]);
        let matches = text.search("bA", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[7..10]);
        let matches = text.search("Ba", SearchMatcher::SmartCase).unwrap();
        assert_eq!(&matches.0, &[]);
    }

    #[test]
    fn search_regex() {
        let input = "fo foo fooo";
        let text = build_text(input, &[0..2, 3..6, 7..11]);

        let matches = text.search("foo+", SearchMatcher::CaseSensitiveRegex).unwrap();
        assert_eq!(&matches.0, &[3..6, 7..11]);
        let matches = text.search("foo?", SearchMatcher::CaseSensitiveRegex).unwrap();
        assert_eq!(&matches.0, &[0..2, 3..6, 7..10]);
        let matches = text.search("o+f", SearchMatcher::CaseSensitiveRegex).unwrap();
        assert_eq!(&matches.0, &[1..4, 4..8]);
        let matches = text.search("o?f", SearchMatcher::CaseSensitiveRegex).unwrap();
        assert_eq!(&matches.0, &[0..1, 1..4, 5..8]);
        let matches = text.search("(fo+)+", SearchMatcher::CaseSensitiveRegex).unwrap();
        assert_eq!(&matches.0, &[0..11]);
    }
}
