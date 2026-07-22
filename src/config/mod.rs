/// Engine configuration parameters

/// Rendering configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub target_fps: u32,
    pub max_points_per_chunk: usize,
    pub point_base_size: f32,
    pub fog_density: f32,
    pub bloom_intensity: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            max_points_per_chunk: 100_000,
            point_base_size: 2.0,
            fog_density: 0.01,
            bloom_intensity: 0.5,
        }
    }
}

/// World generation configuration
#[derive(Debug, Clone)]
pub struct WorldConfig {
    pub chunk_size: f32,
    pub render_distance: f32,
    pub generation_distance: f32,
    pub floor_y: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            chunk_size: 100.0,
            render_distance: 300.0,
            generation_distance: 400.0,
            floor_y: 0.0,
        }
    }
}

/// Camera configuration
#[derive(Debug, Clone)]
pub struct CameraConfig {
    pub movement_speed: f32,
    pub fast_speed_multiplier: f32,
    pub mouse_sensitivity: f32,
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            movement_speed: 10.0,
            fast_speed_multiplier: 3.0,
            mouse_sensitivity: 0.002,
            fov: 70.0_f32.to_radians(),
            near_plane: 1.0,      // Было 0.1 - слишком близко
            far_plane: 10000.0,   // Было 1000.0 - увеличил для дальних расстояний
        }
    }
}

/// Complete engine configuration
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub render: RenderConfig,
    pub world: WorldConfig,
    pub camera: CameraConfig,
}
