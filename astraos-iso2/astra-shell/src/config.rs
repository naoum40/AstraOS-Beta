use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub taskbar_mode: String,
    pub accent_color: String,
    pub glassmorphism: bool,
    pub wallpaper: String,
}

impl Default for ShellConfig {
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

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
        PathBuf::from(home).join(".config/astra/desktop.toml")
    }
}

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
