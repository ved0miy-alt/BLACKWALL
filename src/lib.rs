// C API for Cyberspace Engine
// Exports engine functionality for FFI integration with Unreal Engine

use std::ffi::c_void;
use std::sync::Arc;
use glam::Vec3;

mod camera;
mod chunk;
mod config;
mod math;
mod network;
mod noise;

use chunk::{ChunkManager, Point};
use network::{NetworkParams, NetworkSimulator};
use noise::{FbmNoise, NoiseGenerator, PerlinNoise};

// Opaque handle for the engine instance
pub struct CyberspaceEngine {
    chunk_manager: ChunkManager,
    network_rx: std::sync::mpsc::Receiver<NetworkParams>,
    current_params: NetworkParams,
    camera_position: Vec3,
}

// C-compatible point structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPoint {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub brightness: f32,
    pub size: f32,
}

impl From<Point> for CPoint {
    fn from(p: Point) -> Self {
        Self {
            position: p.position,
            color: p.color,
            brightness: p.brightness,
            size: p.size,
        }
    }
}

// C-compatible network parameters
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CNetworkParams {
    pub density: f32,
    pub chaos: f32,
    pub flow: f32,
    pub entropy: f32,
    pub packet_rate: f32,
    pub energy: f32,
    pub frequency: f32,
    pub curvature: f32,
    pub packets_per_sec: f32,
    pub bytes_per_sec: f32,
    pub tcp_count: u32,
    pub udp_count: u32,
}

impl From<NetworkParams> for CNetworkParams {
    fn from(p: NetworkParams) -> Self {
        Self {
            density: p.density,
            chaos: p.chaos,
            flow: p.flow,
            entropy: p.entropy,
            packet_rate: p.packet_rate,
            energy: p.energy,
            frequency: p.frequency,
            curvature: p.curvature,
            packets_per_sec: p.packets_per_sec,
            bytes_per_sec: p.bytes_per_sec,
            tcp_count: p.tcp_count as u32,
            udp_count: p.udp_count as u32,
        }
    }
}

// Initialize the engine
// Returns opaque handle or null on failure
#[no_mangle]
pub extern "C" fn cyberspace_engine_create(
    chunk_size: f32,
    render_distance: f32,
    seed: u32,
) -> *mut c_void {
    // Create noise generator
    let base_noise = PerlinNoise::new(seed, 1.0, 1.0);
    let fbm = FbmNoise::new(Box::new(base_noise), 4, 2.0, 0.5);
    let noise_gen: Arc<Box<dyn NoiseGenerator>> = Arc::new(Box::new(fbm));

    // Create chunk manager
    let chunk_manager = ChunkManager::new(chunk_size, render_distance, noise_gen);

    // Start network simulator
    let (_simulator, simulation_rx, _capture_rx) = NetworkSimulator::new();

    let engine = Box::new(CyberspaceEngine {
        chunk_manager,
        network_rx: simulation_rx,
        current_params: NetworkParams::default(),
        camera_position: Vec3::ZERO,
    });

    Box::into_raw(engine) as *mut c_void
}

// Destroy the engine
#[no_mangle]
pub extern "C" fn cyberspace_engine_destroy(handle: *mut c_void) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle as *mut CyberspaceEngine);
        }
    }
}

// Update camera position (triggers chunk loading/unloading)
#[no_mangle]
pub extern "C" fn cyberspace_engine_update_camera(
    handle: *mut c_void,
    x: f32,
    y: f32,
    z: f32,
) {
    if handle.is_null() {
        return;
    }

    let engine = unsafe { &mut *(handle as *mut CyberspaceEngine) };
    engine.camera_position = Vec3::new(x, y, z);

    // Poll for latest network parameters
    while let Ok(params) = engine.network_rx.try_recv() {
        engine.current_params = params;
    }

    // Update chunks
    engine.chunk_manager.update(engine.camera_position, engine.current_params);
}

// Get visible points for rendering
// Returns number of points written to out_points buffer
// If out_points is null, returns the required buffer size
#[no_mangle]
pub extern "C" fn cyberspace_engine_get_points(
    handle: *mut c_void,
    out_points: *mut CPoint,
    max_points: u32,
) -> u32 {
    if handle.is_null() {
        return 0;
    }

    let engine = unsafe { &*(handle as *const CyberspaceEngine) };
    let points = engine.chunk_manager.get_visible_points();

    // Return required size if buffer is null
    if out_points.is_null() {
        return points.len() as u32;
    }

    // Copy points to output buffer
    let count = (points.len() as u32).min(max_points);
    unsafe {
        for i in 0..count as usize {
            *out_points.add(i) = points[i].into();
        }
    }

    count
}

// Get current network parameters
#[no_mangle]
pub extern "C" fn cyberspace_engine_get_network_params(
    handle: *mut c_void,
    out_params: *mut CNetworkParams,
) -> bool {
    if handle.is_null() || out_params.is_null() {
        return false;
    }

    let engine = unsafe { &*(handle as *const CyberspaceEngine) };
    unsafe {
        *out_params = engine.current_params.into();
    }

    true
}

// Get statistics
#[no_mangle]
pub extern "C" fn cyberspace_engine_get_chunk_count(handle: *mut c_void) -> u32 {
    if handle.is_null() {
        return 0;
    }

    let engine = unsafe { &*(handle as *const CyberspaceEngine) };
    engine.chunk_manager.chunk_count() as u32
}

#[no_mangle]
pub extern "C" fn cyberspace_engine_get_point_count(handle: *mut c_void) -> u32 {
    if handle.is_null() {
        return 0;
    }

    let engine = unsafe { &*(handle as *const CyberspaceEngine) };
    engine.chunk_manager.point_count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_create_destroy() {
        let handle = cyberspace_engine_create(100.0, 500.0, 42);
        assert!(!handle.is_null());
        cyberspace_engine_destroy(handle);
    }

    #[test]
    fn test_engine_update_camera() {
        let handle = cyberspace_engine_create(100.0, 500.0, 42);
        cyberspace_engine_update_camera(handle, 0.0, 0.0, 0.0);
        cyberspace_engine_destroy(handle);
    }

    #[test]
    fn test_engine_get_points() {
        let handle = cyberspace_engine_create(100.0, 500.0, 42);
        cyberspace_engine_update_camera(handle, 0.0, 10.0, 0.0);

        // Get required buffer size
        let count = cyberspace_engine_get_points(handle, std::ptr::null_mut(), 0);
        assert!(count > 0);

        // Allocate and get points
        let mut points = vec![CPoint {
            position: [0.0; 3],
            color: [0.0; 3],
            brightness: 0.0,
            size: 0.0,
        }; count as usize];

        let actual_count = cyberspace_engine_get_points(handle, points.as_mut_ptr(), count);
        assert_eq!(actual_count, count);

        cyberspace_engine_destroy(handle);
    }
}
