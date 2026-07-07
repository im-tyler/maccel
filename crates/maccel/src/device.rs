use anyhow::Result;
use evdev::Device;
use std::path::PathBuf;

/// Enumerate pointer devices on the system, returning `(path, device)` pairs.
///
/// Pointer devices are those that:
///   - support EV_REL, AND
///   - either have "mouse" in their name (case-insensitive) OR explicitly
///     support both REL_X and REL_Y axes.
///
/// maccel-owned virtual devices are NOT filtered here — the daemon filters
/// them by name (`maccel ...`) so this function stays a pure enumeration.
pub fn enumerate_pointer_devices() -> Vec<(PathBuf, Device)> {
    evdev::enumerate()
        .filter(|(_, dev)| is_pointer_device(dev))
        .collect()
}

/// List pointer devices to stdout (used by `maccel devices`).
pub fn list_mice() -> Result<()> {
    let devices = enumerate_pointer_devices();

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

/// Predicate for "this evdev device is a pointer maccel should consider managing".
fn is_pointer_device(dev: &Device) -> bool {
    use evdev::EventType;
    use evdev::RelativeAxisType;

    let supports_relative = dev.supported_events().contains(EventType::RELATIVE);
    if !supports_relative {
        return false;
    }

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

/// Open an evdev device for reading.
///
/// Caller must have read permission on `/dev/input/event*` — in practice
/// this means running as root, or being a member of the `input` group.
#[allow(dead_code)]
pub fn open(path: &PathBuf) -> Result<Device> {
    Ok(Device::open(path)?)
}
