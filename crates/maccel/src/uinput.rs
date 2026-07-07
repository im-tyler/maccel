//! uinput virtual device wrapper.
//!
//! Wraps `evdev::uinput::VirtualDevice` with a more ergonomic surface for
//! maccel's needs: emit relative pointer motion and pass through button /
//! synchronization events.

use anyhow::{Context, Result};
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, Key, RelativeAxisType};

/// A virtual pointer device created via `/dev/uinput`.
///
/// Events written to a `Sink` appear to the compositor as if they came from
/// a real mouse. maccel grabs the real source device and forwards modified
/// events through this sink so the compositor only sees maccel's output.
pub struct Sink {
    device: VirtualDevice,
}

impl Sink {
    /// Create a new virtual pointer with the given display name.
    ///
    /// Caller must have write access to `/dev/uinput` (typically: root, or
    /// member of the `input` group with uinput rules configured).
    pub fn create(name: &str) -> Result<Self> {
        let mut axes = AttributeSet::<RelativeAxisType>::new();
        axes.insert(RelativeAxisType::REL_X);
        axes.insert(RelativeAxisType::REL_Y);
        axes.insert(RelativeAxisType::REL_WHEEL);
        axes.insert(RelativeAxisType::REL_HWHEEL);
        axes.insert(RelativeAxisType::REL_WHEEL_HI_RES);
        axes.insert(RelativeAxisType::REL_HWHEEL_HI_RES);

        let mut keys = AttributeSet::<Key>::new();
        // Common mouse buttons. Less common ones (forward/back/extra) included
        // so high-button-count mice pass through without loss.
        for key in [
            Key::BTN_LEFT,
            Key::BTN_RIGHT,
            Key::BTN_MIDDLE,
            Key::BTN_SIDE,
            Key::BTN_EXTRA,
            Key::BTN_FORWARD,
            Key::BTN_BACK,
            Key::BTN_TASK,
        ] {
            keys.insert(key);
        }

        let device = VirtualDeviceBuilder::new()?
            .name(name)
            .input_id(evdev::InputId::new(evdev::BusType::BUS_USB, 0x1234, 0x5678, 0x0001))
            .with_relative_axes(&axes)?
            .with_keys(&keys)?
            .build()
            .context("failed to create virtual uinput device")?;

        Ok(Self { device })
    }

    /// Emit a single relative motion event.
    ///
    /// The kernel timestamps the event at emission time; we don't control it.
    pub fn emit_relative(
        &mut self,
        axis: RelativeAxisType,
        value: i32,
    ) -> Result<()> {
        let event = evdev::InputEvent::new(EventType::RELATIVE, axis.0, value);
        self.device.emit(&[event]).context("emit relative")?;
        Ok(())
    }

    /// Pass through an event verbatim (for buttons, sync, scroll, etc.).
    pub fn emit(&mut self, event: evdev::InputEvent) -> Result<()> {
        self.device.emit(&[event]).context("emit passthrough")?;
        Ok(())
    }

    /// Returns the event types this sink supports (debugging).
    #[allow(dead_code)]
    pub fn supported_event_types(&self) -> &'static [EventType] {
        &[EventType::RELATIVE, EventType::KEY, EventType::SYNCHRONIZATION]
    }
}

pub use evdev::uinput::VirtualDeviceBuilder;
