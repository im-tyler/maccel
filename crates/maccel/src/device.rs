use anyhow::Result;
use std::path::PathBuf;

/// List pointer devices that maccel could manage.
///
/// For v0.1 scaffolding, this enumerates evdev devices that:
///   - support relative events (RELATIVE event type), AND
///   - have "mouse" in their name (case-insensitive), OR
///   - explicitly support REL_X and REL_Y axes
///
/// Real device classification will be more sophisticated in v0.1 proper:
/// we'll want to look at axis resolution, polling rate, vendor/product IDs,
/// and the device's bus type to distinguish pointer devices from
/// keyboards-with-pointer-nub or other oddities.
pub fn list_mice() -> Result<()> {
    let devices: Vec<_> = evdev::enumerate()
        .filter(|(_, dev)| is_pointer_device(dev))
        .collect();

    println!("Detected pointer devices:");
    if devices.is_empty() {
        println!("  (none found — run as root if devices aren't showing up)");
        return Ok(());
    }

    for (path, dev) in devices {
        let name = dev.name().unwrap_or("(unnamed)");
        let phys = dev.physical_path().unwrap_or("?");
        println!("  {}", path.display());
        println!("    name: {name}");
        println!("    phys: {phys}");
    }

    Ok(())
}

/// Predicate for "this evdev device is a pointer maccel should manage".
fn is_pointer_device(dev: &evdev::Device) -> bool {
    use evdev::EventType;
    use evdev::RelativeAxisType;

    let supports_relative = dev
        .supported_events()
        .contains(EventType::RELATIVE);

    if !supports_relative {
        return false;
    }

    // Either name contains "mouse" or it explicitly supports REL_X + REL_Y.
    let named_like_mouse = dev
        .name()
        .map(|n| n.to_lowercase().contains("mouse"))
        .unwrap_or(false);

    let supports_xy = dev
        .supported_relative_axes()
        .map(|axes| {
            axes.contains(RelativeAxisType::REL_X)
                && axes.contains(RelativeAxisType::REL_Y)
        })
        .unwrap_or(false);

    named_like_mouse || supports_xy
}

/// Open an evdev device for reading. Returns the owned device on success.
///
/// Caller must have read permission on `/dev/input/event*`. In practice
/// this means running as root, or being a member of the `input` group.
pub fn open(path: &PathBuf) -> Result<evdev::Device> {
    let device = evdev::Device::open(path)?;
    Ok(device)
}
