/// Chunk management system for infinite world generation
use glam::Vec3;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use crate::network::NetworkParams;
use crate::noise::NoiseGenerator;

/// Chunk coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_world_pos(pos: Vec3, chunk_size: f32) -> Self {
        Self {
            x: (pos.x / chunk_size).floor() as i32,
            z: (pos.z / chunk_size).floor() as i32,
        }
    }

    pub fn to_world_pos(&self, chunk_size: f32) -> Vec3 {
        Vec3::new(self.x as f32 * chunk_size, 0.0, self.z as f32 * chunk_size)
    }
}

/// Point data for GPU rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub brightness: f32,
    pub size: f32,
}

/// A chunk of generated points
pub struct Chunk {
    pub coord: ChunkCoord,
    pub points: Vec<Point>,
}

impl Chunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            coord,
            points: Vec::new(),
        }
    }
}

/// Message for chunk generation thread
enum ChunkGenMessage {
    Generate {
        coord: ChunkCoord,
        params: NetworkParams,
    },
    Shutdown,
}

/// Chunk manager handles async chunk loading/unloading
pub struct ChunkManager {
    pub chunks: HashMap<ChunkCoord, Chunk>,
    loaded_chunks: HashSet<ChunkCoord>,
    chunk_size: f32,
    render_distance: f32,

    // Generation thread communication
    gen_sender: Sender<ChunkGenMessage>,
    gen_receiver: Receiver<Chunk>,
    pending_chunks: HashSet<ChunkCoord>,
}

impl ChunkManager {
    pub fn new(chunk_size: f32, render_distance: f32, noise_gen: Arc<Box<dyn NoiseGenerator>>) -> Self {
        let (gen_tx, gen_rx_internal) = channel();
        let (result_tx, result_rx) = channel();

        // Spawn generation thread
        thread::spawn(move || {
            Self::generation_thread(gen_rx_internal, result_tx, chunk_size, noise_gen);
        });

        Self {
            chunks: HashMap::new(),
            loaded_chunks: HashSet::new(),
            chunk_size,
            render_distance,
            gen_sender: gen_tx,
            gen_receiver: result_rx,
            pending_chunks: HashSet::new(),
        }
    }

    /// Background generation thread
    fn generation_thread(
        receiver: Receiver<ChunkGenMessage>,
        sender: Sender<Chunk>,
        chunk_size: f32,
        noise_gen: Arc<Box<dyn NoiseGenerator>>,
    ) {
        while let Ok(msg) = receiver.recv() {
            match msg {
                ChunkGenMessage::Generate { coord, params } => {
                    let chunk = Self::generate_chunk(coord, chunk_size, &noise_gen, params);
                    if sender.send(chunk).is_err() {
                        break;
                    }
                }
                ChunkGenMessage::Shutdown => break,
            }
        }
    }

    /// Generate a single chunk
    fn generate_chunk(
        coord: ChunkCoord,
        chunk_size: f32,
        noise_gen: &Box<dyn NoiseGenerator>,
        params: NetworkParams,
    ) -> Chunk {
        let mut chunk = Chunk::new(coord);
        let base_pos = coord.to_world_pos(chunk_size);

        // Floor lines - horizontal lines extending into depth
        let line_spacing = 3.0; // Spacing between floor lines
        let num_lines = (chunk_size / line_spacing).ceil() as usize;

        // Points per line - controlled by network data
        let points_per_line = (100.0 * params.density * (1.0 + params.packet_rate)).max(10.0) as usize;
        let point_spacing = chunk_size / points_per_line as f32;

        for line_idx in 0..num_lines {
            // Lines run along X axis (left-right from camera view)
            let local_z = line_idx as f32 * line_spacing;
            let world_z = base_pos.z + local_z;

            for point_idx in 0..points_per_line {
                let local_x = point_idx as f32 * point_spacing;
                let world_x = base_pos.x + local_x;

                // Dynamic waves based on network parameters
                // Add minimum frequency to ensure points always animate
                let min_frequency = 0.5;
                let min_energy = 0.3;
                let min_entropy = 0.2;
                let wave_scale = 0.03 * (params.frequency.max(min_frequency));
                let noise_val = noise_gen.sample_3d(Vec3::new(
                    world_x * wave_scale,
                    params.entropy.max(min_entropy) * 10.0,
                    world_z * wave_scale,
                ));

                // Multi-layer waves for breathing effect
                let wave1 = noise_gen.sample_3d(Vec3::new(
                    world_x * 0.02,
                    params.energy.max(min_energy) * 3.0,
                    world_z * 0.02,
                ));

                let wave2 = noise_gen.sample_3d(Vec3::new(
                    world_x * 0.08,
                    params.curvature * 7.0,
                    world_z * 0.08,
                ));

                // Combine waves - height changes with network data
                let base_height = noise_val * 15.0 * params.chaos.max(0.2);
                let wave_height = wave1 * 8.0 * params.energy.max(min_energy) + wave2 * 4.0 * params.entropy.max(min_entropy);
                let y = base_height + wave_height;

                let position = Vec3::new(world_x, y, world_z);

                // Pure RED color
                let color = Vec3::new(1.0, 0.0, 0.0);

                // Brightness varies with height for depth
                let brightness = (1.0 + y * 0.02).clamp(0.5, 1.5);
                let size = 1.0 + params.packet_rate * 0.5;

                chunk.points.push(Point {
                    position: position.to_array(),
                    color: color.to_array(),
                    brightness,
                    size,
                });
            }
        }

        // STRUCTURES: Giant wave wall - single massive structure across entire world
        // Only spawn when there's significant traffic
        if params.energy > 0.6 && params.flow.abs() > 0.5 {
            // Calculate if this chunk is near the wave position
            let wave_x = params.flow * 200.0; // Wave position moves with flow parameter
            let chunk_center_x = base_pos.x + chunk_size * 0.5;
            let distance_to_wave = (chunk_center_x - wave_x).abs();

            // Only generate wave points if chunk is near the wave
            if distance_to_wave < chunk_size * 2.0 {
                let wall_thickness = 20.0;

                // Generate a massive flat wave spanning the Z axis
                let z_points = (chunk_size / 2.0) as usize;
                let wall_height = 80.0 * params.energy;
                let y_points = (wall_height / 1.0) as usize;

                for z_idx in 0..z_points {
                    let z = base_pos.z + (z_idx as f32 * 2.0);

                    for y_idx in 0..y_points {
                        let y = y_idx as f32 * 1.0;

                        // Add some wave motion to the wall
                        let wave_offset = (z * 0.05 + y * 0.03).sin() * 3.0 * params.chaos;
                        let x = wave_x + wave_offset;

                        // Only add point if it's within this chunk's bounds
                        if x >= base_pos.x && x < base_pos.x + chunk_size {
                            // Brightness varies with height for dramatic effect
                            let height_factor = y / wall_height;
                            let brightness = 1.0 + height_factor * 0.8;

                            chunk.points.push(Point {
                                position: [x, y, z],
                                color: [1.0, 0.0, 0.0],
                                brightness,
                                size: 1.5 + params.packet_rate * 0.5,
                            });
                        }
                    }
                }
            }
        }

        // FIREWALL BARRIERS: Pulsing protective layers
        // Appear when entropy is high (indicating irregular traffic)
        if params.entropy > 0.6 && params.chaos > 0.5 {
            // Multiple horizontal barrier layers at different heights
            let barrier_positions = [15.0, 35.0, 55.0]; // Three layers

            for &barrier_y in &barrier_positions {
                // Create a grid pattern for the barrier
                let grid_spacing = 5.0;
                let grid_x = (chunk_size / grid_spacing) as usize;
                let grid_z = (chunk_size / grid_spacing) as usize;

                for gx in 0..grid_x {
                    for gz in 0..grid_z {
                        let x = base_pos.x + (gx as f32 * grid_spacing);
                        let z = base_pos.z + (gz as f32 * grid_spacing);

                        // Hexagonal pattern - skip some points
                        if (gx + gz) % 2 == 0 {
                            // Pulsing effect based on position and entropy
                            let pulse = ((x * 0.1 + z * 0.1 + params.entropy * 10.0).sin() * 0.5 + 0.5);
                            let brightness = 0.8 + pulse * 0.7;

                            // Barrier thickness - small vertical variation
                            let y_offset = pulse * 2.0;

                            chunk.points.push(Point {
                                position: [x, barrier_y + y_offset, z],
                                color: [1.0, 0.0, 0.0],
                                brightness,
                                size: 2.0 + pulse * 0.5,
                            });
                        }
                    }
                }
            }
        }

        // VOLUMETRIC COLUMNS - thick cylindrical structures
        if params.energy > 0.7 && params.chaos > 0.75 {
            // Very few columns, based on chunk position
            let column_seed = (coord.x * 73 + coord.z * 37) as f32;
            if (column_seed % 7.0) < 1.0 {
                let col_center_x = base_pos.x + chunk_size * 0.5;
                let col_center_z = base_pos.z + chunk_size * 0.5;

                let column_height = 50.0 * params.packet_rate;
                let column_radius = 3.0; // Radius of the column
                let height_segments = (column_height * 2.0) as usize;

                // Generate circular cross-sections at each height
                for h in 0..height_segments {
                    let y = h as f32 * 0.5;

                    // Create circular ring at this height
                    let ring_points = 12; // 12 points per ring for smooth circle
                    for ring_idx in 0..ring_points {
                        let angle = (ring_idx as f32 / ring_points as f32) * std::f32::consts::PI * 2.0;

                        let offset_x = angle.cos() * column_radius;
                        let offset_z = angle.sin() * column_radius;

                        chunk.points.push(Point {
                            position: [col_center_x + offset_x, y, col_center_z + offset_z],
                            color: [1.0, 0.0, 0.0],
                            brightness: 1.5,
                            size: 1.8,
                        });
                    }

                    // Fill interior with some points for solidity
                    if h % 3 == 0 {
                        let inner_radius = column_radius * 0.6;
                        for ring_idx in 0..8 {
                            let angle = (ring_idx as f32 / 8.0) * std::f32::consts::PI * 2.0;

                            let offset_x = angle.cos() * inner_radius;
                            let offset_z = angle.sin() * inner_radius;

                            chunk.points.push(Point {
                                position: [col_center_x + offset_x, y, col_center_z + offset_z],
                                color: [1.0, 0.0, 0.0],
                                brightness: 1.3,
                                size: 1.5,
                            });
                        }
                    }
                }
            }
        }

        chunk
    }

    /// Update chunks based on camera position
    pub fn update(&mut self, camera_pos: Vec3, params: NetworkParams) {
        let camera_chunk = ChunkCoord::from_world_pos(camera_pos, self.chunk_size);

        // Determine which chunks should be loaded
        let load_radius = (self.render_distance / self.chunk_size).ceil() as i32;
        let mut should_be_loaded = HashSet::new();

        for dx in -load_radius..=load_radius {
            for dz in -load_radius..=load_radius {
                let coord = ChunkCoord::new(camera_chunk.x + dx, camera_chunk.z + dz);
                let chunk_center = coord.to_world_pos(self.chunk_size) + Vec3::new(self.chunk_size * 0.5, 0.0, self.chunk_size * 0.5);
                let distance = camera_pos.distance(chunk_center);

                if distance < self.render_distance {
                    should_be_loaded.insert(coord);
                }
            }
        }

        // Unload distant chunks only
        self.chunks.retain(|coord, _| should_be_loaded.contains(coord));
        self.loaded_chunks.retain(|coord| should_be_loaded.contains(coord));
        self.pending_chunks.retain(|coord| should_be_loaded.contains(coord));

        // Request regeneration for ALL visible chunks to get live updates
        for coord in &should_be_loaded {
            // Always regenerate for breathing effect
            self.loaded_chunks.remove(coord);
            if !self.pending_chunks.contains(coord) {
                let _ = self.gen_sender.send(ChunkGenMessage::Generate {
                    coord: *coord,
                    params,
                });
                self.pending_chunks.insert(*coord);
            }
        }

        // Collect generated chunks
        while let Ok(chunk) = self.gen_receiver.try_recv() {
            self.pending_chunks.remove(&chunk.coord);
            self.loaded_chunks.insert(chunk.coord);
            self.chunks.insert(chunk.coord, chunk);
        }
    }

    /// Get all visible points for rendering
    pub fn get_visible_points(&self) -> Vec<Point> {
        let mut points = Vec::new();
        for chunk in self.chunks.values() {
            points.extend_from_slice(&chunk.points);
        }
        points
    }

    /// Get chunk count
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get total point count
    pub fn point_count(&self) -> usize {
        self.chunks.values().map(|c| c.points.len()).sum()
    }
}

impl Drop for ChunkManager {
    fn drop(&mut self) {
        let _ = self.gen_sender.send(ChunkGenMessage::Shutdown);
    }
}
