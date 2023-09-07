use anyhow::Result;
use std::ffi::OsStr;

pub trait Opener: Default {
    fn open(&mut self, path: impl AsRef<OsStr>) -> Result<()>;
}

#[derive(Default)]
pub struct SystemOpener;

impl Opener for SystemOpener {
    fn open(&mut self, path: impl AsRef<OsStr>) -> Result<()> {
        Ok(open::that(path)?)
    }
}
