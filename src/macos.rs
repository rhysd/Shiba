use anyhow::Result;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSImage};
use objc2_foundation::NSData;
use std::env::current_dir;
use std::io::{IsTerminal, stderr, stdin, stdout};
use std::sync::atomic::{AtomicU8, Ordering};

pub fn is_app_process() -> bool {
    static CACHED: AtomicU8 = AtomicU8::new(u8::MAX); // `u8` for tribool

    // Note: `if let x = ... && ... { ... }` is warned by rustc.
    // https://github.com/rust-lang/rust/issues/139369
    let cached = CACHED.load(Ordering::Relaxed);
    if cached < 2 {
        return cached == 1;
    }

    let ret = !stdout().is_terminal()
        && !stderr().is_terminal()
        && !stdin().is_terminal()
        && current_dir().is_ok_and(|d| &d == "/");
    CACHED.store(ret as u8, Ordering::Relaxed);
    ret
}

pub fn set_dock_icon(icon: &[u8]) -> Result<()> {
    if is_app_process() {
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
