use tao::event_loop::{EventLoopProxy, EventLoopWindowTarget};
use tao::window::WindowId;

pub type Request = crate::renderer::Request<WindowId>;
pub type Event = crate::renderer::Event<WindowId>;
pub type EventLoop = EventLoopWindowTarget<Request>;
pub type Proxy = EventLoopProxy<Request>;
