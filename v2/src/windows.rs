use windows_sys::Win32::System::Console::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};

pub struct WindowsConsole {
    success: bool,
}

impl WindowsConsole {
    pub fn attach() -> Self {
        let success = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) != 0 };
        Self { success }
    }
}

impl Drop for WindowsConsole {
    fn drop(&mut self) {
        if self.success {
            unsafe { FreeConsole() };
        }
    }
}
