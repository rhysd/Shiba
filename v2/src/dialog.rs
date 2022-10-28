use rfd::FileDialog;
use std::path::{Path, PathBuf};

pub trait Dialog {
    fn pick_file(dir: &Path, extensions: &[&str]) -> Option<PathBuf>;
    fn pick_dir(dir: &Path) -> Option<PathBuf>;
}

impl Dialog for FileDialog {
    fn pick_file(dir: &Path, extensions: &[&str]) -> Option<PathBuf> {
        FileDialog::new().add_filter("Markdown", extensions).set_directory(dir).pick_file()
    }

    fn pick_dir(dir: &Path) -> Option<PathBuf> {
        FileDialog::new().set_directory(dir).pick_folder()
    }
}
