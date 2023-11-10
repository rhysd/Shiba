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
    fn pick_file(dir: &Path, extensions: &FileExtensions) -> Option<PathBuf>;

    fn pick_dir(dir: &Path) -> Option<PathBuf>;

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
    fn pick_file(dir: &Path, extensions: &FileExtensions) -> Option<PathBuf> {
        FileDialog::new()
            .add_filter("Markdown", extensions.as_slice())
            .set_directory(dir)
            .pick_file()
    }

    fn pick_dir(dir: &Path) -> Option<PathBuf> {
        FileDialog::new().set_directory(dir).pick_folder()
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
