// config.rs - Astra VM configuration model + persistence layer.
//
// Two serde structs:
//
//   - `VmConfig`: one virtual machine definition (id, name, ISO,
//     architecture, RAM, CPU, disk, timestamps). 4096 MB RAM / 2 cores
//     / 20 GB disk defaults are applied via serde `default = "…"`
//     attributes so old config files missing those fields still parse.
//
//   - `AppConfig`: top-level config written to
//     `~/.config/astra-vm/config.toml`. Holds the list of VMs + the
//     QEMU binary path + the default ISO directory.
//
// Persistence is plain TOML (human-readable + git-friendly). All file
// I/O is checked through `Box<dyn std::error::Error>` so the caller can
// surface a single error string to the UI.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// GApplication config subdirectory under `~/.config`.
const CONFIG_SUBDIR: &str = "astra-vm";

/// Config file name inside the config subdir.
const CONFIG_FILENAME: &str = "config.toml";

/// VM image storage subdirectory under `~/.local/share/astra-vm`.
pub const VM_DATA_SUBDIR: &str = "astra-vm/vms";

/// ISO image storage subdirectory under `~/.local/share/astra-vm`.
pub const ISO_DATA_SUBDIR: &str = "astra-vm/isos";

/// QEMU portable subdirectory under `~/.local/share/astra-vm`.
pub const QEMU_DATA_SUBDIR: &str = "astra-vm/qemu";

/// Default RAM (MB) for a new VM when the field is missing from TOML.
fn default_ram() -> u32 {
    4096
}

/// Default CPU core count for a new VM when the field is missing.
fn default_cpu() -> u32 {
    2
}

/// Default disk size (GB) for a new VM when the field is missing.
fn default_disk() -> u32 {
    20
}

/// Default ISO directory (under the user's data dir) for new ISOs.
fn default_iso_dir() -> String {
    data_dir()
        .join(ISO_DATA_SUBDIR)
        .to_string_lossy()
        .into_owned()
}

/// One virtual machine definition. Serialized into the `[[vms]]` array
/// of `config.toml`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VmConfig {
    /// Stable identifier (UUID-style 32-char hex). Generated at creation.
    pub id: String,
    /// User-visible display name (e.g. "AstraOS Dev").
    pub name: String,
    /// Absolute path to the boot ISO image.
    pub iso_path: String,
    /// Target architecture: "x86_64" or "aarch64".
    pub architecture: String,
    /// Allocated RAM in megabytes. Defaults to 4096.
    #[serde(default = "default_ram")]
    pub ram_mb: u32,
    /// Number of virtual CPU cores. Defaults to 2.
    #[serde(default = "default_cpu")]
    pub cpu_cores: u32,
    /// Virtual disk size in gigabytes. Defaults to 20.
    #[serde(default = "default_disk")]
    pub disk_gb: u32,
    /// Creation timestamp (Unix seconds).
    pub created_at: i64,
    /// Last launch timestamp (Unix seconds), or None if never launched.
    #[serde(default)]
    pub last_used: Option<i64>,
}

impl VmConfig {
    /// Build a fresh `VmConfig` with a random id + the given fields.
    /// `created_at` is stamped at call time.
    pub fn new(name: &str, iso_path: &str, architecture: &str) -> Self {
        Self {
            id: new_vm_id(),
            name: name.to_string(),
            iso_path: iso_path.to_string(),
            architecture: architecture.to_string(),
            ram_mb: default_ram(),
            cpu_cores: default_cpu(),
            disk_gb: default_disk(),
            created_at: now_unix_seconds(),
            last_used: None,
        }
    }

    /// Best-effort OS family detection from name + ISO path. Drives the
    /// icon emoji + gradient style applied by `VmItem`.
    ///
    /// Detection order:
    ///   1. architecture == "aarch64" → `Arm`
    ///   2. name/iso contains "windows" → `Windows`
    ///   3. name/iso contains "astra" → `AstraOS`
    ///   4. name/iso contains "linux"/"ubuntu"/"arch"/"fedora" → `Linux`
    ///   5. otherwise → `Unknown`
    pub fn os_family(&self) -> OsFamily {
        if self.architecture.eq_ignore_ascii_case("aarch64") {
            return OsFamily::Arm;
        }
        let hay = format!("{} {}", self.name, self.iso_path).to_lowercase();
        if hay.contains("windows") {
            OsFamily::Windows
        } else if hay.contains("astra") {
            OsFamily::AstraOS
        } else if hay.contains("linux")
            || hay.contains("ubuntu")
            || hay.contains("arch")
            || hay.contains("fedora")
            || hay.contains("debian")
        {
            OsFamily::Linux
        } else {
            OsFamily::Unknown
        }
    }
}

/// OS family inferred from a `VmConfig` — used by `VmItem` to pick the
/// icon emoji + gradient theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OsFamily {
    AstraOS,
    Windows,
    Linux,
    Arm,
    Unknown,
}

impl OsFamily {
    /// Emoji rendered inside the gradient icon box.
    pub fn emoji(&self) -> &'static str {
        match self {
            OsFamily::AstraOS => "\u{1F427}", // 🐧
            OsFamily::Windows => "\u{1FA9F}", // 🪟
            OsFamily::Linux => "\u{1F4BB}",   // 💻
            OsFamily::Arm => "\u{1F4F1}",     // 📱
            OsFamily::Unknown => "\u{1F4E6}", // 📦
        }
    }

    /// CSS modifier class — selects the gradient palette for the icon.
    pub fn css_class(&self) -> &'static str {
        match self {
            OsFamily::AstraOS => "os-astra",
            OsFamily::Windows => "os-windows",
            OsFamily::Linux => "os-linux",
            OsFamily::Arm => "os-arm",
            OsFamily::Unknown => "os-unknown",
        }
    }
}

/// Top-level Astra VM configuration. Serialized to `config.toml`.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// List of VMs managed by Astra VM. May be empty on first launch.
    #[serde(default)]
    pub vms: Vec<VmConfig>,
    /// Absolute path to the QEMU portable binary, if detected.
    /// `None` until the user picks a QEMU folder or auto-detection
    /// succeeds.
    #[serde(default)]
    pub qemu_path: Option<String>,
    /// Default directory for new ISO images (under user data dir).
    #[serde(default = "default_iso_dir")]
    pub default_iso_dir: String,
}

impl AppConfig {
    /// Load the config from `~/.config/astra-vm/config.toml`.
    ///
    /// Returns a fresh default config (no VMs, no QEMU path) if the
    /// file does not exist yet (first-launch path).
    /// Returns a default config + logs an error if the file exists but
    /// fails to parse — never panics.
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            log::info!("Config file missing at {} — using defaults", path.display());
            return Self::default();
        }
        match fs::read_to_string(&path) {
            Ok(text) => match toml::from_str::<AppConfig>(&text) {
                Ok(cfg) => {
                    log::info!(
                        "Loaded config from {} ({} VMs)",
                        path.display(),
                        cfg.vms.len()
                    );
                    cfg
                }
                Err(e) => {
                    log::error!("Failed to parse {}: {}", path.display(), e);
                    Self::default()
                }
            },
            Err(e) => {
                log::error!("Failed to read {}: {}", path.display(), e);
                Self::default()
            }
        }
    }

    /// Persist the config to `~/.config/astra-vm/config.toml`.
    ///
    /// Creates the parent directory if it doesn't exist.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(&path, text)?;
        log::debug!("Saved config to {}", path.display());
        Ok(())
    }

    /// Append a VM to the list. Caller is responsible for `save()`.
    pub fn add_vm(&mut self, vm: VmConfig) {
        self.vms.push(vm);
    }

    /// Remove a VM by id. Returns `true` if a VM was removed.
    pub fn remove_vm(&mut self, id: &str) -> bool {
        let before = self.vms.len();
        self.vms.retain(|v| v.id != id);
        self.vms.len() != before
    }

    /// Find a VM by id (read-only borrow).
    pub fn get_vm(&self, id: &str) -> Option<&VmConfig> {
        self.vms.iter().find(|v| v.id == id)
    }

    /// Find a VM by id (mutable borrow) — used by subagents to update
    /// `last_used` after a launch.
    pub fn get_vm_mut(&mut self, id: &str) -> Option<&mut VmConfig> {
        self.vms.iter_mut().find(|v| v.id == id)
    }
}

/// Resolve `~/.config/astra-vm/config.toml`.
pub fn config_path() -> PathBuf {
    config_dir().join(CONFIG_SUBDIR).join(CONFIG_FILENAME)
}

/// Resolve the user's home directory.
///
/// Falls back to `/home/astra` if `HOME` is unset (defensive: AstraOS
/// sessions always set `HOME`).
pub fn home_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
    PathBuf::from(home)
}

/// Resolve `~/.config` — falls back to `/home/astra/.config` if `HOME`
/// is unset (defensive: AstraOS sessions always set `HOME`).
pub fn config_dir() -> PathBuf {
    home_dir().join(".config")
}

/// Resolve `~/.local/share` (XDG data dir).
pub fn data_dir() -> PathBuf {
    home_dir().join(".local").join("share")
}

/// Resolve the on-disk path of a VM's qcow2 disk image:
/// `~/.local/share/astra-vm/vms/{vm_id}.qcow2`.
pub fn vm_disk_path(vm_id: &str) -> PathBuf {
    vm_data_dir().join(format!("{vm_id}.qcow2"))
}

/// Current Unix timestamp in seconds — used to stamp `created_at` and
/// `last_used` on `VmConfig`. Falls back to 0 if the system clock is
/// before the Unix epoch (should never happen in practice).
pub fn now_unix_seconds() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Resolve the VM storage directory `~/.local/share/astra-vm/vms/`.
/// Creates it if missing.
pub fn vm_data_dir() -> PathBuf {
    let p = data_dir().join(VM_DATA_SUBDIR);
    let _ = fs::create_dir_all(&p);
    p
}

/// Resolve the ISO storage directory `~/.local/share/astra-vm/isos/`.
/// Creates it if missing.
pub fn iso_data_dir() -> PathBuf {
    let p = data_dir().join(ISO_DATA_SUBDIR);
    let _ = fs::create_dir_all(&p);
    p
}

/// Resolve the QEMU portable directory `~/.local/share/astra-vm/qemu/`.
/// Creates it if missing.
pub fn qemu_data_dir() -> PathBuf {
    let p = data_dir().join(QEMU_DATA_SUBDIR);
    let _ = fs::create_dir_all(&p);
    p
}

/// Generate a 32-char hex id for a new VM using `chrono`-seeded pseudo
/// randomness (sufficient for VM ids — not cryptographically unique).
/// Format: 8-4-4-4-12 (UUID-like).
fn new_vm_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Simple xorshift seeded from nanos. Cheap + deterministic enough.
    let mut state = now as u64 ^ 0x9E37_79B9_7F4A_7C15;
    let mut buf = String::with_capacity(36);
    for i in 0..32 {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let byte = ((state >> ((i % 8) * 8)) & 0xff) as u8;
        buf.push_str(&format!("{:02x}", byte));
        if i == 7 || i == 11 || i == 15 || i == 19 {
            buf.push('-');
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_config_defaults() {
        let vm = VmConfig::new("test", "/iso/x.iso", "x86_64");
        assert_eq!(vm.ram_mb, 4096);
        assert_eq!(vm.cpu_cores, 2);
        assert_eq!(vm.disk_gb, 20);
        assert_eq!(vm.architecture, "x86_64");
        assert!(vm.last_used.is_none());
        assert_eq!(vm.id.len(), 36);
        assert!(vm.id.chars().filter(|c| *c == '-').count() == 4);
    }

    #[test]
    fn os_family_detection_astra() {
        let vm = VmConfig::new("AstraOS Dev", "/iso/astra.iso", "x86_64");
        assert_eq!(vm.os_family(), OsFamily::AstraOS);
    }

    #[test]
    fn os_family_detection_windows() {
        let vm = VmConfig::new("Win11", "/iso/windows11.iso", "x86_64");
        assert_eq!(vm.os_family(), OsFamily::Windows);
    }

    #[test]
    fn os_family_detection_linux() {
        let vm = VmConfig::new("Ubuntu", "/iso/ubuntu.iso", "x86_64");
        assert_eq!(vm.os_family(), OsFamily::Linux);
    }

    #[test]
    fn os_family_detection_arm_overrides_name() {
        // aarch64 wins regardless of name match.
        let vm = VmConfig::new("Ubuntu ARM", "/iso/ubuntu.iso", "aarch64");
        assert_eq!(vm.os_family(), OsFamily::Arm);
    }

    #[test]
    fn os_family_detection_unknown() {
        let vm = VmConfig::new("Mystery", "/iso/x.iso", "x86_64");
        assert_eq!(vm.os_family(), OsFamily::Unknown);
    }

    #[test]
    fn app_config_add_remove_get() {
        let mut cfg = AppConfig::default();
        let vm = VmConfig::new("A", "/a.iso", "x86_64");
        let id = vm.id.clone();
        cfg.add_vm(vm);
        assert_eq!(cfg.vms.len(), 1);
        assert!(cfg.get_vm(&id).is_some());
        assert!(cfg.get_vm_mut(&id).is_some());
        assert!(cfg.remove_vm(&id));
        assert!(!cfg.remove_vm(&id)); // already gone
        assert!(cfg.get_vm(&id).is_none());
    }

    #[test]
    fn app_config_save_load_roundtrip() {
        let tmp = std::env::temp_dir().join("astra-vm-cfg-roundtrip.toml");
        let _ = fs::remove_file(&tmp);

        let mut cfg = AppConfig::default();
        cfg.add_vm(VmConfig::new("AstraOS", "/a.iso", "x86_64"));
        cfg.qemu_path = Some("/usr/bin/qemu-system-x86_64".to_string());

        // Save to the temp path.
        let text = toml::to_string_pretty(&cfg).unwrap();
        fs::write(&tmp, text).unwrap();

        // Read back + parse.
        let read = fs::read_to_string(&tmp).unwrap();
        let parsed: AppConfig = toml::from_str(&read).unwrap();
        assert_eq!(parsed.vms.len(), 1);
        assert_eq!(parsed.vms[0].name, "AstraOS");
        assert_eq!(
            parsed.qemu_path.as_deref(),
            Some("/usr/bin/qemu-system-x86_64")
        );

        let _ = fs::remove_file(&tmp);
    }
}
