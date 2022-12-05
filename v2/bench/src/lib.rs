use std::fs;
use std::path::Path;

pub fn assets_dir() -> &'static Path {
    Path::new("assets")
}

pub fn asset(filename: &str) -> String {
    let path = assets_dir().join(filename);
    fs::read_to_string(path).expect("failed to read file in assets dir")
}
