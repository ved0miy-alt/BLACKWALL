/// First-person camera with free flight controls
use glam::{Mat4, Vec3};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Camera state and controls
pub struct Camera {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub right: Vec3,

    yaw: f32,
    pitch: f32,

    // Input state
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    move_fast: bool,

    pub speed: f32,
    pub fast_multiplier: f32,
    pub sensitivity: f32,

    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vec3, speed: f32, sensitivity: f32, fov: f32, aspect: f32) -> Self {
        let mut camera = Self {
            position,
            forward: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::Y,
            right: Vec3::X,
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            move_fast: false,
            speed,
            fast_multiplier: 3.0,
            sensitivity,
            fov,
            aspect,
            near: 1.0,        // Улучшенные значения
            far: 10000.0,     // Для больших расстояний
        };
        camera.update_vectors();
        camera
    }

    /// Handle keyboard input
    pub fn handle_keyboard(&mut self, event: &KeyEvent) {
        let pressed = event.state == ElementState::Pressed;

        if let PhysicalKey::Code(keycode) = event.physical_key {
            match keycode {
                KeyCode::KeyW => self.move_forward = pressed,
                KeyCode::KeyS => self.move_backward = pressed,
                KeyCode::KeyA => self.move_left = pressed,
                KeyCode::KeyD => self.move_right = pressed,
                KeyCode::Space => self.move_up = pressed,
                KeyCode::ControlLeft | KeyCode::ControlRight => self.move_down = pressed,
                KeyCode::ShiftLeft | KeyCode::ShiftRight => self.move_fast = pressed,
                _ => {}
            }
        }
    }

    /// Handle mouse movement
    pub fn handle_mouse_move(&mut self, delta_x: f64, delta_y: f64) {
        self.yaw += delta_x as f32 * self.sensitivity;
        self.pitch -= delta_y as f32 * self.sensitivity;

        // Clamp pitch to prevent gimbal lock
        self.pitch = self.pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        self.update_vectors();
    }

    /// Update camera vectors based on yaw and pitch
    fn update_vectors(&mut self) {
        self.forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        self.right = self.forward.cross(Vec3::Y).normalize();
        self.up = self.right.cross(self.forward).normalize();
    }

    /// Update camera position based on input
    pub fn update(&mut self, dt: f32) {
        let speed = if self.move_fast {
            self.speed * self.fast_multiplier
        } else {
            self.speed
        };

        let velocity = speed * dt;

        if self.move_forward {
            self.position += self.forward * velocity;
        }
        if self.move_backward {
            self.position -= self.forward * velocity;
        }
        if self.move_right {
            self.position += self.right * velocity;
        }
        if self.move_left {
            self.position -= self.right * velocity;
        }
        if self.move_up {
            self.position += Vec3::Y * velocity;
        }
        if self.move_down {
            self.position -= Vec3::Y * velocity;
        }
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.position + self.forward, self.up)
    }

    /// Get projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    /// Get view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Update aspect ratio
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
