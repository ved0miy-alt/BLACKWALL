<h1 align="center">BLACKWALL</h1>

<p align="center">
  Professional-grade procedural graphics engine for real time geometric visualisation of network traffic
</p>

<h1 align="center">BLACKWALL - experimental procedural engine based on network traffic packet data. Packet changes affect the variables of mathematical functions, thereby creating a three-dimensional geometric space</h1>
<p align="center">

  <img src="demo.gif" alt="Demo" width="700">

</p>
<p align="center">
  demo v0.1.1 alpha
</p>

## Features

- **Dual Mode Operation**
  - `cyberspace-sim`: Procedural simulation mode
  - `cyberspace-net`: Real network traffic capture mode (with fallback)
  
- **GPU-Accelerated Rendering**
  - wgpu-based rendering pipeline
  - Real-time point generation and animation
  - Bloom-free point rendering with fog effects
  
- **Dynamic Visualization**
  - Network parameters drive procedural generation
  - Continuous animation with minimum speed constraints
  - Static line directions for visual stability

## In future update:

1. Migration to Unreal Engine 5.8
2. Creation of online servers for cyberspace
   
## Technology Stack

- **Graphics**: wgpu (Vulkan/Metal/DX12)
- **Windowing**: winit
- **Math**: glam
- **UI**: egui
- **Language**: Rust (stable)

## Network Parameters

The engine generates the procedural world based on these parameters:

- **density**
- **chaos**
- **flow**
- **entropy**
- **packet_rate**
- **energy**
- **frequency**
- **curvature**

## Threading
```
Network Thread (100ms interval)
    │
    ├─→ Generate Parameters
    │
    └─→ Send via Channel
           │
           └─→ Main Thread (reads parameters)
                   │
                   └─→ Chunk Generation
```

## Building

### Basic Build (Simulation Only)

```bash
cargo build --release
```

### Build with Network Capture

```bash
cargo build --release --features network-capture
```

Note: Network capture requires pcap library and may need elevated permissions.

## Running

### Simulation Mode
```bash
./target/release/cyberspace-sim
```

### Network Traffic Mode
```bash
./target/release/cyberspace-net
```

## Dependencies

- Rust 1.70+
- wgpu 0.19
- winit 0.29
- egui 0.27
- pcap 1.1 (optional, for network capture)

