use crate::config::{Config, FileExtensions};
use crate::renderer::Renderer;
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

    fn pick_files<R: Renderer>(&self, renderer: &R) -> Vec<PathBuf>;

    fn pick_dirs<R: Renderer>(&self, renderer: &R) -> Vec<PathBuf>;

    fn message(&self, level: DialogMessageLevel, title: impl Into<String>, body: impl Into<String>);

    fn alert(&self, error: &Error) {
        let mut errs = error.chain();
        let title = format!("Error: {}", errs.next().unwrap());
        let mut message = title.clone();
        for err in errs {
            write!(message, "\n  Caused by: {}", err).unwrap();
        }
        log::error!("{}", message);
        self.message(DialogMessageLevel::Error, title, message);
    }
}

#[derive(Default)]
pub struct SystemDialog {
    extensions: FileExtensions,
    dir: PathBuf,
}

impl SystemDialog {
    fn file_dialog<R: Renderer>(&self, renderer: &R) -> FileDialog {
        let mut dialog =
            FileDialog::new().set_directory(&self.dir).set_can_create_directories(true);
        if let Some(handles) = renderer.window_handles() {
            dialog = dialog.set_parent(&handles);
        }
        dialog
    }
}

impl Dialog for SystemDialog {
    fn new(config: &Config) -> Result<Self> {
        let extensions = config.watch().file_extensions().clone();
        let dir = config.dialog().default_dir()?;
        Ok(Self { extensions, dir })
    }

    fn pick_files<R: Renderer>(&self, renderer: &R) -> Vec<PathBuf> {
        self.file_dialog(renderer)
            .set_title("Open files to preview")
            .add_filter("Markdown", self.extensions.as_slice())
            .pick_files()
            .unwrap_or_default()
    }

    fn pick_dirs<R: Renderer>(&self, renderer: &R) -> Vec<PathBuf> {
        self.file_dialog(renderer)
            .set_title("Choose directories to watch")
            .pick_folders()
            .unwrap_or_default()
    }

    fn message(
        &self,
        level: DialogMessageLevel,
        title: impl Into<String>,
        body: impl Into<String>,
    ) {
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
