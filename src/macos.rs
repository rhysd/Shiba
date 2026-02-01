use anyhow::Result;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSImage};
use objc2_foundation::NSData;
use std::env::current_dir;
use std::io::{IsTerminal, stdout};

pub fn set_dock_icon(icon: &[u8]) -> Result<()> {
    if current_dir().map_or(true, |d| &d == "/") && !stdout().is_terminal() {
        log::debug!("Icon is not set to the dock because this process is run as macOS app");
        return Ok(());
    }

    let Some(mtm) = MainThreadMarker::new() else {
        anyhow::bail!("Dock icon cannot be set outside the main thread");
    };
    let app = NSApplication::sharedApplication(mtm);
    let data = NSData::with_bytes(icon);
    let alloc = app.mtm().alloc();
    let Some(image) = NSImage::initWithData(alloc, &data) else {
        anyhow::bail!("Invalid icon image data");
    };

    // Safety: The icon image argument is not None.
    unsafe {
        app.setApplicationIconImage(Some(&image));
    }

    Ok(())
}
