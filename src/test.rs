use crate::config::Config;
use crate::renderer::{
    Event, EventHandler, MessageToWindow, RawMessageWriter, Renderer, RendererHandle, Request,
    Window, WindowAppearance, WindowHandles, WindowState, ZoomLevel,
};
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Default)]
pub struct TestWindow {
    pub messages: RefCell<Vec<String>>,
    pub title: RefCell<String>,
    pub window_state: Option<WindowState>,
    pub is_visible: AtomicBool,
    pub print_called: AtomicBool,
    pub zoom_level: ZoomLevel,
    pub always_on_top: bool,
    pub drag_started: AtomicBool,
    pub is_maximized: bool,
    pub is_minimized: bool,
    pub window_appearance: WindowAppearance,
    pub context_menu_pos: RefCell<Option<(f64, f64)>>,
    pub menu_visible: bool,
    pub is_low_memory: bool,
    pub cache_deleted: bool,
    pub window_handles_requested: AtomicBool,
    pub window_id: u32,
    pub is_focused: AtomicBool,
}

impl TestWindow {
    pub fn new() -> Self {
        static ID: AtomicU32 = AtomicU32::new(0);

        let mut w = Self::default();
        let id = ID.load(Ordering::Relaxed);
        w.window_id = id;
        ID.store(id + 1, Ordering::Relaxed);
        w
    }
}

impl Window for TestWindow {
    type Id = u32;

    fn send_message(&self, message: MessageToWindow<'_>) -> Result<()> {
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

    fn state(&self) -> Option<WindowState> {
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

    fn maximize(&mut self, maximized: bool) {
        self.is_maximized = maximized;
    }

    fn is_minimized(&self) -> bool {
        self.is_minimized
    }

    fn minimize(&mut self, minimized: bool) {
        self.is_minimized = minimized;
    }

    fn appearance(&self) -> WindowAppearance {
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

    fn delete_cache(&mut self) -> Result<()> {
        self.cache_deleted = true;
        Ok(())
    }

    fn handles(&self) -> WindowHandles<'_> {
        self.window_handles_requested.store(true, Ordering::Relaxed);
        WindowHandles::unavailable()
    }

    fn id(&self) -> Self::Id {
        self.window_id
    }

    fn focus(&self) {
        self.is_focused.store(true, Ordering::Relaxed);
    }
}

#[derive(Clone)]
pub struct TestRendererHandle {
    tx: Sender<Request<u32>>,
}

impl RendererHandle for TestRendererHandle {
    type WindowId = u32;

    fn send(&self, event: Event<Self::WindowId>) {
        self.tx.send(Request::Emit(event)).unwrap();
    }

    fn create_window(&self) {
        self.tx.send(Request::CreateWindow).unwrap();
    }
}

pub struct TestRenderer {
    tx: Sender<Request<u32>>,
    rx: Receiver<Request<u32>>,
}

impl TestRenderer {
    pub fn recv(&self) -> Request<u32> {
        self.rx.try_recv().unwrap()
    }
}

impl Renderer for TestRenderer {
    type WindowId = u32;
    type Window = TestWindow;
    type Handle = TestRendererHandle;

    fn new(_: Rc<Config>) -> Result<Self> {
        let (tx, rx) = channel();
        Ok(Self { tx, rx })
    }

    fn create_handle(&self) -> Self::Handle {
        TestRendererHandle { tx: self.tx.clone() }
    }

    fn start<H>(self, _: H) -> !
    where
        H: EventHandler<Window = Self::Window, WindowId = Self::WindowId> + 'static,
    {
        unreachable!()
    }
}

#[test]
fn test_renderer_create_window() {
    let renderer = TestRenderer::new(Rc::new(Config::default())).unwrap();
    let handle = renderer.create_handle();
    handle.create_window();
    let req = renderer.recv();
    assert!(matches!(req, Request::CreateWindow), "request={req:?}");
}
