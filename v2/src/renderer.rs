use crate::cli::Options;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[non_exhaustive]
#[derive(Serialize)]
pub enum KeyAction {
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
}

impl<'a> MessageToRenderer<'a> {
    pub fn default_key_mappings() -> Self {
        let mut m = HashMap::new();
        m.insert("j".to_string(), KeyAction::ScrollDown);
        m.insert("k".to_string(), KeyAction::ScrollUp);
        m.insert("ctrl+f".to_string(), KeyAction::ScrollPageDown);
        m.insert("ctrl+b".to_string(), KeyAction::ScrollPageUp);
        Self::KeyMappings { keymaps: m }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum MessageFromRenderer {
    Init,
    Open { link: String },
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
