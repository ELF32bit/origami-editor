use winit::window::Window;
use std::{intrinsics::sinf64, sync::Arc};

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::vertex::{create_vertex_buffer_layout, VERTEX_INDEX_LIST, VERTEX_LIST};

pub struct GraphicsContext<'window> {
	surface: wgpu::Surface<'window>,
	surface_configuration: wgpu::SurfaceConfiguration,
	adapter: wgpu::Adapter,
	device: wgpu::Device,
	queue: wgpu::Queue,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	vertex_index_buffer: wgpu::Buffer,
}

impl<'window> GraphicsContext<'window> {
	pub async fn new_async(window: Arc<Window>) -> GraphicsContext<'window> {
		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			flags: wgpu::InstanceFlags::default(),
			backend_options: wgpu::BackendOptions::default(),
		});
		let surface = instance.create_surface(Arc::clone(&window)).unwrap();
		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		}).await.expect("Failed to find an appropriate adapter");

		let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
			label: None,
			required_features: wgpu::Features::empty(),
			required_limits: wgpu::Limits::default(),
			memory_hints: wgpu::MemoryHints::Performance,
			trace: wgpu::Trace::Off,
		}).await.expect("Failed to create device");

		let window_size = window.inner_size();
		let surface_capabilities = surface.get_capabilities(&adapter);
		let surface_format = surface_capabilities.formats[0];
		let surface_configuration = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: window_size.width.max(1),
			height: window_size.height.max(1),
			present_mode: wgpu::PresentMode::Fifo,
			desired_maximum_frame_latency: 2,
			alpha_mode: surface_capabilities.alpha_modes[0],
			view_formats: vec![],
		};
		surface.configure(&device, &surface_configuration);

		let render_pipeline = create_pipeline(&device, surface_configuration.format);

		let bytes: &[u8] = bytemuck::cast_slice(&VERTEX_LIST);
		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: None,
			contents: bytes,
			usage: wgpu::BufferUsages::VERTEX,
		});
		let vertex_index_bytes = bytemuck::cast_slice(&VERTEX_INDEX_LIST);
		let vertex_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: None,
			contents: vertex_index_bytes,
			usage: wgpu::BufferUsages::INDEX,
		});

		return GraphicsContext {
			surface,
			surface_configuration,
			adapter,
			device,
			queue,
			render_pipeline,
			vertex_buffer,
			vertex_index_buffer,
		};
	}

	pub fn new(window: Arc<Window>) -> GraphicsContext<'window> {
		pollster::block_on(GraphicsContext::new_async(window))
	}

	pub fn resize(&mut self, new_size: (u32, u32)) {
		let (width, height) = new_size;
		self.surface_configuration.width = width.max(1);
		self.surface_configuration.height = height.max(1);
		self.surface.configure(&self.device, &self.surface_configuration);
	}

	pub fn draw(&mut self) {
		let surface_texture = self.surface.get_current_texture()
			.expect("Failed to acquire next swap chain texture");
		let surface_texture_view = surface_texture.texture
			.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &surface_texture_view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			render_pass.set_pipeline(&self.render_pipeline);

			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.set_index_buffer(self.vertex_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
			render_pass.draw_indexed(0..VERTEX_INDEX_LIST.len() as u32, 0, 0..1);
			render_pass.draw(0..VERTEX_LIST.len() as u32, 0..1);
		}

		self.queue.submit(Some(encoder.finish()));
		surface_texture.present();
	}
}

fn create_pipeline(device: &wgpu::Device, swap_chain_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
	let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: None,
		source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("shader.wgsl"))),
	});
	return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: None,
		layout: None,
		vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[
                create_vertex_buffer_layout()
            ],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(swap_chain_format.into())],
        }),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: None,
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
		cache: None,
	});
}
