use anyhow::Result;
use clap::{Parser, Subcommand};

#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod curve;
mod daemon;
mod device;
mod pipeline;
#[allow(dead_code)]
mod uinput;

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
    /// Print the default TOML config (pipe to a file to use as a starting point)
    Init,
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
        Some(Commands::Init) => print_default_config()?,
        Some(Commands::Profile { command }) => match command {
            ProfileCommands::Set { name } => {
                println!("Setting profile to '{name}' — runtime profile switching arrives in v0.2");
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
    println!("maccel v0.1.0-dev");
    println!("Default curve: {:?}", curve::Curve::macos());
    println!("Daemon: not running (run `maccel run` to start)");
    Ok(())
}

fn print_default_config() -> Result<()> {
    print!("{}", config::Config::defaults_as_toml()?);
    Ok(())
}
