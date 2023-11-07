use crate::config::FileExtensions;
use rfd::{FileDialog, MessageDialog, MessageLevel};
use std::path::{Path, PathBuf};

pub trait Dialog: Default {
    fn pick_file(&self, dir: &Path, extensions: &FileExtensions) -> Option<PathBuf>;
    fn pick_dir(&self, dir: &Path) -> Option<PathBuf>;
    fn alert(&self, title: impl Into<String>, message: impl Into<String>);
}

#[derive(Default)]
pub struct SystemDialog;

impl Dialog for SystemDialog {
    fn pick_file(&self, dir: &Path, extensions: &FileExtensions) -> Option<PathBuf> {
        // `FileDialog::add_filter` requires `&[&str]` but we have `Vec<String>` in config
        let extensions: Vec<&str> = extensions.as_slice().iter().map(String::as_str).collect();
        FileDialog::new().add_filter("Markdown", &extensions).set_directory(dir).pick_file()
    }

    fn pick_dir(&self, dir: &Path) -> Option<PathBuf> {
        FileDialog::new().set_directory(dir).pick_folder()
    }

    fn alert(&self, title: impl Into<String>, message: impl Into<String>) {
        MessageDialog::new()
            .set_level(MessageLevel::Error)
            .set_title(title.into())
            .set_description(message.into())
            .show();
    }
}
