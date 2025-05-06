use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::event::{WindowEvent, KeyEvent, ElementState};
use winit::keyboard::{Key, NamedKey};
use std::sync::Arc;

use crate::graphics::GraphicsContext;

#[derive(Default)]
pub struct Application<'window> {
	window: Option<Arc<Window>>,
	graphics_context: Option<GraphicsContext<'window>>,
}

impl<'window> ApplicationHandler for Application<'window> {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_none() {
			let window_attributes = Window::default_attributes()
				.with_title("winit example");
			let window = Arc::new(event_loop.create_window(window_attributes)
				.expect("create window error."));
			self.window = Some(window.clone());

			let graphics_context = GraphicsContext::new(window.clone());
			self.graphics_context = Some(graphics_context);
		}
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::Resized(new_size) => {
				if let (Some(graphics_context), Some(window)) =
					(self.graphics_context.as_mut(), self.window.as_ref())
					{
						graphics_context.resize((new_size.width, new_size.height));
						window.request_redraw();
					}
			}

			WindowEvent::KeyboardInput {
				event: KeyEvent {
					repeat: false,
					state: ElementState::Pressed,
					logical_key: Key::Named(NamedKey::Escape),
					..
				},
				..
			} => {
				event_loop.exit();
			}

			WindowEvent::RedrawRequested => {
				if let Some(graphics_context) = self.graphics_context.as_mut() {
					graphics_context.draw();
				}
			}

			WindowEvent::CloseRequested => {
				event_loop.exit();
			}

			_ => ()
		}
	}
}
