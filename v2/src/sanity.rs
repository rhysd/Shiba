use crate::renderer::{MessageFromRenderer, UserEvent, UserEventSender};
use std::thread::{sleep, spawn};
use std::time::Duration;

pub struct SanityTest<S> {
    sender: Option<S>,
}

impl<S: UserEventSender> SanityTest<S> {
    pub fn new(sender: S) -> Self {
        Self { sender: Some(sender) }
    }

    pub fn run_test(&mut self) {
        let Some(sender) = self.sender.take() else {
            return;
        };

        log::debug!("Start sanity test. This app will quit soon");
        spawn(move || {
            use MessageFromRenderer::*;

            let messages = [
                OpenFile { path: "README.md".to_string() },
                OpenFile {
                    #[cfg(target_os = "windows")]
                    path: r"docs\installation.md".to_string(),
                    #[cfg(not(target_os = "windows"))]
                    path: "docs/installation.md".to_string(),
                },
                Back,
                Forward,
                Reload,
                ToggleMaximized,
                ToggleMaximized,
                Quit,
            ];

            for msg in messages {
                sleep(Duration::from_millis(1000));
                log::debug!("Sanity test case is about to send message: {msg:?}");
                sender.send(UserEvent::IpcMessage(msg));
            }
        });
    }
}
