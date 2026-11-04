use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use image::RgbaImage;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    camera::Camera,
    world::{Vertex, WorldMesh},
};

pub fn run(mesh: WorldMesh, textures: [RgbaImage; 4]) -> Result<(), String> {
    let event_loop = EventLoop::new().map_err(|error| error.to_string())?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        mesh: Some(mesh),
        textures: Some(textures),
        window: None,
        state: None,
        failure: None,
        suspended: false,
    };
    event_loop
        .run_app(&mut app)
        .map_err(|error| error.to_string())?;
    if let Some(error) = app.failure {
        return Err(error.into());
    }
    Ok(())
}

struct App {
    mesh: Option<WorldMesh>,
    textures: Option<[RgbaImage; 4]>,
    window: Option<Arc<Window>>,
    state: Option<State>,
    failure: Option<String>,
    suspended: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("NEXPERT | Starting GPU client…")
                .with_inner_size(PhysicalSize::new(1280, 800)),
        ) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.failure = Some(format!("Could not create game window: {error}"));
                event_loop.exit();
                return;
            }
        };
        match pollster::block_on(State::new(
            window.clone(),
            self.mesh.take().unwrap(),
            self.textures.take().unwrap(),
        )) {
            Ok(state) => {
                self.window = Some(window);
                self.state = Some(state);
            }
            Err(error) => {
                self.failure = Some(error);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let (Some(window), Some(state)) = (&self.window, &mut self.state) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size),
            WindowEvent::Occluded(suspended) => self.suspended = suspended,
            WindowEvent::Focused(false) => {
                state.keys.clear();
                state.cursor_locked = false;
                window.set_cursor_visible(true);
                let _ = window.set_cursor_grab(CursorGrabMode::None);
            }
            WindowEvent::RedrawRequested => state.draw(window),
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    if key == KeyCode::Escape && event.state == ElementState::Pressed {
                        event_loop.exit();
                    }
                    if key == KeyCode::Enter && event.state == ElementState::Pressed {
                        let _ = window
                            .set_cursor_grab(CursorGrabMode::Locked)
                            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
                        window.set_cursor_visible(false);
                        state.cursor_locked = true;
                    }
                    if event.state == ElementState::Pressed {
                        state.keys.insert(key);
                    } else {
                        state.keys.remove(&key);
                    }
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: DeviceEvent) {
        if let (Some(state), DeviceEvent::MouseMotion { delta }) = (&mut self.state, event) {
            if state.cursor_locked {
                state.camera.look(delta.0 as f32, delta.1 as f32);
            }
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if !self.suspended
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_group: wgpu::BindGroup,
    texture_groups: [wgpu::BindGroup; 4],
    buffers: [Option<wgpu::Buffer>; 4],
    counts: [u32; 4],
    depth: wgpu::TextureView,
    camera: Camera,
    keys: HashSet<KeyCode>,
    last_frame: Instant,
    fps_started: Instant,
    frames: u32,
    cursor_locked: bool,
    gpu_label: String,
}

impl State {
    async fn new(
        window: Arc<Window>,
        mesh: WorldMesh,
        images: [RgbaImage; 4],
    ) -> Result<Self, String> {
        let size = window.inner_size();
        let mut instance_desc =
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(window.clone()));
        instance_desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(instance_desc);
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| format!("Could not create GPU surface: {error}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|error| format!("No compatible GPU adapter found: {error}"))?;
        let gpu_info = adapter.get_info();
        let gpu_label = format!("{} · {:?}", gpu_info.name, gpu_info.backend);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|error| format!("Could not initialize GPU device: {error}"))?;
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("GPU does not support a presentation surface")?;
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("block texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let camera = Camera::default();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera uniform"),
            contents: bytemuck::cast_slice(
                &camera
                    .view_projection(config.width as f32 / config.height as f32)
                    .to_cols_array(),
            ),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest block sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let texture_groups = images.map(|image| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("local Minecraft texture"),
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                image.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image.width()),
                    rows_per_image: Some(image.height()),
                },
                wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
            );
            let view = texture.create_view(&Default::default());
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("block texture bind group"),
                layout: &texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("voxel shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("voxel pipeline layout"),
            bind_group_layouts: &[Some(&camera_layout), Some(&texture_layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("voxel pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                })],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let groups = [mesh.grass_top, mesh.grass_side, mesh.dirt, mesh.stone];
        let counts = groups.each_ref().map(|vertices| vertices.len() as u32);
        let buffers = groups.map(|vertices| {
            if vertices.is_empty() {
                None
            } else {
                Some(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("voxel face vertices"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
                )
            }
        });
        let depth = depth_view(&device, config.width, config.height);
        window.set_title(&format!(
            "NEXPERT | {gpu_label} | WASD + mouse · Enter capture · Esc exit"
        ));
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            camera_buffer,
            camera_group,
            texture_groups,
            buffers,
            counts,
            depth,
            camera,
            keys: HashSet::new(),
            last_frame: Instant::now(),
            fps_started: Instant::now(),
            frames: 0,
            cursor_locked: false,
            gpu_label,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth = depth_view(&self.device, size.width, size.height);
    }

    fn draw(&mut self, window: &Window) {
        let now = Instant::now();
        let delta = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        let forward = f32::from(self.keys.contains(&KeyCode::KeyW))
            - f32::from(self.keys.contains(&KeyCode::KeyS));
        let right = f32::from(self.keys.contains(&KeyCode::KeyD))
            - f32::from(self.keys.contains(&KeyCode::KeyA));
        let up = f32::from(self.keys.contains(&KeyCode::Space))
            - f32::from(self.keys.contains(&KeyCode::ShiftLeft));
        self.camera.move_local(forward, right, up, delta);
        let matrix = self
            .camera
            .view_projection(self.config.width as f32 / self.config.height.max(1) as f32);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&matrix.to_cols_array()),
        );

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return,
        };
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("game frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("voxel world"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.39,
                            g: 0.67,
                            b: 0.86,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.camera_group, &[]);
            for i in 0..4 {
                if let Some(buffer) = &self.buffers[i] {
                    pass.set_bind_group(1, &self.texture_groups[i], &[]);
                    pass.set_vertex_buffer(0, buffer.slice(..));
                    pass.draw(0..self.counts[i], 0..1);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        self.frames += 1;
        if self.fps_started.elapsed() >= Duration::from_secs(1) {
            let fps = self.frames as f32 / self.fps_started.elapsed().as_secs_f32();
            window.set_title(&format!(
                "NEXPERT | {} | {fps:.0} FPS | {:.1} ms | WASD + mouse · Enter capture · Esc exit",
                self.gpu_label,
                1000.0 / fps.max(1.0)
            ));
            self.frames = 0;
            self.fps_started = Instant::now();
        }
    }
}

fn depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("world depth"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}
