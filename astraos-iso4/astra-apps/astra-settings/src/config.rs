// config.rs - User configuration for Astra Settings.
//
// Loads `~/.config/astra/desktop.toml` (TOML) into a `SettingsConfig`
// struct. Falls back to a hard-coded AstraOS default (dark theme, violet
// accent, default wallpaper, Windows-mode taskbar, glassmorphism on,
// French locale, auto-lock on, balanced performance) when the file is
// missing or unparseable, so a fresh ISO 3 boot always renders a working
// settings panel.
//
// Example `desktop.toml`:
// ```toml
// theme = "dark"
// accent_color = "#818cf8"
// wallpaper = "/usr/share/backgrounds/astraos/default-violet.png"
// taskbar_mode = "win"            # "win" or "mac"
// glassmorphism = true
// language = "fr"
// auto_lock = true
// performance_mode = "balanced"   # "eco" | "balanced" | "performance"
// ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Settings panel configuration. Drives every toggle / dropdown in the
/// Personalization + System pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    /// `"dark"` or `"light"` — AstraOS default is dark.
    pub theme: String,
    /// Hex accent color (e.g. `"#818cf8"`). Drives highlights + switches.
    pub accent_color: String,
    /// Absolute path to the active wallpaper PNG.
    pub wallpaper: String,
    /// `"win"` (Windows-style taskbar) or `"mac"` (floating dock).
    pub taskbar_mode: String,
    /// When true, panels render translucent + blurred (CSS `backdrop-filter`).
    pub glassmorphism: bool,
    /// ISO 639-1 language code (e.g. `"fr"`, `"en"`). Default `"fr"`.
    pub language: String,
    /// When true, the screen auto-locks after a period of inactivity.
    pub auto_lock: bool,
    /// `"eco"` | `"balanced"` | `"performance"`.
    pub performance_mode: String,
}

impl Default for SettingsConfig {
    /// AstraOS default — matches the ISO 3 violet glassmorphism brand.
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            accent_color: "#818cf8".to_string(),
            wallpaper: "/usr/share/backgrounds/astraos/default-violet.png".to_string(),
            taskbar_mode: "win".to_string(),
            glassmorphism: true,
            language: "fr".to_string(),
            auto_lock: true,
            performance_mode: "balanced".to_string(),
        }
    }
}

impl SettingsConfig {
    /// Load the config from `~/.config/astra/desktop.toml`.
    ///
    /// On any read/parse error, falls back to [`SettingsConfig::default`]
    /// and logs a warning — the settings panel must always boot, even
    /// with a broken or missing config file.
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
                    "Config file {:?} not readable ({}) — using defaults",
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
    #[allow(dead_code)] // exposed for the future settings UI save action
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
// Tests — pure data shape checks, no filesystem.
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_dark_violet_balanced() {
        let d = SettingsConfig::default();
        assert_eq!(d.theme, "dark");
        assert_eq!(d.accent_color, "#818cf8");
        assert!(d.glassmorphism);
        assert!(d.wallpaper.ends_with("default-violet.png"));
        assert_eq!(d.taskbar_mode, "win");
        assert_eq!(d.language, "fr");
        assert!(d.auto_lock);
        assert_eq!(d.performance_mode, "balanced");
    }

    #[test]
    fn parse_full_toml() {
        // NOTE: r##"..."## (not r#"..."#) — the TOML content contains
        // `"#` (accent_color = "#E040FB") which would prematurely close
        // a single-hash raw string.
        let toml_src = r##"
        theme = "light"
        accent_color = "#E040FB"
        wallpaper = "/tmp/w.png"
        taskbar_mode = "mac"
        glassmorphism = false
        language = "en"
        auto_lock = false
        performance_mode = "performance"
        "##;
        let cfg: SettingsConfig = toml::from_str(toml_src).expect("parse");
        assert_eq!(cfg.theme, "light");
        assert_eq!(cfg.accent_color, "#E040FB");
        assert!(!cfg.glassmorphism);
        assert_eq!(cfg.language, "en");
        assert!(!cfg.auto_lock);
        assert_eq!(cfg.performance_mode, "performance");
    }

    #[test]
    fn parse_partial_toml_falls_back() {
        // Missing fields — serde should reject so the caller falls back
        // to defaults (this is the documented behavior in `load`).
        let toml_src = r##"theme = "dark""##;
        let res: Result<SettingsConfig, _> = toml::from_str(toml_src);
        assert!(res.is_err(), "partial TOML must not silently default");
    }

    #[test]
    fn roundtrip_serialize_deserialize() {
        let cfg = SettingsConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: SettingsConfig = toml::from_str(&s).unwrap();
        assert_eq!(cfg.theme, back.theme);
        assert_eq!(cfg.accent_color, back.accent_color);
        assert_eq!(cfg.glassmorphism, back.glassmorphism);
        assert_eq!(cfg.wallpaper, back.wallpaper);
        assert_eq!(cfg.language, back.language);
        assert_eq!(cfg.performance_mode, back.performance_mode);
    }
}
