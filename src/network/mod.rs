/// Network simulation layer
/// Generates procedural parameters from real network traffic or simulation
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[cfg(feature = "network-capture")]
use pcap::{Capture, Device};

/// Network parameters that drive procedural generation
#[derive(Debug, Clone, Copy)]
pub struct NetworkParams {
    pub density: f32,      // 0.0 - 1.0: controls point density
    pub chaos: f32,        // 0.0 - 1.0: increases distortion
    pub flow: f32,         // -1.0 - 1.0: bends space direction
    pub entropy: f32,      // 0.0 - 1.0: randomness factor
    pub packet_rate: f32,  // 0.0 - 1.0: activity level
    pub energy: f32,       // 0.0 - 1.0: brightness multiplier
    pub frequency: f32,    // 0.0 - 10.0: oscillation speed
    pub curvature: f32,    // -1.0 - 1.0: geometric distortion

    // Traffic stats for display
    pub packets_per_sec: f32,
    pub bytes_per_sec: f32,
    pub tcp_count: usize,
    pub udp_count: usize,
}

impl Default for NetworkParams {
    fn default() -> Self {
        Self {
            density: 0.5,
            chaos: 0.3,
            flow: 0.0,
            entropy: 0.4,
            packet_rate: 0.5,
            energy: 0.7,
            frequency: 1.0,
            curvature: 0.0,
            packets_per_sec: 0.0,
            bytes_per_sec: 0.0,
            tcp_count: 0,
            udp_count: 0,
        }
    }
}

/// Network traffic analyzer
struct TrafficStats {
    packet_count: usize,
    total_bytes: usize,
    tcp_count: usize,
    udp_count: usize,
    avg_packet_size: f32,

    // Smoothed values for gradual transitions
    smoothed_density: f32,
    smoothed_chaos: f32,
    smoothed_flow: f32,
    smoothed_entropy: f32,
    smoothed_packet_rate: f32,
    smoothed_energy: f32,
    smoothed_frequency: f32,
    smoothed_curvature: f32,

    // For display
    pub packets_per_sec: f32,
    pub bytes_per_sec: f32,
}

impl TrafficStats {
    fn new() -> Self {
        Self {
            packet_count: 0,
            total_bytes: 0,
            tcp_count: 0,
            udp_count: 0,
            avg_packet_size: 0.0,
            smoothed_density: 0.0,
            smoothed_chaos: 0.0,
            smoothed_flow: 0.0,
            smoothed_entropy: 0.0,
            smoothed_packet_rate: 0.0,
            smoothed_energy: 0.0,
            smoothed_frequency: 0.0,
            smoothed_curvature: 0.0,
            packets_per_sec: 0.0,
            bytes_per_sec: 0.0,
        }
    }

    fn update(&mut self, packet_size: usize, is_tcp: bool) {
        self.packet_count += 1;
        self.total_bytes += packet_size;
        if is_tcp {
            self.tcp_count += 1;
        } else {
            self.udp_count += 1;
        }
        self.avg_packet_size = self.total_bytes as f32 / self.packet_count as f32;
    }

    fn to_params(&mut self) -> NetworkParams {
        // Update display stats (for 100ms window)
        self.packets_per_sec = self.packet_count as f32 * 10.0; // 100ms * 10 = 1 sec
        self.bytes_per_sec = self.total_bytes as f32 * 10.0;

        // If no traffic, gradually fade to empty space
        if self.packet_count == 0 {
            let alpha = 0.05; // Smooth fade out
            self.smoothed_density *= 1.0 - alpha;
            self.smoothed_chaos *= 1.0 - alpha;
            self.smoothed_flow *= 1.0 - alpha;
            self.smoothed_entropy *= 1.0 - alpha;
            self.smoothed_packet_rate *= 1.0 - alpha;
            self.smoothed_energy *= 1.0 - alpha;
            self.smoothed_frequency *= 1.0 - alpha;
            self.smoothed_curvature *= 1.0 - alpha;

            return NetworkParams {
                density: self.smoothed_density,
                chaos: self.smoothed_chaos,
                flow: self.smoothed_flow,
                entropy: self.smoothed_entropy,
                packet_rate: self.smoothed_packet_rate,
                energy: self.smoothed_energy,
                frequency: self.smoothed_frequency,
                curvature: self.smoothed_curvature,
                packets_per_sec: self.packets_per_sec,
                bytes_per_sec: self.bytes_per_sec,
                tcp_count: self.tcp_count,
                udp_count: self.udp_count,
            };
        }

        // Calculate target values from traffic
        let packet_rate = (self.packet_count as f32 / 100.0).min(1.0);
        let density = (self.avg_packet_size / 1500.0).clamp(0.2, 0.8);
        let tcp_ratio = self.tcp_count as f32 / self.packet_count as f32;

        let target_density = density;
        let target_chaos = (1.0 - tcp_ratio).clamp(0.1, 0.8);
        let target_flow = (tcp_ratio - 0.5) * 2.0;
        let target_entropy = packet_rate;
        let target_packet_rate = packet_rate;
        let target_energy = (self.total_bytes as f32 / 100000.0).clamp(0.4, 0.9);
        let target_frequency = 1.0 + packet_rate;
        let target_curvature = (tcp_ratio - 0.5).clamp(-0.7, 0.7);

        // Smooth interpolation (lerp) with alpha = 0.1 for gradual changes
        let alpha = 0.1;
        self.smoothed_density = self.smoothed_density * (1.0 - alpha) + target_density * alpha;
        self.smoothed_chaos = self.smoothed_chaos * (1.0 - alpha) + target_chaos * alpha;
        self.smoothed_flow = self.smoothed_flow * (1.0 - alpha) + target_flow * alpha;
        self.smoothed_entropy = self.smoothed_entropy * (1.0 - alpha) + target_entropy * alpha;
        self.smoothed_packet_rate = self.smoothed_packet_rate * (1.0 - alpha) + target_packet_rate * alpha;
        self.smoothed_energy = self.smoothed_energy * (1.0 - alpha) + target_energy * alpha;
        self.smoothed_frequency = self.smoothed_frequency * (1.0 - alpha) + target_frequency * alpha;
        self.smoothed_curvature = self.smoothed_curvature * (1.0 - alpha) + target_curvature * alpha;

        NetworkParams {
            density: self.smoothed_density,
            chaos: self.smoothed_chaos,
            flow: self.smoothed_flow,
            entropy: self.smoothed_entropy,
            packet_rate: self.smoothed_packet_rate,
            energy: self.smoothed_energy,
            frequency: self.smoothed_frequency,
            curvature: self.smoothed_curvature,
            packets_per_sec: self.packets_per_sec,
            bytes_per_sec: self.bytes_per_sec,
            tcp_count: self.tcp_count,
            udp_count: self.udp_count,
        }
    }

    fn reset(&mut self) {
        self.packet_count = 0;
        self.total_bytes = 0;
        self.tcp_count = 0;
        self.udp_count = 0;
    }
}

/// Network traffic simulator/capturer
pub struct NetworkSimulator {
    simulation_sender: Sender<NetworkParams>,
    capture_sender: Option<Sender<NetworkParams>>,
    _simulation_handle: thread::JoinHandle<()>,
    _capture_handle: Option<thread::JoinHandle<()>>,
}

impl NetworkSimulator {
    /// Create new network capturer with both simulation and real traffic
    pub fn new() -> (Self, Receiver<NetworkParams>, Receiver<NetworkParams>) {
        // Simulation channel
        let (sim_tx, sim_rx) = channel();
        let sim_sender = sim_tx.clone();

        // Real traffic channel
        let (cap_tx, cap_rx) = channel();
        let cap_sender_clone = cap_tx.clone();

        // Spawn simulation thread
        let sim_handle = thread::spawn(move || {
            Self::run_simulation(sim_tx);
        });

        // Try to spawn capture thread
        let (capture_handle, capture_sender) = {
            #[cfg(feature = "network-capture")]
            {
                if let Some(device) = Device::lookup().ok().flatten() {
                    println!("Capturing traffic on device: {}", device.name);
                    let handle = thread::spawn(move || {
                        Self::run_capture(cap_tx, device);
                    });
                    (Some(handle), Some(cap_sender_clone))
                } else {
                    println!("Failed to open capture device, using simulation for network traffic mode");
                    // Start simulation as fallback for capture channel
                    let handle = thread::spawn(move || {
                        Self::run_simulation(cap_tx);
                    });
                    (Some(handle), Some(cap_sender_clone))
                }
            }

            #[cfg(not(feature = "network-capture"))]
            {
                println!("Network capture disabled (pcap feature not enabled), using simulation for network traffic mode");
                // Start simulation as fallback for capture channel
                let handle = thread::spawn(move || {
                    Self::run_simulation(cap_tx);
                });
                (Some(handle), Some(cap_sender_clone))
            }
        };

        (
            Self {
                simulation_sender: sim_sender,
                capture_sender,
                _simulation_handle: sim_handle,
                _capture_handle: capture_handle,
            },
            sim_rx,
            cap_rx,
        )
    }

    /// Capture real network traffic
    #[cfg(feature = "network-capture")]
    fn run_capture(tx: Sender<NetworkParams>, device: Device) {
        let mut cap = match Capture::from_device(device) {
            Ok(cap) => match cap.promisc(true).timeout(100).open() {
                Ok(cap) => cap,
                Err(e) => {
                    println!("Failed to open capture: {}, falling back to simulation", e);
                    Self::run_simulation(tx);
                    return;
                }
            },
            Err(e) => {
                println!("Failed to create capture: {}, falling back to simulation", e);
                Self::run_simulation(tx);
                return;
            }
        };

        let mut stats = TrafficStats::new();
        let mut last_update = std::time::Instant::now();

        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    let packet_size = packet.data.len();

                    // Simple protocol detection (check IP header)
                    let is_tcp = if packet.data.len() > 23 {
                        packet.data[23] == 6 // TCP protocol number
                    } else {
                        false
                    };

                    stats.update(packet_size, is_tcp);

                    // Update params every 100ms
                    if last_update.elapsed() > Duration::from_millis(100) {
                        let params = stats.to_params();
                        if tx.send(params).is_err() {
                            break;
                        }
                        stats.reset();
                        last_update = std::time::Instant::now();
                    }
                }
                Err(_) => {
                    // Timeout or error, send current stats
                    if last_update.elapsed() > Duration::from_millis(100) {
                        let params = stats.to_params();
                        if tx.send(params).is_err() {
                            break;
                        }
                        stats.reset();
                        last_update = std::time::Instant::now();
                    }
                }
            }
        }
    }

    /// Fallback simulation mode
    fn run_simulation(tx: Sender<NetworkParams>) {
        let mut time = 0.0_f32;

        loop {
            thread::sleep(Duration::from_millis(16)); // ~60 FPS update rate for smoother changes
            time += 0.016;

            // Smoother, slower changes with multiple frequencies
            let t1 = time * 0.5;
            let t2 = time * 0.3;
            let t3 = time * 0.7;
            let t4 = time * 0.4;

            // Smooth noise layers
            let noise1 = (t1 * 2.3).sin() * (t2 * 3.7).cos();
            let noise2 = (t3 * 1.9).sin() * (t4 * 2.1).sin();
            let burst = ((time * 0.2).sin() * 5.0).sin() * 0.5;

            // Simulate traffic stats
            let sim_packets = (50.0 + 200.0 * (t1 * 0.8).sin().abs()) as usize;
            let sim_bytes = (5000.0 + 20000.0 * (t2 * 0.6).sin().abs()) as usize;
            let sim_tcp = (sim_packets as f32 * (0.5 + 0.3 * (t3 * 0.5).sin())) as usize;
            let sim_udp = sim_packets - sim_tcp;

            let params = NetworkParams {
                density: (0.4 + 0.3 * (t1).sin() + 0.1 * noise1).clamp(0.2, 0.8),
                chaos: (0.3 + 0.4 * (t2).sin() + 0.2 * burst).clamp(0.1, 0.8),
                flow: ((t3).sin() * 0.5 + 0.3 * noise2).clamp(-0.8, 0.8),
                entropy: (0.4 + 0.3 * (t4).cos() + 0.2 * (t1 * 1.5).sin()).clamp(0.2, 0.8),
                packet_rate: (0.5 + 0.3 * (t1 * 1.2).sin() + 0.15 * burst).clamp(0.3, 0.9),
                energy: (0.6 + 0.3 * (t2 * 0.8).sin() + 0.1 * noise1).clamp(0.4, 0.9),
                frequency: (1.5 + 1.0 * (t3 * 0.9).sin() + 0.3 * noise2).clamp(0.8, 2.5),
                curvature: ((t4 * 0.6).sin() + 0.3 * noise1).clamp(-0.7, 0.7),
                packets_per_sec: sim_packets as f32,
                bytes_per_sec: sim_bytes as f32,
                tcp_count: sim_tcp,
                udp_count: sim_udp,
            };

            if tx.send(params).is_err() {
                break;
            }
        }
    }
}
