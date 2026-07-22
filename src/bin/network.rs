// Cyberspace Engine - Network Traffic Mode
// Uses real network traffic capture (or fallback simulation if unavailable)

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
    println!("Mode: Network Traffic");

    app::run(config, false) // false = use network capture
}
