// config.rs - User settings for the Astra Settings app.
//
// Loads `~/.config/astra/desktop.toml` (TOML) into a `SettingsConfig`
// struct. Falls back to a hard-coded AstraOS default (dark theme, violet
// accent, violet wallpaper, Windows taskbar, glassmorphism on, French
// language, auto-lock on, balanced performance) when the file is missing
// or unparseable, so a fresh ISO boot always renders a working UI.
//
// This file is shared with the Astra Shell (which reads only 4 of the 8
// fields: `taskbar_mode`, `accent_color`, `glassmorphism`, `wallpaper`).
// The 4 extra fields (`theme`, `language`, `auto_lock`,
// `performance_mode`) are decorated with `#[serde(default = ...)]` so a
// file written by an older shell still loads cleanly - serde just
// substitutes the per-field default for the missing keys.
//
// Example `desktop.toml`:
// ```toml
// theme = "dark"
// accent_color = "#818cf8"
// wallpaper = "default-violet.png"
// taskbar_mode = "win"
// glassmorphism = true
// language = "fr"
// auto_lock = true
// performance_mode = "balanced"
// ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Settings runtime configuration. Drives every page in the settings app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    /// `"dark"` or `"light"` - drives the GTK theme variant.
    #[serde(default = "SettingsConfig::default_theme")]
    pub theme: String,
    /// Hex accent color (e.g. `"#818cf8"`).
    pub accent_color: String,
    /// Wallpaper basename (e.g. `"default-violet.png"`). The shell
    /// resolves this against `/usr/share/backgrounds/astraos/`.
    pub wallpaper: String,
    /// `"win"` (Windows-style taskbar) or `"mac"` (floating dock).
    pub taskbar_mode: String,
    /// When true, panels render translucent + blurred.
    #[serde(default = "SettingsConfig::default_glass")]
    pub glassmorphism: bool,
    /// ISO 639-1 language code (e.g. `"fr"`, `"en"`).
    #[serde(default = "SettingsConfig::default_language")]
    pub language: String,
    /// When true, lock the screen after the configured idle timeout.
    #[serde(default = "SettingsConfig::default_auto_lock")]
    pub auto_lock: bool,
    /// `"eco"`, `"balanced"`, or `"performance"` - CPU governor hint.
    #[serde(default = "SettingsConfig::default_performance_mode")]
    pub performance_mode: String,
}

impl SettingsConfig {
    /// Default theme: AstraOS ships dark by default.
    fn default_theme() -> String {
        "dark".to_string()
    }
    /// Default glassmorphism: ON (matches the AstraOS brand).
    fn default_glass() -> bool {
        true
    }
    /// Default language: French (the project editor's primary language).
    fn default_language() -> String {
        "fr".to_string()
    }
    /// Default auto-lock: ON.
    fn default_auto_lock() -> bool {
        true
    }
    /// Default performance mode: balanced.
    fn default_performance_mode() -> String {
        "balanced".to_string()
    }

    /// Load the config from `~/.config/astra/desktop.toml`.
    ///
    /// On any read/parse error, falls back to [`SettingsConfig::default`]
    /// and logs a warning - the settings app must always open, even with
    /// a broken or missing config file.
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
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        log::info!("Settings saved to {:?}", path);
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

impl Default for SettingsConfig {
    /// AstraOS default - matches the ISO violet glassmorphism brand.
    fn default() -> Self {
        Self {
            theme: Self::default_theme(),
            accent_color: "#818cf8".to_string(),
            wallpaper: "default-violet.png".to_string(),
            taskbar_mode: "win".to_string(),
            glassmorphism: Self::default_glass(),
            language: Self::default_language(),
            auto_lock: Self::default_auto_lock(),
            performance_mode: Self::default_performance_mode(),
        }
    }
}

// ---------------------------------------------------------------------
// Tests - pure data shape checks, no filesystem.
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let d = SettingsConfig::default();
        assert_eq!(d.theme, "dark");
        assert_eq!(d.accent_color, "#818cf8");
        assert_eq!(d.wallpaper, "default-violet.png");
        assert_eq!(d.taskbar_mode, "win");
        assert!(d.glassmorphism);
        assert_eq!(d.language, "fr");
        assert!(d.auto_lock);
        assert_eq!(d.performance_mode, "balanced");
    }

    #[test]
    fn parse_full_toml() {
        let toml_src = r##"
        theme = "light"
        accent_color = "#E040FB"
        wallpaper = "classic.png"
        taskbar_mode = "mac"
        glassmorphism = false
        language = "en"
        auto_lock = false
        performance_mode = "performance"
        "##;
        let cfg: SettingsConfig = toml::from_str(toml_src).expect("parse");
        assert_eq!(cfg.theme, "light");
        assert!(!cfg.glassmorphism);
        assert_eq!(cfg.language, "en");
        assert_eq!(cfg.performance_mode, "performance");
    }

    #[test]
    fn parse_partial_toml_uses_serde_defaults() {
        // Simulates a file written by an older Astra Shell that only
        // knew the 4 original fields. The 4 extra fields must fall
        // back to their per-field serde defaults instead of erroring.
        let toml_src = r##"
        taskbar_mode = "mac"
        accent_color = "#E040FB"
        glassmorphism = false
        wallpaper = "/tmp/w.png"
        "##;
        let cfg: SettingsConfig = toml::from_str(toml_src).expect("parse");
        assert_eq!(cfg.taskbar_mode, "mac");
        assert!(!cfg.glassmorphism);
        assert_eq!(cfg.wallpaper, "/tmp/w.png");
        assert_eq!(cfg.theme, "dark", "theme should default");
        assert_eq!(cfg.language, "fr", "language should default");
        assert!(cfg.auto_lock, "auto_lock should default");
        assert_eq!(cfg.performance_mode, "balanced", "perf should default");
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
        assert_eq!(cfg.auto_lock, back.auto_lock);
        assert_eq!(cfg.performance_mode, back.performance_mode);
    }
}
