use crate::renderer::{Event, MessageFromWindow, RendererHandle};
use std::thread::{sleep, spawn};
use std::time::Duration;

pub struct SanityTest<H> {
    handle: Option<H>,
}

impl<H: RendererHandle> SanityTest<H> {
    pub fn new(handle: H) -> Self {
        Self { handle: Some(handle) }
    }

    pub fn run_test(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };

        log::debug!("Start sanity test. This app will quit soon");
        spawn(move || {
            use MessageFromWindow::*;

            let messages = [
                OpenFile { path: "README.md".to_string() },
                OpenFile {
                    #[cfg(target_os = "windows")]
                    path: r"docs\installation.md".to_string(),
                    #[cfg(not(target_os = "windows"))]
                    path: "docs/installation.md".to_string(),
                },
                GoBack,
                GoForward,
                Reload,
                ToggleMaximized,
                ToggleMaximized,
                Quit,
            ];

            for msg in messages {
                sleep(Duration::from_millis(1000));
                log::debug!("Sanity test case is about to send message: {msg:?}");
                handle.send(Event::WindowMessage(msg));
            }
        });
    }
}
