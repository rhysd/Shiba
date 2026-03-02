use tao::dpi::{PhysicalPosition, PhysicalSize};
use tao::monitor::MonitorHandle;

pub trait MonitorExtWorkArea {
    fn work_area(&self) -> (PhysicalSize<u32>, PhysicalPosition<i32>);
}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use objc2_app_kit::NSScreen;
    use tao::dpi::{LogicalPosition, LogicalSize};
    use tao::platform::macos::MonitorHandleExtMacOS;

    impl MonitorExtWorkArea for MonitorHandle {
        fn work_area(&self) -> (PhysicalSize<u32>, PhysicalPosition<i32>) {
            let Some(screen) = self.ns_screen() else {
                return (self.size(), self.position());
            };
            assert!(!screen.is_null());
            // Safety: `screen` is the valid value returned from `Monitor::ns_screen` and checked as non-null.
            let screen = unsafe { &*(screen as *const NSScreen) };
            let frame = screen.frame();
            let visible_frame = screen.visibleFrame();
            let factor = self.scale_factor();
            let position = self.position().to_logical::<f64>(factor);
            let x = position.x + visible_frame.origin.x - frame.origin.x;
            let y = position.y + visible_frame.origin.y - frame.origin.y;
            let position = LogicalPosition::new(x, y).to_physical(factor);
            let size = LogicalSize::new(visible_frame.size.width, visible_frame.size.height)
                .to_physical(factor);
            (size, position)
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::mem::size_of;
    use tao::platform::windows::MonitorHandleExtWindows;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};

    impl MonitorExtWorkArea for MonitorHandle {
        fn work_area(&self) -> (PhysicalSize<u32>, PhysicalPosition<i32>) {
            let mut info =
                MONITORINFO { cbSize: size_of::<MONITORINFO>() as _, ..Default::default() };
            let hmonitor = HMONITOR(self.hmonitor() as _);

            // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmonitorinfow
            // Safety: `hmonitor` handle was got from `MonitorHandle::hmonitor()` and `MONITORINFO` has correct `cbSize`.
            let status = unsafe { GetMonitorInfoW(hmonitor, &mut info) };
            if !status.as_bool() {
                return (self.size(), self.position());
            }

            let width = (info.rcWork.right - info.rcWork.left) as u32;
            let height = (info.rcWork.bottom - info.rcWork.top) as u32;
            let size = PhysicalSize::new(width, height);
            let position = PhysicalPosition::new(info.rcWork.left, info.rcWork.top);
            (size, position)
        }
    }
}

#[cfg(target_os = "linux")]
mod macos {
    use super::*;

    impl MonitorExtWorkArea for MonitorHandle {
        fn work_area(&self) -> (PhysicalSize<u32>, PhysicalPosition<i32>) {
            // TODO
            (self.size(), self.position())
        }
    }
}
