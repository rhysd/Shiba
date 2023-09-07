use crate::config::FileExtensions;
use rfd::FileDialog;
use std::path::{Path, PathBuf};

pub trait Dialog: Default {
    fn pick_file(&mut self, dir: &Path, extensions: &FileExtensions) -> Option<PathBuf>;
    fn pick_dir(&mut self, dir: &Path) -> Option<PathBuf>;
}

#[derive(Default)]
pub struct SystemDialog;

impl Dialog for SystemDialog {
    fn pick_file(&mut self, dir: &Path, extensions: &FileExtensions) -> Option<PathBuf> {
        // `FileDialog::add_filter` requires `&[&str]` but we have `Vec<String>` in config
        let extensions: Vec<&str> = extensions.as_slice().iter().map(String::as_str).collect();
        FileDialog::new().add_filter("Markdown", &extensions).set_directory(dir).pick_file()
    }

    fn pick_dir(&mut self, dir: &Path) -> Option<PathBuf> {
        FileDialog::new().set_directory(dir).pick_folder()
    }
}
