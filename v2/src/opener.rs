use anyhow::Result;
use std::ffi::OsStr;

pub trait Opener {
    fn new() -> Self;
    fn open(&self, path: impl AsRef<OsStr>) -> Result<()>;
}

pub struct SystemOpener;

impl Opener for SystemOpener {
    fn new() -> Self {
        Self
    }

    fn open(&self, path: impl AsRef<OsStr>) -> Result<()> {
        Ok(open::that(path)?)
    }
}
