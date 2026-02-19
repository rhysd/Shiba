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
    dir: PathBuf,
}

impl SystemDialog {
    fn file_dialog(&self, handles: &WindowHandles<'_>) -> FileDialog {
        log::debug!("Opening file dialog at directory {:?}", self.dir);
        FileDialog::new()
            .set_directory(&self.dir)
            .set_can_create_directories(true)
            .set_parent(handles)
    }
}

impl Dialog for SystemDialog {
    fn new(config: &Config) -> Result<Self> {
        let extensions = config.watch().file_extensions.clone();
        let dir = config.dialog().default_dir()?;
        Ok(Self { extensions, dir })
    }

    fn pick_files(&mut self, handles: &WindowHandles<'_>) -> Vec<PathBuf> {
        let files = self
            .file_dialog(handles)
            .set_title("Open files to preview")
            .add_filter("Markdown", self.extensions.as_slice())
            .pick_files()
            .unwrap_or_default();
        if let Some(file) = files.first()
            && let Some(dir) = file.parent()
        {
            self.dir = dir.to_path_buf();
        }
        files
    }

    fn pick_dirs(&mut self, handles: &WindowHandles<'_>) -> Vec<PathBuf> {
        let dirs = self
            .file_dialog(handles)
            .set_title("Choose directories to watch")
            .pick_folders()
            .unwrap_or_default();
        if let Some(dir) = dirs.first() {
            self.dir = dir.clone();
        }
        dirs
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
