use crate::config::{Config, FileExtensions};
use crate::renderer::WindowHandles;
use anyhow::{Error, Result};
use rfd::{FileDialog, MessageDialog, MessageLevel};
use std::fmt::Write as _;
use std::path::PathBuf;

#[non_exhaustive]
pub enum DialogMessageLevel {
    Error,
}

pub trait Dialog: Default {
    fn new(config: &Config) -> Result<Self>;

    fn pick_files(&mut self, handles: &WindowHandles<'_>) -> Vec<PathBuf>;

    fn pick_dirs(&mut self, handles: &WindowHandles<'_>) -> Vec<PathBuf>;

    fn message(
        &self,
        level: DialogMessageLevel,
        title: impl Into<String>,
        body: impl Into<String>,
        handles: &WindowHandles<'_>,
    );

    fn alert(&self, error: &Error, handles: &WindowHandles<'_>) {
        let mut errs = error.chain();
        let title = format!("Error: {}", errs.next().unwrap());
        let mut message = title.clone();
        for err in errs {
            write!(message, "\n  Caused by: {}", err).unwrap();
        }
        log::error!("{}", message);
        self.message(DialogMessageLevel::Error, title, message, handles);
    }
}

#[derive(Default)]
pub struct SystemDialog {
    extensions: FileExtensions,
    start_dir: Option<PathBuf>,
}

impl SystemDialog {
    fn file_dialog(&mut self, handles: &WindowHandles<'_>) -> FileDialog {
        let mut dialog = FileDialog::new().set_can_create_directories(true).set_parent(handles);
        if let Some(dir) = self.start_dir.take() {
            log::debug!("Opening file dialog at start directory {:?}", dir);
            dialog = dialog.set_directory(dir);
        }
        dialog
    }
}

impl Dialog for SystemDialog {
    fn new(config: &Config) -> Result<Self> {
        let extensions = config.watch().file_extensions.clone();
        let start_dir = config.dialog().default_dir();
        Ok(Self { extensions, start_dir })
    }

    fn pick_files(&mut self, handles: &WindowHandles<'_>) -> Vec<PathBuf> {
        self.file_dialog(handles)
            .set_title("Open files to preview")
            .add_filter("Markdown", self.extensions.as_slice())
            .pick_files()
            .unwrap_or_default()
    }

    fn pick_dirs(&mut self, handles: &WindowHandles<'_>) -> Vec<PathBuf> {
        self.file_dialog(handles)
            .set_title("Choose directories to watch")
            .pick_folders()
            .unwrap_or_default()
    }

    fn message(
        &self,
        level: DialogMessageLevel,
        title: impl Into<String>,
        body: impl Into<String>,
        handles: &WindowHandles<'_>,
    ) {
        let level = match level {
            DialogMessageLevel::Error => MessageLevel::Error,
        };
        MessageDialog::new()
            .set_level(level)
            .set_title(title.into())
            .set_description(body.into())
            .set_parent(handles)
            .show();
    }
}
