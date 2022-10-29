use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
fn from_slash(s: &str) -> PathBuf {
    PathBuf::from(s.replace('/', "\\"))
}
#[cfg(not(target_os = "windows"))]
fn from_slash(s: &str) -> PathBuf {
    PathBuf::from(s)
}

struct Bundler {
    is_release: bool,
    html: String,
}

impl Bundler {
    fn new(template: &str) -> Self {
        println!("cargo:rerun-if-changed={}", template);
        let is_release = env::var("PROFILE").unwrap() == "release";
        let html = fs::read_to_string(from_slash(template)).unwrap();
        Self { is_release, html }
    }

    fn embed(mut self, debug: &str, release: &str, replaced: &str) -> Self {
        println!("cargo:rerun-if-changed={}", debug);
        println!("cargo:rerun-if-changed={}", release);

        let path = from_slash(if self.is_release { release } else { debug });
        let placeholder = format!("/* replace with {} */", replaced);
        self.html = self.html.replace(&placeholder, &fs::read_to_string(path).unwrap());
        self
    }

    fn bundle(self, debug: &str, release: &str) {
        let path = if self.is_release { release } else { debug };
        let mut out = fs::File::create(from_slash(path)).unwrap();
        out.write_all(self.html.as_bytes()).unwrap();
    }
}

fn main() {
    Bundler::new("dist/template.html")
        .embed(
            "node_modules/github-markdown-css/github-markdown.css",
            "dist/github-markdown.min.css",
            "github-markdown.css",
        )
        .embed(
            "node_modules/highlight.js/styles/github.css",
            "dist/hljs-github.min.css",
            "hljs-github.css",
        )
        .embed("dist/bundle.js", "dist/bundle.min.js", "bundle.js")
        .bundle("src/bundle.html", "src/bundle.min.html");
}
