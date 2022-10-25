use std::path::PathBuf;

// TODO: Move CLI parser here

#[derive(Debug, Default)]
pub struct Options {
    pub debug: bool,
    pub init_file: Option<PathBuf>,
}
