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

fn embed(slash_path: &str, src: &str) -> String {
    println!("cargo:rerun-if-changed={}", slash_path);
    let path = from_slash(slash_path);
    let placeholder = format!("/* replace with {} */", slash_path);
    src.replace(&placeholder, &fs::read_to_string(path).unwrap())
}

fn main() {
    println!("cargo:rerun-if-changed=dist/template.html");
    println!("cargo:rerun-if-changed=dist/bundle.js");

    let html = fs::read_to_string(from_slash("dist/template.html")).unwrap();
    let html = embed("node_modules/github-markdown-css/github-markdown.css", &html);
    let html = embed("node_modules/highlight.js/styles/github.css", &html);
    let html = embed("dist/bundle.js", &html);

    let mut out = fs::File::create(from_slash("src/bundle.html")).unwrap();
    out.write_all(html.as_bytes()).unwrap();
}
