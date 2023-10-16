use ammonia::{Builder, UrlRelative, UrlRelativeEvaluate};
use once_cell::unsync::OnceCell;
use std::borrow::Cow;
use std::io::{Read, Result, Write};
use std::ops::Deref;
use std::path::Path;

#[derive(Default)]
pub struct SlashPath(String);

#[cfg(target_os = "windows")]
fn to_slash_path(path: &Path) -> String {
    use std::path::Component::*;

    // Remove UNC path prefix and drive letter since WebView2 does not allow loading local resources directly
    // by the absolute path.
    //   e.g. '\\?\C:\Users\rhysd\foo.md' -> '/Users/rhysd/foo.md'
    let mut slash = String::new();
    for component in path.components() {
        match component {
            RootDir => slash.push('/'),
            ParentDir => slash.push_str("../"),
            Normal(s) => {
                slash.push_str(&s.to_string_lossy());
                slash.push('/');
            }
            _ => {}
        }
    }

    slash
}

impl<'a> From<&'a Path> for SlashPath {
    fn from(path: &'a Path) -> Self {
        #[cfg(not(target_os = "windows"))]
        let mut path = path.to_string_lossy().into_owned();
        #[cfg(target_os = "windows")]
        let mut path = to_slash_path(path);
        if path.ends_with('/') {
            path.pop(); // Ensure the path does not end with /
        }
        Self(path)
    }
}

impl Deref for SlashPath {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn should_rebase_url(url: &str) -> bool {
    !url.starts_with('#')
        && !url.starts_with("https://")
        && !url.starts_with("http://")
        && !url.starts_with("//")
}

// TODO: Change this to `struct RebaseUrl<'a>{ prefix: &'a str }` when the following PR is released.
// https://github.com/rust-ammonia/ammonia/pull/176
struct RebaseUrl {
    prefix: String,
}

impl UrlRelativeEvaluate for RebaseUrl {
    fn evaluate<'u>(&self, url: &'u str) -> Option<Cow<'u, str>> {
        if !should_rebase_url(url) {
            return Some(Cow::Borrowed(url));
        }

        let mut s = self.prefix.clone();
        if !url.starts_with('/') {
            s.push('/');
        }
        s.push_str(url);
        Some(Cow::Owned(s))
    }
}

const ALLOWED_ATTRIBUTES: &[&str] = &["name", "id"];

pub struct Sanitizer<'a> {
    base_dir: &'a SlashPath,
    cleaner: OnceCell<Builder<'a>>,
}

impl<'a> Sanitizer<'a> {
    pub fn new(base_dir: &'a SlashPath) -> Self {
        Self { base_dir, cleaner: OnceCell::new() }
    }

    pub fn clean<W: Write, R: Read>(&self, out: W, reader: R) -> Result<()> {
        let cleaner = self.cleaner.get_or_init(|| {
            let prefix = self.base_dir.to_string();
            let eval = Box::new(RebaseUrl { prefix });
            let mut builder = Builder::default();
            builder
                .add_generic_attributes(ALLOWED_ATTRIBUTES)
                .url_relative(UrlRelative::Custom(eval));
            builder
        });
        cleaner.clean_from_reader(reader)?.write_to(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn create_slash_path_non_windows() {
        for (input, want) in [("", ""), ("/a/b/c", "/a/b/c"), ("/a/b/c/", "/a/b/c"), ("/", "")] {
            let path = SlashPath::from(Path::new(input));
            assert_eq!(path.deref(), want);
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn create_slash_path_windows() {
        for (input, want) in [
            ("", ""),
            (r"\\?\C:\a\b\c", "/a/b/c"),
            (r"\\?\C:\a\b\c\", "/a/b/c"),
            (r"C:\a\b\c", "/a/b/c"),
            (r"C:\a\b\c\", "/a/b/c"),
            (r"\a\b\c", "/a/b/c"),
            (r"\a\b\c\", "/a/b/c"),
            (r"\\?\C:\", ""),
            (r"C:\", ""),
            (r"\", ""),
            (r"\\?\C:\a\.\b\.\c", "/a/b/c"),
            (r"\\?\C:\a\b\d\..\c", "/a/b/d/../c"),
        ] {
            let path = SlashPath::from(Path::new(input));
            assert_eq!(path.deref(), want);
        }
    }

    #[test]
    fn rebase_relative_url() {
        let rebase = RebaseUrl { prefix: "/foo/bar".to_string() };

        for (url, want) in [
            ("", "/foo/bar/"),
            ("aaa", "/foo/bar/aaa"),
            ("aaa/", "/foo/bar/aaa/"),
            ("/aaa", "/foo/bar/aaa"),
            ("..", "/foo/bar/.."),
            ("./aaa", "/foo/bar/./aaa"),
            ("http://example.com", "http://example.com"),
            ("https://example.com", "https://example.com"),
            ("//example.com", "//example.com"),
            ("#hash", "#hash"),
        ] {
            let have = rebase.evaluate(url).unwrap();
            assert_eq!(&have, want);
        }
    }

    #[test]
    fn sanitize_raw_html() {
        #[cfg(target_os = "windows")]
        const BASE_DIR: &str = r"\a\b\c\d\e";
        #[cfg(not(target_os = "windows"))]
        const BASE_DIR: &str = "/a/b/c/d/e";
        let base_dir = SlashPath::from(Path::new(BASE_DIR));
        let sanitizer = Sanitizer::new(&base_dir);

        for (input, want) in [
            ("", ""),
            ("foo", "foo"),
            ("<div>foo</div>", "<div>foo</div>"),
            (
                "<div><span>aaa</span><span>bbb</span></div>",
                "<div><span>aaa</span><span>bbb</span></div>",
            ),
            (
                "<img src=\"https://example.com\" alt=\"hello\">",
                "<img src=\"https://example.com\" alt=\"hello\">",
            ),
            ("<script>alert()</script>", ""),
            ("<span><script>alert()</script>foo</span>", "<span>foo</span>"),
            ("<div onclick=\"alert\">hello</div>", "<div>hello</div>"),
            (
                "<a href=\"javascript:alert()\">hello</a>",
                "<a rel=\"noopener noreferrer\">hello</a>",
            ),
            (
                "<a href=\"data:aGVsbG8sd29ybGQK\">hello</a>",
                "<a rel=\"noopener noreferrer\">hello</a>",
            ),
            (
                "<a href=\"https://example.com\">foo</a>",
                "<a href=\"https://example.com\" rel=\"noopener noreferrer\">foo</a>",
            ),
            (
                "<a href=\"http://example.com\">foo</a>",
                "<a href=\"http://example.com\" rel=\"noopener noreferrer\">foo</a>",
            ),
            (
                "<a href=\"//example.com\">foo</a>",
                "<a href=\"//example.com\" rel=\"noopener noreferrer\">foo</a>",
            ),
            (
                "<a href=\"foo/bar\">foo</a>",
                "<a href=\"/a/b/c/d/e/foo/bar\" rel=\"noopener noreferrer\">foo</a>",
            ),
            (
                "<a href=\"../foo/bar\">foo</a>",
                "<a href=\"/a/b/c/d/e/../foo/bar\" rel=\"noopener noreferrer\">foo</a>",
            ),
            (
                "<a href=\"#hash-link\">foo</a>",
                "<a href=\"#hash-link\" rel=\"noopener noreferrer\">foo</a>",
            ),
            (
                "<a name=\"hash-link\"></a>",
                "<a name=\"hash-link\" rel=\"noopener noreferrer\"></a>",
            ),
            (
                "<a id=\"hash-link-id\"></a>",
                "<a id=\"hash-link-id\" rel=\"noopener noreferrer\"></a>",
            ),
            (
                "<a href=\"hello&world\">hello</a>",
                "<a href=\"/a/b/c/d/e/hello&amp;world\" rel=\"noopener noreferrer\">hello</a>",
            ),
        ] {
            let mut have = Vec::new();
            sanitizer.clean(&mut have, input.as_bytes()).unwrap();
            let have = String::from_utf8(have).unwrap();
            assert_eq!(&have, want);
        }
    }
}
