use crate::renderer::{Event, MessageFromWindow, RendererHandle};
use std::env;
use std::thread::{sleep, spawn};
use std::time::Duration;

pub struct SanityTest<H> {
    handle: H,
}

impl<H: RendererHandle> SanityTest<H> {
    pub fn new(handle: H) -> Self {
        Self { handle }
    }

    pub fn run(self, id: H::WindowId) {
        log::debug!("Start sanity test. This app will quit soon");
        spawn(move || {
            use MessageFromWindow::*;

            let cwd = env::current_dir().unwrap();
            let docs = cwd.join("docs");
            let readme = cwd.join("README.md").to_string_lossy().into();
            let installation = docs.join("installation.md").to_string_lossy().into();
            let messages = [
                OpenFile { path: readme, window: false },
                OpenFile { path: installation, window: false },
                GoBack,
                GoForward,
                Reload,
                ToggleMaximized,
                ToggleMaximized,
                Quit,
            ];

            for message in messages {
                sleep(Duration::from_millis(1000));
                log::debug!("Sanity test case is about to send message: {message:?}");
                self.handle.send(Event::WindowMessage { message, id });
            }
        });
    }
}
