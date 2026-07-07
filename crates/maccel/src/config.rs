use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::curve::Curve;

/// Top-level maccel configuration.
///
/// Default location: `/etc/maccel/config.toml`
/// Override via `--config <path>` (TODO when daemon.rs grows args).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub curve: Curve,
    #[serde(default)]
    pub devices: DeviceConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            curve: Curve::default(),
            devices: DeviceConfig::default(),
        }
    }
}

/// Per-device filtering rules.
///
/// If `allow` is non-empty, only listed device paths are managed.
/// `deny` always wins (explicit opt-out).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceConfig {
    #[serde(default)]
    pub allow: Vec<PathBuf>,
    #[serde(default)]
    pub deny: Vec<PathBuf>,
}

impl Config {
    /// Load config from a path. If the path doesn't exist, returns defaults.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::debug!("config path {} does not exist, using defaults", path.display());
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&text)?;
        Ok(config)
    }

    /// Serialize defaults to a TOML string (for `maccel init` or reference).
    pub fn defaults_as_toml() -> Result<String> {
        Ok(toml::to_string_pretty(&Self::default())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_roundtrip() {
        let c = Config::default();
        let toml_text = toml::to_string(&c).unwrap();
        let c2: Config = toml::from_str(&toml_text).unwrap();
        assert!((c.curve.base_gain - c2.curve.base_gain).abs() < 1e-9);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let c = Config::load(Path::new("/nonexistent/maccel-config.toml")).unwrap();
        assert_eq!(c.curve.base_gain, 1.0);
    }
}
