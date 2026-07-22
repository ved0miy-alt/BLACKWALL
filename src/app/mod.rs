/// Application core - main event loop and state management
use anyhow::Result;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use crate::camera::Camera;
use crate::chunk::ChunkManager;
use crate::config::EngineConfig;
use crate::debug::DebugUI;
use crate::network::{NetworkParams, NetworkSimulator};
use crate::noise::{FbmNoise, NoiseGenerator, PerlinNoise};
use crate::renderer::Renderer;

/// Application state
pub struct App {
    window: Arc<Window>,
    renderer: Renderer,
    camera: Camera,
    chunk_manager: ChunkManager,
    network_receiver: Receiver<NetworkParams>,
    current_params: NetworkParams,

    // Debug UI
    debug_ui: DebugUI,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,

    // Timing
    last_update: Instant,
    frame_times: Vec<f32>,

    // Input state
    cursor_grabbed: bool,
}

impl App {
    pub fn new(window: Arc<Window>, use_simulation: bool) -> Result<(Self, NetworkSimulator)> {
        let (_network_sim, simulation_receiver, capture_receiver) = NetworkSimulator::new();

        // Choose receiver based on mode
        let network_receiver = if use_simulation {
            simulation_receiver
        } else {
            capture_receiver
        };

        // Initialize renderer
        let renderer = pollster::block_on(Renderer::new(Arc::clone(&window)))?;

        // Initialize camera
        let size = window.inner_size();
        let camera = Camera::new(
            glam::Vec3::new(0.0, 20.0, 0.0),
            10.0,  // speed
            0.002, // sensitivity
            70.0_f32.to_radians(),
            size.width as f32 / size.height as f32,
        );

        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
        );

        // Initialize noise generator
        let base_noise = Box::new(PerlinNoise::new(42, 1.0, 1.0));
        let fbm_noise: Box<dyn NoiseGenerator> = Box::new(FbmNoise::new(base_noise, 4, 2.0, 0.5));
        let noise_arc = Arc::new(fbm_noise);

        // Initialize chunk manager
        let chunk_manager = ChunkManager::new(100.0, 300.0, noise_arc);

        let app = Self {
            window,
            renderer,
            camera,
            chunk_manager,
            network_receiver,
            current_params: NetworkParams::default(),
            debug_ui: DebugUI::default(),
            egui_ctx,
            egui_state,
            last_update: Instant::now(),
            frame_times: Vec::with_capacity(60),
            cursor_grabbed: false,
        };

        Ok((app, _network_sim))
    }

    fn update(&mut self, dt: f32) {
        // Update network parameters from receiver
        while let Ok(params) = self.network_receiver.try_recv() {
            self.current_params = params;
        }

        // Update camera
        self.camera.update(dt);

        // Update chunks EVERY frame for instant updates
        self.chunk_manager
            .update(self.camera.position, self.current_params);

        // Update debug UI
        self.frame_times.push(dt);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        self.debug_ui.fps = 1.0 / avg_frame_time;
        self.debug_ui.camera_pos = self.camera.position;
        self.debug_ui.chunk_count = self.chunk_manager.chunk_count();
        self.debug_ui.point_count = self.chunk_manager.point_count();
        self.debug_ui.density = self.current_params.density;
        self.debug_ui.chaos = self.current_params.chaos;
        self.debug_ui.flow = self.current_params.flow;
        self.debug_ui.entropy = self.current_params.entropy;
        self.debug_ui.packet_rate = self.current_params.packet_rate;
        self.debug_ui.energy = self.current_params.energy;
        self.debug_ui.frequency = self.current_params.frequency;
        self.debug_ui.curvature = self.current_params.curvature;
        self.debug_ui.packets_per_sec = self.current_params.packets_per_sec;
        self.debug_ui.bytes_per_sec = self.current_params.bytes_per_sec;
        self.debug_ui.tcp_count = self.current_params.tcp_count;
        self.debug_ui.udp_count = self.current_params.udp_count;
    }

    fn render(&mut self) -> Result<()> {
        let dt = self.last_update.elapsed().as_secs_f32();

        // Update point buffer
        let points = self.chunk_manager.get_visible_points();
        self.renderer.update_points(points);

        // Render egui
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            self.debug_ui.ui(ctx);
        });

        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let egui_primitives = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.renderer.size().width, self.renderer.size().height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        // Render scene
        let view_proj = self.camera.view_projection_matrix();
        match self
            .renderer
            .render(view_proj, self.camera.position, dt, egui_primitives, full_output.textures_delta, screen_descriptor)
        {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => self.renderer.resize(self.renderer.size()),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                return Err(anyhow::anyhow!("Out of memory"));
            }
            Err(e) => eprintln!("Render error: {:?}", e),
        }

        Ok(())
    }

    fn grab_cursor(&mut self) {
        self.window.set_cursor_visible(false);
        let _ = self
            .window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined);
        self.cursor_grabbed = true;
    }

    fn release_cursor(&mut self) {
        self.window.set_cursor_visible(true);
        let _ = self
            .window
            .set_cursor_grab(winit::window::CursorGrabMode::None);
        self.cursor_grabbed = false;
    }

    fn handle_window_event(&mut self, event: WindowEvent, should_exit: &mut bool) {
        // Handle egui events first
        let _ = self.egui_state.on_window_event(&self.window, &event);

        match event {
            WindowEvent::CloseRequested => {
                *should_exit = true;
            }
            WindowEvent::Resized(physical_size) => {
                self.renderer.resize(physical_size);
                self.camera
                    .set_aspect(physical_size.width as f32 / physical_size.height as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        match keycode {
                            KeyCode::Escape => {
                                if self.cursor_grabbed {
                                    self.release_cursor();
                                } else {
                                    *should_exit = true;
                                }
                            }
                            KeyCode::F1 => {
                                self.debug_ui.toggle();
                            }
                            _ => {}
                        }
                    }
                }

                self.camera.handle_keyboard(&event);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if button == winit::event::MouseButton::Left && state == ElementState::Pressed {
                    if !self.cursor_grabbed {
                        self.grab_cursor();
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.cursor_grabbed {
                self.camera.handle_mouse_move(delta.0, delta.1);
            }
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) -> Result<()> {
        self.grab_cursor();
        let mut should_exit = false;

        let _ = event_loop.run(move |event, control_flow| {
            if should_exit {
                control_flow.exit();
                return;
            }

            match event {
                Event::WindowEvent { event, .. } => {
                    self.handle_window_event(event, &mut should_exit);
                }
                Event::DeviceEvent { event, .. } => {
                    self.handle_device_event(event);
                }
                Event::AboutToWait => {
                    let now = Instant::now();
                    let dt = now.duration_since(self.last_update).as_secs_f32();
                    self.last_update = now;

                    self.update(dt);
                    let _ = self.render();
                    self.window.request_redraw();
                }
                _ => {}
            }
        });

        Ok(())
    }
}

pub fn run(config: EngineConfig, use_simulation: bool) -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Cyberspace Engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080))
            .build(&event_loop)?,
    );

    let (app, _network_sim) = App::new(window, use_simulation)?;
    app.run(event_loop)
}
