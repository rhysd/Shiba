use rfd::FileDialog;
use std::path::{Path, PathBuf};

pub trait Dialog {
    fn new(extensions: &[&str]) -> Self;
    fn pick_file(self, dir: &Path) -> Option<PathBuf>;
}

impl Dialog for FileDialog {
    fn new(extensions: &[&str]) -> Self {
        FileDialog::new().add_filter("Markdown", extensions)
    }
    fn pick_file(self, dir: &Path) -> Option<PathBuf> {
        self.set_directory(dir).pick_file()
    }
}
