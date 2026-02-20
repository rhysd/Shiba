use crate::renderer::{
    MessageToRenderer, RawMessageWriter, Renderer, WindowAppearance, WindowHandles, WindowState,
    ZoomLevel,
};
use anyhow::Result;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub struct TestRenderer {
    pub messages: RefCell<Vec<String>>,
    pub title: RefCell<String>,
    pub window_state: Option<WindowState>,
    pub is_visible: AtomicBool,
    pub print_called: AtomicBool,
    pub zoom_level: ZoomLevel,
    pub always_on_top: bool,
    pub drag_started: AtomicBool,
    pub is_maximized: bool,
    pub window_appearance: WindowAppearance,
    pub context_menu_pos: RefCell<Option<(f64, f64)>>,
    pub menu_visible: bool,
    pub is_low_memory: bool,
    pub cookie_deleted: AtomicBool,
    pub window_handles_requested: AtomicBool,
}

impl Renderer for TestRenderer {
    fn send_message(&self, message: MessageToRenderer<'_>) -> Result<()> {
        let msg = serde_json::to_string_pretty(&message)?;
        self.messages.borrow_mut().push(msg);
        Ok(())
    }

    fn send_message_raw<W: RawMessageWriter>(&self, writer: W) -> Result<W::Output> {
        let mut msg = vec![];
        let out = writer.write_to(&mut msg)?;
        self.messages.borrow_mut().push(String::from_utf8(msg)?);
        Ok(out)
    }

    fn set_title(&self, title: &str) {
        *self.title.borrow_mut() = title.into();
    }

    fn window_state(&self) -> Option<WindowState> {
        self.window_state.clone()
    }

    fn show(&self) {
        self.is_visible.store(true, Ordering::Relaxed);
    }

    fn hide(&self) {
        self.is_visible.store(false, Ordering::Relaxed);
    }

    fn print(&self) -> Result<()> {
        self.print_called.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn zoom(&mut self, level: ZoomLevel) -> Result<()> {
        self.zoom_level = level;
        Ok(())
    }

    fn zoom_level(&self) -> ZoomLevel {
        self.zoom_level
    }

    fn set_always_on_top(&mut self, enabled: bool) {
        self.always_on_top = enabled;
    }

    fn always_on_top(&self) -> bool {
        self.always_on_top
    }

    fn drag_window(&self) -> Result<()> {
        self.drag_started.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn is_maximized(&self) -> bool {
        self.is_maximized
    }

    fn set_maximized(&mut self, maximized: bool) {
        self.is_maximized = maximized;
    }

    fn window_appearance(&self) -> WindowAppearance {
        self.window_appearance
    }

    fn show_menu_at(&self, position: Option<(f64, f64)>) {
        *self.context_menu_pos.borrow_mut() = position;
    }

    fn toggle_menu(&mut self) -> Result<()> {
        self.menu_visible = !self.menu_visible;
        Ok(())
    }

    fn save_memory(&mut self, is_low: bool) -> Result<()> {
        self.is_low_memory = is_low;
        Ok(())
    }

    fn delete_cookies(&self) -> Result<()> {
        self.cookie_deleted.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn window_handles(&self) -> WindowHandles<'_> {
        self.window_handles_requested.store(true, Ordering::Relaxed);
        WindowHandles::unsupported()
    }
}
