use tao::dpi::{PhysicalPosition, PhysicalSize};
use tao::monitor::MonitorHandle;

pub trait MonitorExt {
    fn work_area(&self) -> (PhysicalSize<u32>, PhysicalPosition<i32>);
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2_app_kit::NSScreen;
    use tao::dpi::{LogicalPosition, LogicalSize};
    use tao::platform::macos::MonitorHandleExtMacOS;

    impl MonitorExt for MonitorHandle {
        fn work_area(&self) -> (PhysicalSize<u32>, PhysicalPosition<i32>) {
            let Some(screen) = self.ns_screen() else {
                return (self.size(), self.position());
            };
            assert!(!screen.is_null());
            // Safety: `screen` is asserted as non-null
            let screen = unsafe { &*screen.cast::<NSScreen>() };
            let frame = screen.frame();
            let visible_frame = screen.visibleFrame();
            let factor = self.scale_factor();
            let position = self.position().to_logical::<f64>(factor);
            let position = LogicalPosition::new(
                position.x + visible_frame.origin.x - frame.origin.x,
                position.y + visible_frame.origin.y - frame.origin.y,
            )
            .to_physical(factor);
            let size = LogicalSize::new(visible_frame.size.width, visible_frame.size.height)
                .to_physical(factor);
            (size, position)
        }
    }
}

// TODO
#[cfg(not(target_os = "macos"))]
mod macos {
    use super::*;

    impl MonitorExt for MonitorHandle {
        fn work_area(&self) -> (PhysicalSize<u32>, PhysicalPosition<i32>) {
            (self.size(), self.position())
        }
    }
}
