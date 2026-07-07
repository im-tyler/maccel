use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::device;
use crate::pipeline::Pipeline;

/// Run the maccel daemon.
///
/// Loads config, discovers pointer devices, spawns one pipeline thread per
/// managed device, and blocks until SIGINT/SIGTERM arrives (handled by ctrlc).
pub fn run() -> Result<()> {
    let config_path = std::env::var("MACCEL_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/etc/maccel/config.toml"));

    run_with_config(&config_path)
}

pub fn run_with_config(config_path: &Path) -> Result<()> {
    tracing::info!("maccel daemon starting (v0.1.0-dev)");
    tracing::info!(path = %config_path.display(), "loading config");

    let config = Config::load(config_path)?;
    tracing::info!(
        base_gain = config.curve.base_gain,
        threshold = config.curve.threshold,
        exponent = config.curve.exponent,
        "loaded curve",
    );

    let shutdown = Arc::new(AtomicBool::new(false));
    install_signal_handler(shutdown.clone())?;

    let managed = discover_managed_devices(&config)?;
    if managed.is_empty() {
        tracing::error!(
            "no pointer devices found. Is a mouse connected? Are you root or in the 'input' group?"
        );
        return Err(anyhow::anyhow!("no managed devices"));
    }

    tracing::info!(count = managed.len(), "starting pipeline threads");
    let mut handles = Vec::with_capacity(managed.len());
    let curve = Arc::new(config.curve.clone());

    for path in managed {
        let curve = curve.clone();
        let shutdown = shutdown.clone();
        let name_for_log = path.display().to_string();

        let handle = std::thread::Builder::new()
            .name(format!("maccel-{name_for_log}"))
            .spawn(move || {
                let pipeline = match Pipeline::new(&path, (*curve).clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!(path = %path.display(), error = %e, "failed to start pipeline");
                        return;
                    }
                };
                let name = pipeline.name().to_string();
                tracing::info!(device = %name, "pipeline started");
                if let Err(e) = pipeline.run(shutdown) {
                    tracing::error!(device = %name, error = %e, "pipeline exited with error");
                }
                tracing::info!(device = %name, "pipeline exited");
            })?;
        handles.push(handle);
    }

    // Wait for shutdown signal.
    while !shutdown.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }
    tracing::info!("shutdown signal received, joining pipeline threads");

    // Give threads a moment to exit cleanly. They check the shutdown flag
    // between event reads; in worst case (device blocked on read), they'll
    // be killed when the process exits.
    for handle in handles {
        let _ = handle.join();
    }

    tracing::info!("maccel daemon exiting");
    Ok(())
}

/// Discover evdev devices that match the config's allow/deny rules,
/// excluding any maccel virtual devices we may have created previously.
fn discover_managed_devices(config: &Config) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for (path, dev) in device::enumerate_pointer_devices() {
        let name = dev.name().unwrap_or("").to_string();
        // Skip our own virtual devices to avoid feedback loops.
        if name.starts_with("maccel") {
            tracing::debug!(path = %path.display(), name = %name, "skipping maccel-owned device");
            continue;
        }
        // Apply deny list (always wins).
        if config.devices.deny.iter().any(|d| d == &path) {
            tracing::debug!(path = %path.display(), "denied by config");
            continue;
        }
        // Apply allow list if non-empty.
        if !config.devices.allow.is_empty()
            && !config.devices.allow.iter().any(|d| d == &path)
        {
            tracing::debug!(path = %path.display(), "not in allow list");
            continue;
        }
        tracing::info!(path = %path.display(), name = %name, "will manage");
        out.push(path);
    }
    Ok(out)
}

fn install_signal_handler(shutdown: Arc<AtomicBool>) -> Result<()> {
    ctrlc::set_handler(move || {
        if shutdown.swap(true, Ordering::SeqCst) {
            // Second signal — force exit.
            tracing::warn!("second signal received, force-exiting");
            std::process::exit(130);
        } else {
            tracing::info!("signal received, initiating graceful shutdown");
            // Spawn a watchdog: if shutdown takes too long, force-exit.
            let s = shutdown.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(2));
                if s.load(Ordering::SeqCst) {
                    tracing::warn!("graceful shutdown timed out, force-exiting");
                    std::process::exit(130);
                }
            });
        }
    })
    .map_err(|e| anyhow::anyhow!("installing signal handler: {e}"))?;
    Ok(())
}
