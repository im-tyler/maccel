//! Per-device event pipeline: source evdev reader → curve → virtual uinput sink.
//!
//! One `Pipeline` instance manages a single source device. The daemon spawns
//! one thread per managed device, each running its own pipeline loop.

use anyhow::{Context, Result};
use evdev::{Device, EventType, RelativeAxisType};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::curve::Curve;
use crate::uinput::Sink;

/// Default frame interval (ms) assumed when no prior timestamp exists.
const DEFAULT_FRAME_MS: f64 = 8.0;

/// One source device → one virtual sink, with the given curve applied.
pub struct Pipeline {
    source: Device,
    sink: Sink,
    curve: Curve,
    name: String,
}

impl Pipeline {
    pub fn new(source_path: &Path, curve: Curve) -> Result<Self> {
        let source =
            Device::open(source_path).with_context(|| {
                format!("opening source device {}", source_path.display())
            })?;

        let source_name = source.name().unwrap_or("maccel source").to_string();
        let sink_name = format!("maccel virtual pointer ({source_name})");
        let sink = Sink::create(&sink_name)
            .with_context(|| format!("creating virtual sink for {source_name}"))?;

        Ok(Self {
            source,
            sink,
            curve,
            name: source_name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Run the pipeline until `shutdown` is set or an unrecoverable error occurs.
    ///
    /// On exit, attempts to ungrab the source device. Kernel will release the
    /// grab automatically when the source fd is dropped, so this is best-effort.
    pub fn run(mut self, shutdown: Arc<AtomicBool>) -> Result<()> {
        // Grab the source so events don't reach the compositor directly.
        // Without this, the compositor would see both the original and the
        // maccel-modified events, causing doubled motion.
        self.source
            .grab()
            .with_context(|| format!("grabbing source device {}", self.name))?;
        tracing::info!(device = %self.name, "grabbed source device");

        let result = self.event_loop(&shutdown);

        // Best-effort ungrab; kernel releases on fd drop regardless.
        let _ = self.source.ungrab();
        tracing::info!(device = %self.name, "released source device");

        result
    }

    fn event_loop(&mut self, shutdown: &Arc<AtomicBool>) -> Result<()> {
        let mut last_frame_time: Option<SystemTime> = None;
        // Pending X/Y for the current frame. Held until SYN_REPORT so we can
        // apply the curve to the combined magnitude (preserves diagonals).
        let mut pending_x: Option<(i32, SystemTime)> = None;
        let mut pending_y: Option<(i32, SystemTime)> = None;

        while !shutdown.load(Ordering::SeqCst) {
            let events = match self.source.fetch_events() {
                Ok(iter) => iter,
                Err(ref e)
                    if e.raw_os_error() == Some(nix::errno::Errno::EAGAIN as i32) =>
                {
                    // Non-blocking fd returned nothing; brief sleep + retry.
                    std::thread::sleep(Duration::from_millis(2));
                    continue;
                }
                Err(e) => return Err(anyhow::anyhow!("reading events from {}: {e}", self.name)),
            };

            for event in events {
                match event.event_type() {
                    EventType::RELATIVE => {
                        let code = event.code();
                        if code == RelativeAxisType::REL_X.0 {
                            pending_x = Some((event.value(), event.timestamp()));
                        } else if code == RelativeAxisType::REL_Y.0 {
                            pending_y = Some((event.value(), event.timestamp()));
                        } else {
                            // Scroll wheels and other relative axes: pass through.
                            self.sink.emit(event)?;
                        }
                    }
                    EventType::SYNCHRONIZATION => {
                        // End of frame. Apply curve to any pending X/Y, emit, then forward SYN.
                        let frame_time = match (pending_x, pending_y) {
                            (Some((_, xt)), Some((_, yt))) => Some(xt.max(yt)),
                            (Some((_, t)), None) => Some(t),
                            (None, Some((_, t))) => Some(t),
                            (None, None) => None,
                        };

                        if let Some(t) = frame_time {
                            let dt_ms = dt_since(last_frame_time, t);
                            last_frame_time = Some(t);

                            let (sx, sy) = self.curve.apply(
                                pending_x.map(|(v, _)| v as f64).unwrap_or(0.0),
                                pending_y.map(|(v, _)| v as f64).unwrap_or(0.0),
                                dt_ms,
                            );

                            if pending_x.take().is_some() {
                                self.sink.emit_relative(RelativeAxisType::REL_X, round_i32(sx))?;
                            }
                            if pending_y.take().is_some() {
                                self.sink.emit_relative(RelativeAxisType::REL_Y, round_i32(sy))?;
                            }
                        }

                        // Forward the original SYN_REPORT so the compositor flushes.
                        self.sink.emit(event)?;
                    }
                    _ => {
                        // Buttons (EV_KEY), miscellaneous, etc. — pass through.
                        self.sink.emit(event)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn dt_since(prev: Option<SystemTime>, now: SystemTime) -> f64 {
    match prev {
        Some(p) => now
            .duration_since(p)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(DEFAULT_FRAME_MS)
            .max(0.5),
        None => DEFAULT_FRAME_MS,
    }
}

/// Round to nearest i32, clamping to i16 range to match kernel input_event.value.
fn round_i32(v: f64) -> i32 {
    let rounded = v.round();
    if rounded > i16::MAX as f64 {
        i16::MAX as i32
    } else if rounded < i16::MIN as f64 {
        i16::MIN as i32
    } else {
        rounded as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_i32_normal() {
        assert_eq!(round_i32(3.6), 4);
        assert_eq!(round_i32(3.4), 3);
        assert_eq!(round_i32(0.0), 0);
    }

    #[test]
    fn round_i32_clamps() {
        assert_eq!(round_i32(1e9), i16::MAX as i32);
        assert_eq!(round_i32(-1e9), i16::MIN as i32);
    }

    #[test]
    fn dt_since_handles_first_frame() {
        let now = SystemTime::now();
        assert!((dt_since(None, now) - DEFAULT_FRAME_MS).abs() < 1e-9);
    }
}

// Re-export for downstream code that wants to construct raw events if needed.
