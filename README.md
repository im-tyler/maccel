# maccel

Mouse acceleration daemon for Linux. Brings macOS-style pointer feel to any compositor — Sway, Hyprland, Cosmic, GNOME, KDE, X11.

![Status](https://img.shields.io/badge/status-alpha-orange) ![License: MIT](https://img.shields.io/badge/license-MIT-blue) ![Rust](https://img.shields.io/badge/Rust-stable-000?logo=rust)

## Status

Alpha. The core daemon works end-to-end (evdev → curve → uinput) but the macOS curve parameters are untested against real hardware and per-device tuning. See [PLAN.md](PLAN.md) for the roadmap and design rationale.

## Why

Linux has never matched macOS pointer feel. libinput's `flat` and `adaptive` profiles both miss the specific curve Apple uses (base linear scaling + threshold-triggered acceleration + smoothing dampening). Every prior attempt (`mice`, `mos`, `libinput` profile tweaks) gets close but doesn't nail it. This is one of the most-complained-about aspects of Linux desktop history.

maccel is the standalone wedge for a larger Linux desktop effort ([Haven](https://github.com/im-tyler/haven)). Shipped independently because it works with any compositor, not just the parent DE.

## How it works

```
physical mouse → /dev/input/event* (evdev)
                  │
              maccel daemon
                  │ (curve applied per-frame, magnitude-preserving)
              /dev/uinput (virtual device)
                  │
              compositor (sees maccel as the pointer)
```

1. Reads raw pointer events via `libinput`/`evdev`
2. **Grabs** the source device so the compositor doesn't see unmodified events
3. Computes per-frame velocity from event timestamps
4. Applies a configurable acceleration curve with a `macos` preset
5. Emits the scaled motion via a virtual uinput device
6. Forwards all other events (buttons, scroll, sync) unchanged

The daemon runs multi-device — one pipeline thread per managed pointer.

## Install

### Build from source

```bash
git clone https://github.com/im-tyler/maccel.git
cd maccel
cargo build --release
```

The binary is at `target/release/maccel`. See [packaging/install.sh](packaging/install.sh) for a complete install with systemd unit + config directory:

```bash
sudo ./packaging/install.sh
```

### Permissions

maccel needs read access to `/dev/input/event*` (to grab source devices) and write access to `/dev/uinput` (to create the virtual pointer). Two ways:

**Option A — run as root (simplest):** the systemd unit in `packaging/maccel.service` already does this.

**Option B — `input` group + udev rule (recommended for daily use):**

```bash
sudo usermod -aG input $USER
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-maccel.rules
sudo udevadm control --reload-rules
# log out and back in for the group change to take effect
```

### Start the service

```bash
sudo systemctl enable --now maccel
sudo journalctl -u maccel -f   # tail logs
```

## Configure

Default config lives at `/etc/maccel/config.toml`. Override the path via `MACCEL_CONFIG=/path/to/config.toml`.

```toml
[curve]
# Rough approximation of macOS pointer acceleration.
# Tuning against real hardware happens in v0.2.
base_gain = 1.0        # multiplier in the linear region (v < threshold)
threshold = 5.5        # velocity in counts/ms above which acceleration kicks in
exponent = 1.5         # curve aggressiveness above threshold

[devices]
# Empty `allow` = auto-detect all pointer devices.
# allow = ["/dev/input/event3", "/dev/input/event7"]
# deny = ["/dev/input/event0"]
allow = []
deny = []
```

Print the current defaults to stdout with `maccel init`.

## CLI reference

```
maccel                  # run the daemon (default)
maccel run              # same as above, explicit
maccel status           # show version + default curve
maccel devices          # list detected pointer devices
maccel init             # print default TOML config
maccel profile list     # list available curve presets
maccel profile set X    # (v0.2) switch curve at runtime
```

Set `RUST_LOG=debug` (or `trace`) for verbose logs.

## Scope

**In scope:**
- Pointer acceleration for mice (not touchpads)
- Multi-device support, one daemon process
- TOML config; CLI surface
- Per-device allow/deny lists
- Preset curves (`macos`, `linear`)

**Out of scope (deliberate):**
- Touchpad gesture handling (libinput already does this well)
- Per-application curves (deferred to Haven DE integration)
- Button remapping (use `input-remapper`)
- Keyboard input
- Scroll acceleration (separate problem; v0.3)

## Roadmap

See [PLAN.md](PLAN.md) for the full roadmap and design rationale.

- **v0.1** (current) — Core daemon, evdev → curve → uinput pipeline, multi-device, systemd integration
- **v0.2** — GUI config app, runtime curve switching, per-device profiles, calibration tool
- **v0.3** — Scroll acceleration, additional presets, community-contributed device profiles, distribution packaging

## Why this exists

This is the wedge component for a larger Linux desktop effort. macOS-refugee Linux users have no path to native-feeling pointer behavior; this is the missing piece. Shipped standalone so it works with any compositor, not just the parent DE.

## License

MIT — see [LICENSE](LICENSE).
