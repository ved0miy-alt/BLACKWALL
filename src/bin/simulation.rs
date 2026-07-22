// Cyberspace Engine - Simulation Mode
// Uses procedural network simulation

mod app;
mod camera;
mod chunk;
mod config;
mod debug;
mod math;
mod network;
mod noise;
mod renderer;

use anyhow::Result;
use config::EngineConfig;

fn main() -> Result<()> {
    let config = EngineConfig::default();

    println!("Starting engine...");
    println!("Mode: Simulation");

    app::run(config, true) // true = use simulation
}
