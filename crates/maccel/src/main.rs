use anyhow::Result;
use clap::{Parser, Subcommand};

#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod curve;
mod daemon;
#[allow(dead_code)]
mod device;

#[derive(Parser)]
#[command(
    name = "maccel",
    version,
    about = "Mouse acceleration daemon — macOS-style pointer feel for Linux"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the acceleration daemon (default if no subcommand given)
    Run,
    /// Show current status
    Status,
    /// List detected mouse devices
    Devices,
    /// Manage curve presets
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// Set the active curve preset (macos, linear, ...)
    Set { name: String },
    /// List available presets
    List,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run) => daemon::run()?,
        Some(Commands::Status) => print_status()?,
        Some(Commands::Devices) => device::list_mice()?,
        Some(Commands::Profile { command }) => match command {
            ProfileCommands::Set { name } => {
                println!("Setting profile to '{name}' — not yet implemented (scaffold)");
            }
            ProfileCommands::List => {
                println!("Available presets:");
                println!("  macos   — rough approximation of Apple's curve (default)");
                println!("  linear  — pure linear, no acceleration");
            }
        },
        None => daemon::run()?,
    }

    Ok(())
}

fn print_status() -> Result<()> {
    println!("maccel v0.1.0-dev (scaffold)");
    println!("Default curve: {:?}", curve::Curve::macos());
    println!("Daemon: not running (no daemon implementation yet)");
    Ok(())
}
