use ammonia::{Builder, UrlRelative, UrlRelativeEvaluate};
use once_cell::unsync::OnceCell;
use std::borrow::Cow;
use std::io::{Result, Write};
use std::ops::Deref;
use std::path::Path;

#[derive(Default)]
pub struct SlashPath(String);

impl<'a> From<&'a Path> for SlashPath {
    fn from(path: &'a Path) -> Self {
        #[cfg(not(target_os = "windows"))]
        let mut path = path.to_string_lossy().into_owned();
        #[cfg(target_os = "windows")]
        let mut path = path.to_string_lossy().replace("\\", "/");
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

pub struct Sanitizer<'a> {
    base_dir: &'a SlashPath,
    cleaner: OnceCell<Builder<'a>>,
}

impl<'a> Sanitizer<'a> {
    pub fn new(base_dir: &'a SlashPath) -> Self {
        Self { base_dir, cleaner: OnceCell::new() }
    }

    pub fn clean<W: Write>(&self, out: W, html: &str) -> Result<()> {
        let cleaner = self.cleaner.get_or_init(|| {
            let prefix = self.base_dir.to_string();
            let eval = Box::new(RebaseUrl { prefix });
            let mut builder = Builder::default();
            builder.add_generic_attributes(&["name", "id"]).url_relative(UrlRelative::Custom(eval));
            builder
        });
        cleaner.clean(html).write_to(out)
    }
}
