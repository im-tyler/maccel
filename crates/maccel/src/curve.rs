use serde::{Deserialize, Serialize};

/// Acceleration curve parameters.
///
/// Approximates the macOS algorithm: a linear region below `threshold`
/// transitions to a power-law accelerated region above it. Defaults are
/// a rough fit to Apple's published behavior; real macOS feel requires
/// per-device tuning (see PLAN.md for the calibration roadmap).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curve {
    /// Gain applied in the linear region (v < threshold).
    pub base_gain: f64,
    /// Velocity threshold in counts/ms above which acceleration kicks in.
    pub threshold: f64,
    /// Exponent for the accelerated region. 1.0 = linear, >1.0 = aggressive.
    pub exponent: f64,
}

impl Default for Curve {
    fn default() -> Self {
        Self::macos()
    }
}

impl Curve {
    /// Rough approximation of macOS pointer acceleration.
    pub fn macos() -> Self {
        Self {
            base_gain: 1.0,
            threshold: 5.5,
            exponent: 1.5,
        }
    }

    /// Pure linear scaling, no acceleration. Useful as a baseline and for
    /// users who want macOS feel without the curve (LinearMouse-style).
    pub fn linear() -> Self {
        Self {
            base_gain: 1.0,
            threshold: f64::INFINITY,
            exponent: 1.0,
        }
    }

    /// Apply the curve to a single motion event.
    ///
    /// - `dx`, `dy`: motion in device counts
    /// - `dt_ms`: time since the previous event in milliseconds
    ///
    /// Returns the scaled `(dx, dy)`.
    pub fn apply(&self, dx: f64, dy: f64, dt_ms: f64) -> (f64, f64) {
        if dt_ms <= 0.0 {
            return (dx * self.base_gain, dy * self.base_gain);
        }

        let v = (dx * dx + dy * dy).sqrt() / dt_ms;
        let scale = if v < self.threshold {
            self.base_gain
        } else {
            self.base_gain * (v / self.threshold).powf(self.exponent)
        };

        (dx * scale, dy * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_below_threshold() {
        let c = Curve::macos();
        // v = 1 / 10 = 0.1, well below threshold 5.5
        let (ox, oy) = c.apply(1.0, 0.0, 10.0);
        assert!((ox - 1.0).abs() < 1e-9, "got {ox}");
        assert!(oy.abs() < 1e-9);
    }

    #[test]
    fn diagonal_motion_uses_magnitude() {
        let c = Curve::macos();
        let (dx, dy) = (3.0, 4.0); // magnitude = 5
        let dt = 1.0;
        let (ox, oy) = c.apply(dx, dy, dt);
        // v = 5/1 = 5, below threshold 5.5 -> no acceleration
        assert!((ox - 3.0).abs() < 1e-9);
        assert!((oy - 4.0).abs() < 1e-9);
    }

    #[test]
    fn accelerated_above_threshold() {
        let c = Curve::macos();
        // v = 100/1 = 100, well above threshold
        let (ox, _oy) = c.apply(100.0, 0.0, 1.0);
        let expected_scale = (100.0_f64 / 5.5).powf(1.5);
        assert!((ox - 100.0 * expected_scale).abs() < 1e-6, "got {ox}");
    }

    #[test]
    fn linear_curve_never_accelerates() {
        let c = Curve::linear();
        let (ox, _) = c.apply(1000.0, 0.0, 0.001);
        assert!((ox - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn zero_dt_falls_back_to_base_gain() {
        let c = Curve::macos();
        let (ox, oy) = c.apply(50.0, 50.0, 0.0);
        assert!((ox - 50.0).abs() < 1e-9);
        assert!((oy - 50.0).abs() < 1e-9);
    }

    #[test]
    fn serialization_roundtrip() {
        let c = Curve::macos();
        let s = toml::to_string(&c).unwrap();
        let c2: Curve = toml::from_str(&s).unwrap();
        assert!((c.base_gain - c2.base_gain).abs() < 1e-9);
        assert!((c.threshold - c2.threshold).abs() < 1e-9);
        assert!((c.exponent - c2.exponent).abs() < 1e-9);
    }
}
