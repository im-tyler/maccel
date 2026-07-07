# maccel — Plan

## What this is

A userspace mouse acceleration daemon for Linux that brings macOS-style pointer feel to any compositor. Reads raw evdev events, applies a configurable curve with a `macos` preset, re-emits via `uinput` so any Wayland or X11 desktop benefits.

## Why this exists

Linux has never matched macOS pointer feel. libinput's `flat` and `adaptive` profiles both miss the specific curve Apple uses (base linear scaling + threshold-triggered acceleration + smoothing dampening). Every prior attempt (`mice`, `mos`, libinput profile tweaks, `input-remapper`) gets close but doesn't nail it. Mouse feel is the most-complained-about aspect of Linux desktop history.

maccel is the standalone wedge for a larger Linux desktop effort (Haven DE). Shipped independently because it works with any compositor — Sway, Hyprland, Cosmic, GNOME, KDE, X11 — not just the parent DE.

## Architecture

### Data flow

```
physical mouse → /dev/input/event* (evdev)
                  │
              maccel daemon
                  │ (curve applied per-event)
              /dev/uinput (virtual device)
                  │
              compositor (sees maccel as the pointer)
```

### Stack

- **Rust** (edition 2021+)
- `evdev` crate — raw event reading from `/dev/input/event*`
- `uinput` crate — virtual device creation + event emission
- `toml` — config parsing
- `clap` — CLI argument parsing
- `tracing` — structured logging

### Why raw evdev (not libinput)

libinput applies its own acceleration that we'd need to disable per-device. Raw evdev bypasses that — we read motion events directly, apply our curve cleanly, and emit via uinput. No fighting with libinput's pipeline. Cost: no gesture support. Gestures are out of scope anyway.

## Workspace layout (target)

```
maccel/
  Cargo.toml              workspace root
  crates/
    maccel/               the daemon binary
      Cargo.toml
      src/
        main.rs           CLI entry, argument parsing
        daemon.rs         event loop, evdev → curve → uinput
        curve.rs          curve math, preset library
        device.rs         evdev reading + uinput writing
        config.rs         TOML config + per-device profiles
  config/
    default.toml          default config shipped as reference
  README.md  LICENSE  AGENTS.md  PLAN.md
```

Folders materialize as code lands — not pre-created empty.

## Curve math

The macOS algorithm (reverse-engineered by SmoothMouse project and others):

```
For each motion event (dx, dy) at time t:
  v = sqrt(dx² + dy²) / Δt              # velocity in counts/ms
  if v < threshold:
    scale = base_gain                    # linear region
  else:
    scale = base_gain × (v / threshold)^exponent   # accelerated region
  out_dx = dx × scale
  out_dy = dy × scale
```

Defaults approximate Apple's curve:
- `base_gain = 1.0`
- `threshold = 5.5 counts/ms`
- `exponent = 1.5`

These are starting points. Real macOS feel requires tuning against real hardware — v0.2 ships a calibration tool.

## Roadmap

### v0.1 — MVP (target: 4-6 weeks)
- Cargo workspace scaffolded
- evdev reader for one mouse device
- macOS curve implementation
- uinput emitter
- TOML config with `macos` preset
- CLI: `maccel status`, `maccel profile set macos`
- systemd unit file
- README updated with build + install instructions
- Initial release tag

### v0.2 — Polish + GUI
- Per-device profiles (different curves for different mice)
- Simple GUI config app (GTK4 or Iced)
- Curve calibration tool (compare feel to a reference)
- Additional presets: `linear`, `windows`, `macos-aggressive`
- Scroll acceleration (separate problem, separate curve)

### v0.3 — Community
- Community-contributed device profiles
- Distribution packaging (AUR, Debian, Fedora COPR, Homebrew)
- Documentation site
- Benchmark vs libinput presets

## Out of scope (deliberate)

- **Touchpad gesture handling** — libinput already does this well
- **Per-application curves** — would require compositor IPC; deferred to Haven DE integration
- **Button remapping** — use `input-remapper`, don't reinvent
- **Keyboard input** — separate concern
- **Configuration via compositor** — config is via TOML or GUI, not compositor-specific IPC

## Design decisions to defend

1. **Raw evdev over libinput.** Avoids fighting libinput's own acceleration. Costs us gesture support; gestures are out of scope anyway.
2. **Userspace daemon, not kernel module.** Easier to ship, easier to debug, no kernel CVE risk. Microsecond latency cost is negligible.
3. **Rust, not C.** Memory safety in input-handling code; aligns with the larger Haven DE ecosystem (smithay, libcosmic).
4. **TOML config for v0.1, GUI for v0.2.** Config-file-first forces a clean model; GUI wraps it later.
5. **No compositor patches required.** uinput makes maccel universal — the wedge property.

## Threats to validity

- macOS feel is partly subjective. maccel's default curve may not match every user's memory of macOS.
- libinput could evolve to do this itself. (Unlikely — libinput team has been clear they don't prioritize feel-matching.)
- Hardware variance: different mice report different counts/mm. Defaults may need per-device tuning.

## License

MIT — see [LICENSE](LICENSE).
