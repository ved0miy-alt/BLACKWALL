/// Mathematical utilities for procedural generation
use glam::{Vec2, Vec3};

/// Hash function for deterministic random generation
pub fn hash_float(n: f32) -> f32 {
    let x = n.sin() * 43758.5453123;
    x - x.floor()
}

/// 2D hash function
pub fn hash_vec2(p: Vec2) -> f32 {
    let p = Vec2::new(p.dot(Vec2::new(127.1, 311.7)), p.dot(Vec2::new(269.5, 183.3)));
    let x = p.x.sin() * 43758.5453;
    (x - x.floor()) * 2.0 - 1.0
}

/// 3D hash function
pub fn hash_vec3(p: Vec3) -> f32 {
    let p = Vec3::new(
        p.dot(Vec3::new(127.1, 311.7, 74.7)),
        p.dot(Vec3::new(269.5, 183.3, 246.1)),
        p.dot(Vec3::new(113.5, 271.9, 124.6)),
    );
    let x = p.x.sin() * 43758.5453;
    (x - x.floor()) * 2.0 - 1.0
}

/// Smooth interpolation (smoothstep)
pub fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Vector lerp
pub fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}

/// Clamp value between min and max
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

/// Remap value from one range to another
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let normalized = (value - from_min) / (from_max - from_min);
    to_min + normalized * (to_max - to_min)
}
