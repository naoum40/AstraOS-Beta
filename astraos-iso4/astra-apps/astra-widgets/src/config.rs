// config.rs - User configuration for Astra Widgets.
//
// Loads `~/.config/astra/widgets.toml` (TOML) into a `WidgetsConfig`
// struct. The file holds the enabled-widget list and per-widget screen
// positions so a desktop session restores the user's layout across
// reboots. Falls back to a hard-coded AstraOS default (all four widgets
// enabled, stacked top-left) when the file is missing or unparseable —
// a fresh ISO 4 boot always renders a working widget set.
//
// Example `widgets.toml`:
// ```toml
// enabled_widgets = ["clock", "calendar", "weather", "battery"]
//
// [positions]
// clock = [64, 64]
// calendar = [64, 220]
// weather = [64, 480]
// battery = [64, 620]
// ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Desktop widget configuration. Drives which widgets spawn on boot
/// and where each one is placed on screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetsConfig {
    /// Widget names that should be spawned at startup. Supported keys:
    /// `"clock"`, `"calendar"`, `"weather"`, `"battery"`.
    pub enabled_widgets: Vec<String>,
    /// Per-widget (x, y) screen coordinates in logical pixels.
    /// Wayland compositors may ignore these (positions are advisory).
    pub positions: HashMap<String, (i32, i32)>,
}

impl Default for WidgetsConfig {
    /// AstraOS ISO 4 default — all four widgets enabled, stacked top-left
    /// with a 64 px margin so they don't hug the screen edge.
    fn default() -> Self {
        let mut positions = HashMap::new();
        positions.insert("clock".to_string(), (64, 64));
        positions.insert("calendar".to_string(), (360, 64));
        positions.insert("weather".to_string(), (64, 240));
        positions.insert("battery".to_string(), (360, 360));
        Self {
            enabled_widgets: vec![
                "clock".to_string(),
                "calendar".to_string(),
                "weather".to_string(),
                "battery".to_string(),
            ],
            positions,
        }
    }
}

impl WidgetsConfig {
    /// Load the config from `~/.config/astra/widgets.toml`.
    ///
    /// On any read/parse error, falls back to [`WidgetsConfig::default`]
    /// and persists that default so the next boot finds an existing file.
    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    log::warn!(
                        "Failed to parse {:?}: {} — falling back to defaults",
                        path,
                        e
                    );
                    Self::default()
                }
            },
            Err(e) => {
                log::info!(
                    "Config file {:?} not readable ({}) — seeding defaults",
                    path,
                    e
                );
                let cfg = Self::default();
                let _ = cfg.save();
                cfg
            }
        }
    }

    /// Persist the config back to `~/.config/astra/widgets.toml`.
    ///
    /// Creates the parent directory if it doesn't exist. Failure is
    /// non-fatal — widgets remain interactive even if positions can't
    /// be saved (logged via `log::error!`).
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        log::debug!("Config saved to {:?}", path);
        Ok(())
    }

    /// Resolve `~/.config/astra/widgets.toml`.
    ///
    /// Honors `$HOME`; falls back to `/home/astra` (AstraOS default user)
    /// when `HOME` is unset (e.g. when running under a non-login session).
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
        PathBuf::from(home).join(".config/astra/widgets.toml")
    }
}

// ---------------------------------------------------------------------
// Tests — pure data shape checks, no filesystem.
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_enable_all_four_widgets() {
        let d = WidgetsConfig::default();
        assert_eq!(d.enabled_widgets.len(), 4);
        for w in &["clock", "calendar", "weather", "battery"] {
            assert!(d.enabled_widgets.iter().any(|s| s == w), "missing {}", w);
        }
    }

    #[test]
    fn default_positions_are_nonzero() {
        let d = WidgetsConfig::default();
        for (k, (x, y)) in &d.positions {
            assert!(*x > 0 && *y > 0, "position for {} is too small", k);
        }
    }

    #[test]
    fn parse_full_toml() {
        let src = r#"
enabled_widgets = ["clock", "battery"]

[positions]
clock = [100, 200]
battery = [100, 400]
"#;
        let cfg: WidgetsConfig = toml::from_str(src).expect("parse");
        assert_eq!(cfg.enabled_widgets, vec!["clock", "battery"]);
        assert_eq!(cfg.positions.get("clock"), Some(&(100, 200)));
        assert_eq!(cfg.positions.get("battery"), Some(&(100, 400)));
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let cfg = WidgetsConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: WidgetsConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg.enabled_widgets, back.enabled_widgets);
        assert_eq!(cfg.positions.len(), back.positions.len());
    }
}
