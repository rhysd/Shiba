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

    pub fn quit_after_secs(&mut self, seconds: u64) {
        let Some(sender) = self.sender.take() else {
            return;
        };

        // TODO: Send more messages to test more features

        spawn(move || {
            sleep(Duration::from_secs(seconds));
            sender.send(UserEvent::IpcMessage(MessageFromRenderer::Quit));
        });
    }
}
