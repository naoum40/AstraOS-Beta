// config.rs - User configuration for the Astra Shell.
//
// Loads `~/.config/astra/desktop.toml` (TOML) into a `ShellConfig`
// struct. Falls back to a hard-coded AstraOS default (Windows-mode
// taskbar, violet glassmorphism, default wallpaper) when the file is
// missing or unparseable, so a fresh ISO 2 boot always renders a working
// shell.
//
// Example `desktop.toml`:
// ```toml
// taskbar_mode = "win"            # "win" or "mac"
// accent_color = "#818cf8"        # hex color
// glassmorphism = true            # bool
// wallpaper = "/usr/share/backgrounds/astraos/default-violet.png"
// ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Shell runtime configuration. Drives bar / dock / launcher construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    /// `"win"` (Windows-style taskbar) or `"mac"` (floating dock).
    pub taskbar_mode: String,
    /// Hex accent color (e.g. `"#818cf8"`). Drives hover / focus highlights.
    pub accent_color: String,
    /// When true, panels render translucent + blurred (CSS `backdrop-filter`).
    pub glassmorphism: bool,
    /// Absolute path to the active wallpaper PNG.
    pub wallpaper: String,
}

impl Default for ShellConfig {
    /// AstraOS default - matches the ISO 2 violet glassmorphism brand.
    fn default() -> Self {
        Self {
            taskbar_mode: "win".to_string(),
            accent_color: "#818cf8".to_string(),
            glassmorphism: true,
            wallpaper: "/usr/share/backgrounds/astraos/default-violet.png".to_string(),
        }
    }
}

impl ShellConfig {
    /// Load the config from `~/.config/astra/desktop.toml`.
    ///
    /// On any read/parse error, falls back to [`ShellConfig::default`]
    /// and logs a warning - the shell must always boot, even with a
    /// broken or missing config file.
    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    log::warn!(
                        "Failed to parse {:?}: {} - falling back to defaults",
                        path,
                        e
                    );
                    Self::default()
                }
            },
            Err(e) => {
                log::info!(
                    "Config file {:?} not readable ({}); using defaults",
                    path,
                    e
                );
                Self::default()
            }
        }
    }

    /// Persist the config back to `~/.config/astra/desktop.toml`.
    ///
    /// Creates the parent directory if it doesn't exist.
    #[allow(dead_code)] // exposed for the future settings UI (task 3-x)
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        log::info!("Config saved to {:?}", path);
        Ok(())
    }

    /// Resolve `~/.config/astra/desktop.toml`.
    ///
    /// Honors `$HOME`; falls back to `/home/astra` (AstraOS default user)
    /// when `HOME` is unset (e.g. when running under a non-login session).
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
        PathBuf::from(home).join(".config/astra/desktop.toml")
    }
}

// ---------------------------------------------------------------------
// Tests - pure data shape checks, no filesystem.
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_win_mode_violet() {
        let d = ShellConfig::default();
        assert_eq!(d.taskbar_mode, "win");
        assert_eq!(d.accent_color, "#818cf8");
        assert!(d.glassmorphism);
        assert!(d.wallpaper.ends_with("default-violet.png"));
    }

    #[test]
    fn parse_full_toml() {
        // NOTE: r##"..."## (not r#"..."#) - the TOML content contains
        // `"#` (accent_color = "#E040FB") which would prematurely close
        // a single-hash raw string.
        let toml_src = r##"
        taskbar_mode = "mac"
        accent_color = "#E040FB"
        glassmorphism = false
        wallpaper = "/tmp/w.png"
        "##;
        let cfg: ShellConfig = toml::from_str(toml_src).expect("parse");
        assert_eq!(cfg.taskbar_mode, "mac");
        assert!(!cfg.glassmorphism);
        assert_eq!(cfg.wallpaper, "/tmp/w.png");
    }

    #[test]
    fn parse_partial_toml_falls_back() {
        // Missing fields - serde should reject so the caller falls back
        // to defaults (this is the documented behavior in `load`).
        let toml_src = r##"taskbar_mode = "mac""##;
        let res: Result<ShellConfig, _> = toml::from_str(toml_src);
        assert!(res.is_err(), "partial TOML must not silently default");
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let cfg = ShellConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: ShellConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg.taskbar_mode, back.taskbar_mode);
        assert_eq!(cfg.accent_color, back.accent_color);
        assert_eq!(cfg.glassmorphism, back.glassmorphism);
        assert_eq!(cfg.wallpaper, back.wallpaper);
    }
}
