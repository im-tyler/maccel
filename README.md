# maccel

Mouse acceleration daemon for Linux. Brings macOS-style pointer feel to any Wayland or X11 compositor.

![Status](https://img.shields.io/badge/status-planning-lightgrey) ![License: MIT](https://img.shields.io/badge/license-MIT-blue)

## Status

Planning. Not yet implemented.

## Why

Linux has never matched macOS pointer feel. libinput's `flat` and `adaptive` profiles both miss the specific curve Apple uses (base linear scaling + threshold-triggered acceleration + smoothing dampening). Every prior attempt (`mice`, `mos`, `libinput` profile tweaks, `input-remapper`) gets close but doesn't nail it. The result is the most-complained-about aspect of Linux desktop history.

## Approach

A userspace daemon that:

1. Reads pointer events via `libinput` (or raw `evdev`)
2. Applies a configurable acceleration curve with a `macos` preset matching Apple's published algorithm
3. Re-emits modified events via `uinput` so any compositor benefits — Sway, Hyprland, Cosmic, GNOME, KDE, X11
4. Per-device profiles (TOML config) — different mice have different native curves
5. Single Rust binary, no DE dependency, no management server

## Scope

**In scope (MVP):**
- Pointer acceleration only (not scrolling, not gestures, not buttons)
- libinput event source
- uinput event sink
- macOS-curve default + a few presets
- Per-device TOML config
- CLI: `maccel status`, `maccel profile set <name>`, `maccel curve set macos`

**Out of scope (deliberately):**
- GUI config (v0.2)
- Scroll acceleration (separate problem, separate curve math)
- Touchpad gesture handling (libinput already does this well)
- Per-application curves (could be added later via compositor IPC)

## Roadmap

| Version | Scope |
|---|---|
| v0.1 | libinput → uinput pipeline, macOS curve preset, per-device TOML, CLI |
| v0.2 | Simple GUI config app (GTK4 or Iced) |
| v0.3 | Scroll acceleration, additional curve presets, community-contributed device profiles |

## Why this exists

This is the wedge component for a larger Linux desktop effort. macOS-refugee Linux users have no path to native-feeling pointer behavior; this is the missing piece. Shipped standalone so it works with any compositor, not just the parent DE.

## License

MIT.
