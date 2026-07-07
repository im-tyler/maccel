use anyhow::Result;

pub fn run() -> Result<()> {
    tracing::info!("maccel daemon starting (v0.1.0-dev scaffold)");

    // TODO(v0.1):
    // 1. Load config (default path: /etc/maccel/config.toml, override via --config)
    // 2. Discover mouse devices (filter evdev for relative-axis emitters)
    // 3. Open /dev/uinput and create a virtual pointer
    // 4. Spawn one thread per managed device
    // 5. Each thread: blocking read evdev event -> apply curve -> write uinput event
    // 6. Block on a signal handler for clean shutdown (SIGTERM, SIGINT)

    tracing::warn!("daemon: scaffold only — no event processing implemented yet");
    tracing::info!("daemon exiting");
    Ok(())
}
