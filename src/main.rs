mod application;
mod graphics;
mod vertex;

use application::Application;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), EventLoopError> {
	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);
	let mut app = Application::default();
	event_loop.run_app(&mut app)
}
