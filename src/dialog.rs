use crate::config::FileExtensions;
use anyhow::Error;
use rfd::{FileDialog, MessageDialog, MessageLevel};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[non_exhaustive]
pub enum DialogMessageLevel {
    Error,
}

pub trait Dialog {
    fn pick_files(dir: &Path, extensions: &FileExtensions) -> Vec<PathBuf>;

    fn pick_dirs(dir: &Path) -> Vec<PathBuf>;

    fn message(level: DialogMessageLevel, title: impl Into<String>, body: impl Into<String>);

    fn alert(error: &Error) {
        let mut errs = error.chain();
        let title = format!("Error: {}", errs.next().unwrap());
        let mut message = title.clone();
        for err in errs {
            write!(message, "\n  Caused by: {}", err).unwrap();
        }
        log::error!("{}", message);
        Self::message(DialogMessageLevel::Error, title, message);
    }
}

// TODO: Consider to set parent window of dialog. rfd provides `set_parent` methods to dialogs.

pub struct SystemDialog;

impl Dialog for SystemDialog {
    fn pick_files(dir: &Path, extensions: &FileExtensions) -> Vec<PathBuf> {
        FileDialog::new()
            .set_title("Open file to preview")
            .add_filter("Markdown", extensions.as_slice())
            .set_directory(dir)
            .set_can_create_directories(true)
            .pick_files()
            .unwrap_or_default()
    }

    fn pick_dirs(cwd: &Path) -> Vec<PathBuf> {
        FileDialog::new()
            .set_title("Choose directory to watch")
            .set_directory(cwd)
            .set_can_create_directories(true)
            .pick_folders()
            .unwrap_or_default()
    }

    fn message(level: DialogMessageLevel, title: impl Into<String>, body: impl Into<String>) {
        let level = match level {
            DialogMessageLevel::Error => MessageLevel::Error,
        };
        MessageDialog::new()
            .set_level(level)
            .set_title(title.into())
            .set_description(body.into())
            .show();
    }
}
