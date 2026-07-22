/// Noise generation module
/// Provides stackable noise generators for procedural generation
use glam::{Vec2, Vec3};
use noise::{NoiseFn, Perlin, Simplex};

/// Noise generator trait for stackable composition
pub trait NoiseGenerator: Send + Sync {
    fn sample_2d(&self, point: Vec2) -> f32;
    fn sample_3d(&self, point: Vec3) -> f32;
}

/// Perlin noise generator
pub struct PerlinNoise {
    noise: Perlin,
    scale: f32,
    amplitude: f32,
}

impl PerlinNoise {
    pub fn new(seed: u32, scale: f32, amplitude: f32) -> Self {
        Self {
            noise: Perlin::new(seed),
            scale,
            amplitude,
        }
    }
}

impl NoiseGenerator for PerlinNoise {
    fn sample_2d(&self, point: Vec2) -> f32 {
        let p = point * self.scale;
        self.noise.get([p.x as f64, p.y as f64]) as f32 * self.amplitude
    }

    fn sample_3d(&self, point: Vec3) -> f32 {
        let p = point * self.scale;
        self.noise.get([p.x as f64, p.y as f64, p.z as f64]) as f32 * self.amplitude
    }
}

/// Simplex noise generator
pub struct SimplexNoise {
    noise: Simplex,
    scale: f32,
    amplitude: f32,
}

impl SimplexNoise {
    pub fn new(seed: u32, scale: f32, amplitude: f32) -> Self {
        Self {
            noise: Simplex::new(seed),
            scale,
            amplitude,
        }
    }
}

impl NoiseGenerator for SimplexNoise {
    fn sample_2d(&self, point: Vec2) -> f32 {
        let p = point * self.scale;
        self.noise.get([p.x as f64, p.y as f64]) as f32 * self.amplitude
    }

    fn sample_3d(&self, point: Vec3) -> f32 {
        let p = point * self.scale;
        self.noise.get([p.x as f64, p.y as f64, p.z as f64]) as f32 * self.amplitude
    }
}

/// Fractal Brownian Motion (FBM) noise
pub struct FbmNoise {
    base: Box<dyn NoiseGenerator>,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
}

impl FbmNoise {
    pub fn new(base: Box<dyn NoiseGenerator>, octaves: u32, lacunarity: f32, gain: f32) -> Self {
        Self {
            base,
            octaves,
            lacunarity,
            gain,
        }
    }
}

impl NoiseGenerator for FbmNoise {
    fn sample_2d(&self, point: Vec2) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            value += self.base.sample_2d(point * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.gain;
            frequency *= self.lacunarity;
        }

        value / max_value
    }

    fn sample_3d(&self, point: Vec3) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            value += self.base.sample_3d(point * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.gain;
            frequency *= self.lacunarity;
        }

        value / max_value
    }
}

/// Domain warping noise
pub struct DomainWarpNoise {
    base: Box<dyn NoiseGenerator>,
    warp_x: Box<dyn NoiseGenerator>,
    warp_y: Box<dyn NoiseGenerator>,
    warp_z: Box<dyn NoiseGenerator>,
    warp_strength: f32,
}

impl DomainWarpNoise {
    pub fn new(
        base: Box<dyn NoiseGenerator>,
        warp_x: Box<dyn NoiseGenerator>,
        warp_y: Box<dyn NoiseGenerator>,
        warp_z: Box<dyn NoiseGenerator>,
        warp_strength: f32,
    ) -> Self {
        Self {
            base,
            warp_x,
            warp_y,
            warp_z,
            warp_strength,
        }
    }
}

impl NoiseGenerator for DomainWarpNoise {
    fn sample_2d(&self, point: Vec2) -> f32 {
        let offset_x = self.warp_x.sample_2d(point) * self.warp_strength;
        let offset_y = self.warp_y.sample_2d(point) * self.warp_strength;
        let warped = point + Vec2::new(offset_x, offset_y);
        self.base.sample_2d(warped)
    }

    fn sample_3d(&self, point: Vec3) -> f32 {
        let offset_x = self.warp_x.sample_3d(point) * self.warp_strength;
        let offset_y = self.warp_y.sample_3d(point) * self.warp_strength;
        let offset_z = self.warp_z.sample_3d(point) * self.warp_strength;
        let warped = point + Vec3::new(offset_x, offset_y, offset_z);
        self.base.sample_3d(warped)
    }
}

/// Curl noise for fluid-like motion
pub struct CurlNoise {
    base: Box<dyn NoiseGenerator>,
    epsilon: f32,
}

impl CurlNoise {
    pub fn new(base: Box<dyn NoiseGenerator>, epsilon: f32) -> Self {
        Self { base, epsilon }
    }

    pub fn sample_curl_3d(&self, point: Vec3) -> Vec3 {
        let eps = self.epsilon;

        let dx_p = self.base.sample_3d(point + Vec3::new(eps, 0.0, 0.0));
        let dx_n = self.base.sample_3d(point - Vec3::new(eps, 0.0, 0.0));

        let dy_p = self.base.sample_3d(point + Vec3::new(0.0, eps, 0.0));
        let dy_n = self.base.sample_3d(point - Vec3::new(0.0, eps, 0.0));

        let dz_p = self.base.sample_3d(point + Vec3::new(0.0, 0.0, eps));
        let dz_n = self.base.sample_3d(point - Vec3::new(0.0, 0.0, eps));

        Vec3::new(
            (dz_p - dz_n) - (dy_p - dy_n),
            (dx_p - dx_n) - (dz_p - dz_n),
            (dy_p - dy_n) - (dx_p - dx_n),
        ) / (2.0 * eps)
    }
}

impl NoiseGenerator for CurlNoise {
    fn sample_2d(&self, point: Vec2) -> f32 {
        self.base.sample_2d(point)
    }

    fn sample_3d(&self, point: Vec3) -> f32 {
        self.sample_curl_3d(point).length()
    }
}
