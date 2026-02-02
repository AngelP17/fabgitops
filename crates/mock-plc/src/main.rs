mod chaos;
mod server;

use crate::chaos::{ChaosConfig, ChaosEngine};
use crate::server::{start_server, PLCState};
use clap::Parser;
use std::sync::{Arc, Mutex};
use tracing::{info, Level};

#[derive(Parser, Debug)]
#[command(name = "mock-plc")]
#[command(about = "Mock PLC server with chaos mode for FabGitOps testing")]
#[command(version = "0.1.0")]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0")]
    bind: String,

    #[arg(short, long, default_value = "5502")]
    port: u16,

    #[arg(short, long, default_value = "2500")]
    value: u16,

    #[arg(short, long, default_value = "4001")]
    register: u16,

    /// Enable chaos mode (random drift)
    #[arg(long)]
    chaos: bool,

    /// Chaos drift interval in seconds
    #[arg(long, default_value = "10")]
    chaos_interval: u64,

    /// Maximum drift amount
    #[arg(long, default_value = "500")]
    max_drift: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args = Args::parse();

    info!("╔══════════════════════════════════════╗");
    info!("║     FabGitOps Mock PLC Server        ║");
    info!("╚══════════════════════════════════════╝");
    info!("");
    info!("Configuration:");
    info!("  Bind Address: {}:{}", args.bind, args.port);
    info!("  Register: {}", args.register);
    info!("  Initial Value: {}", args.value);
    info!(
        "  Chaos Mode: {}",
        if args.chaos { "ENABLED" } else { "disabled" }
    );

    if args.chaos {
        info!("  Chaos Interval: {}s", args.chaos_interval);
        info!("  Max Drift: {}", args.max_drift);
    }

    info!("");

    let state = Arc::new(Mutex::new(PLCState::new(args.value, args.register)));

    // Start chaos engine if enabled
    let _chaos = if args.chaos {
        let register_value = Arc::new(std::sync::Mutex::new(args.value));
        let chaos = ChaosEngine::new(ChaosConfig {
            enabled: true,
            interval_secs: args.chaos_interval,
            max_drift: args.max_drift,
        });
        chaos.spawn(register_value.clone());
        Some(chaos)
    } else {
        None
    };

    start_server(&args.bind, args.port, state).await
}
