/// GPU renderer using wgpu
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::chunk::Point;

/// Camera uniform buffer data
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    time: f32,
}

/// Floor vertex
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct FloorVertex {
    position: [f32; 3],
}

impl FloorVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FloorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Main GPU renderer
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    // Point rendering
    point_pipeline: wgpu::RenderPipeline,
    point_bind_group_layout: wgpu::BindGroupLayout,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    point_storage_buffer: Option<wgpu::Buffer>,
    point_bind_group: Option<wgpu::BindGroup>,
    point_count: u32,

    // Floor rendering
    floor_pipeline: wgpu::RenderPipeline,
    floor_vertex_buffer: wgpu::Buffer,
    floor_index_buffer: wgpu::Buffer,
    floor_index_count: u32,

    // egui renderer
    egui_renderer: egui_wgpu::Renderer,

    time: f32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Main Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Point storage bind group layout
        let point_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Point Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Point rendering pipeline
        let point_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Point Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/point.wgsl").into()),
        });

        let point_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Point Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &point_bind_group_layout],
                push_constant_ranges: &[],
            });

        let point_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Point Pipeline"),
            layout: Some(&point_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &point_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &point_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Floor geometry
        let floor_size = 1000.0;
        let floor_vertices = vec![
            FloorVertex {
                position: [-floor_size, 0.0, -floor_size],
            },
            FloorVertex {
                position: [floor_size, 0.0, -floor_size],
            },
            FloorVertex {
                position: [floor_size, 0.0, floor_size],
            },
            FloorVertex {
                position: [-floor_size, 0.0, floor_size],
            },
        ];

        let floor_indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let floor_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Floor Vertex Buffer"),
            contents: bytemuck::cast_slice(&floor_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let floor_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Floor Index Buffer"),
            contents: bytemuck::cast_slice(&floor_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Floor shader
        let floor_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Floor Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/floor.wgsl").into()),
        });

        let floor_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Floor Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let floor_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Floor Pipeline"),
            layout: Some(&floor_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &floor_shader,
                entry_point: "vs_main",
                buffers: &[FloorVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &floor_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
        });

        // Initialize egui renderer
        let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, None, 1);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            point_pipeline,
            point_bind_group_layout,
            camera_buffer,
            camera_bind_group,
            point_storage_buffer: None,
            point_bind_group: None,
            point_count: 0,
            floor_pipeline,
            floor_vertex_buffer,
            floor_index_buffer,
            floor_index_count: floor_indices.len() as u32,
            egui_renderer,
            time: 0.0,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_points(&mut self, points: Vec<Point>) {
        if points.is_empty() {
            self.point_storage_buffer = None;
            self.point_bind_group = None;
            self.point_count = 0;
            return;
        }

        self.point_count = points.len() as u32;

        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Point Storage Buffer"),
                contents: bytemuck::cast_slice(&points),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Point Bind Group"),
            layout: &self.point_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        self.point_storage_buffer = Some(buffer);
        self.point_bind_group = Some(bind_group);
    }

    pub fn render(
        &mut self,
        view_proj: Mat4,
        camera_pos: Vec3,
        dt: f32,
        egui_primitives: Vec<egui::ClippedPrimitive>,
        egui_textures_delta: egui::TexturesDelta,
        screen_descriptor: egui_wgpu::ScreenDescriptor,
    ) -> Result<(), wgpu::SurfaceError> {
        self.time += dt;

        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            time: self.time,
        };

        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Update egui textures
        for (id, image_delta) in &egui_textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, image_delta);
        }

        // Update egui buffers
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &egui_primitives,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            // Render floor
            render_pass.set_pipeline(&self.floor_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.floor_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.floor_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.floor_index_count, 0, 0..1);

            // Render points
            if let Some(point_bind_group) = &self.point_bind_group {
                render_pass.set_pipeline(&self.point_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, point_bind_group, &[]);
                render_pass.draw(0..6, 0..self.point_count);
            }

            // Render egui
            self.egui_renderer.render(&mut render_pass, &egui_primitives, &screen_descriptor);
        }

        // Free egui textures
        for id in &egui_textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
}
