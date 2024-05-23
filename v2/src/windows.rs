use windows::Win32::System::Console::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};

pub struct WindowsConsole {
    success: bool,
}

impl WindowsConsole {
    pub fn attach() -> Self {
        // SAFETY: Using Windows C API is always unsafe. I confirmed the usage in official document.
        // https://learn.microsoft.com/en-us/windows/console/attachconsole
        match unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } {
            Ok(()) => Self { success: true },
            Err(err) => {
                log::error!("Failed to attach to console: {err}");
                Self { success: false }
            }
        }
    }
}

impl Drop for WindowsConsole {
    fn drop(&mut self) {
        if self.success {
            // SAFETY: Using Windows C API is always unsafe. I confirmed the usage in official document.
            // https://learn.microsoft.com/en-us/windows/console/freeconsole
            if let Err(err) = unsafe { FreeConsole() } {
                log::error!("Failed to free console: {err}");
            }
        }
    }
}
