use crate::cli::Options;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[non_exhaustive]
#[derive(Clone, Copy, Serialize)]
pub enum KeyAction {
    Forward,
    Back,
    Reload,
    OpenFile,
    #[allow(unused)]
    OpenDir,
    ScrollDown,
    ScrollUp,
    ScrollPageDown,
    ScrollPageUp,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageToRenderer<'a> {
    Content { content: &'a str },
    KeyMappings { keymaps: HashMap<String, KeyAction> },
    Debug,
}

impl<'a> MessageToRenderer<'a> {
    pub fn default_key_mappings() -> Self {
        use KeyAction::*;

        #[rustfmt::skip]
        const DEFAULT_MAPPINGS: &[(&str, KeyAction)] = &[
            ("j",      ScrollDown),
            ("k",      ScrollUp),
            ("h",      Back),
            ("l",      Forward),
            ("r",      Reload),
            ("ctrl+o", OpenFile),
            ("ctrl+f", ScrollPageDown),
            ("ctrl+b", ScrollPageUp),
        ];

        let mut m = HashMap::new();
        for (bind, action) in DEFAULT_MAPPINGS {
            m.insert(bind.to_string(), *action);
        }

        Self::KeyMappings { keymaps: m }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageFromRenderer {
    Init,
    Open { link: String },
    Reload,
    FileDialog,
    DirDialog,
    Forward,
    Back,
}

#[derive(Debug)]
pub enum UserEvent {
    IpcMessage(MessageFromRenderer),
    FileDrop(PathBuf),
    WatchedFilesChanged(Vec<PathBuf>),
    Error(Error),
}

#[derive(Debug)]
pub enum MenuItem {
    Quit,
    Forward,
    Back,
    Reload,
    OpenFile,
    WatchDir,
}

pub trait MenuItems {
    type ItemId: fmt::Debug;
    fn item_from_id(&self, id: Self::ItemId) -> Result<MenuItem>;
}

pub trait Renderer: Sized {
    type EventLoop;
    type Menu: MenuItems;

    fn open(options: &Options, event_loop: &Self::EventLoop, html: &str) -> Result<Self>;
    fn set_menu(&self) -> Self::Menu;
    fn send_message(&self, message: MessageToRenderer) -> Result<()>;
}
