// qemu.rs - QEMU binary detection and process spawning.
//
// Resolution order for `detect_qemu()`:
//   1. Portable install: ~/.local/share/astra-vm/qemu/qemu-system-x86_64
//   2. System binary `qemu-system-x86_64` in $PATH
//   3. Error `QemuError::NotFound`
//
// `build_qemu_args()` produces the argument vector matching the AstraVM
// defaults: virtio disk + virtio-gpu, KVM + host CPU when /dev/kvm is
// available, `-boot d` so a CD (ISO) takes priority over the disk,
// and `-display gtk` by default. Headless mode is available via
// `build_qemu_args_with_headless(vm, disk_path, true)`.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use thiserror::Error;

use crate::config::{qemu_data_dir, VmConfig};

/// A resolved QEMU binary plus its `--version` banner.
#[derive(Debug, Clone)]
pub struct QemuBinary {
    /// Filesystem path (absolute for portable, bare name for PATH lookup).
    pub path: PathBuf,
    /// First line of `qemu-system-* --version` output.
    pub version: String,
}

impl QemuBinary {
    /// Resolve the architecture-specific QEMU binary sibling of `self`.
    ///
    /// If `self.path` is a portable install inside a directory, look for
    /// `qemu-system-{arch}` in the same directory. Otherwise fall back to
    /// a bare name and rely on $PATH lookup.
    pub fn for_arch(&self, arch: &str) -> PathBuf {
        let name = arch_binary_name(arch);
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                let candidate = parent.join(name);
                if candidate.exists() {
                    return candidate;
                }
            }
        }
        PathBuf::from(name)
    }
}

/// Errors that can occur while detecting or spawning QEMU.
#[derive(Debug, Error)]
pub enum QemuError {
    #[error("QEMU binary not found (looked in ~/.local/share/astra-vm/qemu/ and $PATH)")]
    NotFound,
    #[error("permission denied for QEMU binary")]
    PermissionDenied,
    #[error("failed to spawn QEMU: {0}")]
    SpawnFailed(String),
}

/// Map an architecture string to the QEMU binary basename.
///
/// * `"x86_64"` / `"amd64"` -> `qemu-system-x86_64`
/// * `"aarch64"` / `"arm64"` -> `qemu-system-aarch64`
fn arch_binary_name(arch: &str) -> &'static str {
    match arch {
        "aarch64" | "arm64" => "qemu-system-aarch64",
        _ => "qemu-system-x86_64",
    }
}

/// Public helper: returns the bare QEMU binary name for an architecture.
pub fn qemu_arch_binary(arch: &str) -> PathBuf {
    PathBuf::from(arch_binary_name(arch))
}

/// Probe for the QEMU binary, preferring the portable install.
pub fn detect_qemu() -> Result<QemuBinary, QemuError> {
    // 1. Portable install at ~/.local/share/astra-vm/qemu/qemu-system-x86_64
    let portable = qemu_data_dir().join("qemu-system-x86_64");
    if portable.is_file() {
        check_executable(&portable)?;
        return Ok(QemuBinary {
            path: portable.clone(),
            version: detect_version(&portable),
        });
    }

    // 2. System binary in $PATH
    if let Ok(output) = Command::new("qemu-system-x86_64").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("qemu-system-x86_64")
                .to_string();
            return Ok(QemuBinary {
                path: PathBuf::from("qemu-system-x86_64"),
                version,
            });
        }
    }

    Err(QemuError::NotFound)
}

/// Verify the binary is executable by the current user.
fn check_executable(path: &Path) -> Result<(), QemuError> {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(meta) => {
            if meta.permissions().mode() & 0o111 == 0 {
                return Err(QemuError::PermissionDenied);
            }
            Ok(())
        }
        Err(_) => Err(QemuError::NotFound),
    }
}

/// Run `binary --version` and return the first line (or "unknown").
fn detect_version(binary: &Path) -> String {
    Command::new(binary)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout).lines().next().map(str::to_string)
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// True when /dev/kvm exists and is a character device (Linux + virtualisation
/// extensions available). Always false on non-Linux targets.
pub fn kvm_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::FileTypeExt;
        match std::fs::metadata("/dev/kvm") {
            Ok(meta) => meta.file_type().is_char_device(),
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Build the QEMU argument vector for a given VM config + disk path.
///
/// Defaults to non-headless (`-display gtk`). Use
/// [`build_qemu_args_with_headless`] to control the display mode.
///
/// Argument order matches the AstraVM defaults:
///   -m ram  -smp cores  [-cdrom iso]  -drive file=...,format=qcow2,if=virtio
///   -boot d  -display gtk|none  [-enable-kvm -cpu host]  -vga virtio
pub fn build_qemu_args(vm: &VmConfig, disk_path: &Path) -> Vec<String> {
    build_qemu_args_with_headless(vm, disk_path, false)
}

/// Same as [`build_qemu_args`] but lets the caller pick headless mode
/// (`-display none`) — useful for VMs launched from the tray icon or
/// background services that don't need a GTK window.
pub fn build_qemu_args_with_headless(vm: &VmConfig, disk_path: &Path, headless: bool) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    // Memory (MB)
    args.push("-m".into());
    args.push(vm.ram_mb.to_string());

    // SMP cores
    args.push("-smp".into());
    args.push(vm.cpu_cores.to_string());

    // CD-ROM (only if the ISO path is set + the file actually exists)
    if !vm.iso_path.is_empty() && Path::new(&vm.iso_path).is_file() {
        args.push("-cdrom".into());
        args.push(vm.iso_path.clone());
    }

    // Primary disk — virtio for performance
    args.push("-drive".into());
    args.push(format!(
        "file={},format=qcow2,if=virtio",
        disk_path.to_string_lossy()
    ));

    // Boot from CD first, fall back to disk
    args.push("-boot".into());
    args.push("d".into());

    // Display
    if headless {
        args.push("-display".into());
        args.push("none".into());
    } else {
        args.push("-display".into());
        args.push("gtk".into());
    }

    // KVM + host CPU passthrough when /dev/kvm is available
    if kvm_available() {
        args.push("-enable-kvm".into());
        args.push("-cpu".into());
        args.push("host".into());
    }

    // virtio-gpu for better graphics acceleration
    args.push("-vga".into());
    args.push("virtio".into());

    args
}

/// Spawn the QEMU process with the given args. Returns the child handle.
///
/// The caller is responsible for tracking the PID (via `child.id()`) and
/// reaping the process. `stdin`/`stdout`/`stderr` are detached to avoid
/// blocking the UI thread on QEMU's console output.
pub fn spawn_qemu(args: Vec<String>, binary: &Path) -> Result<Child, QemuError> {
    Command::new(binary)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => QemuError::NotFound,
            std::io::ErrorKind::PermissionDenied => QemuError::PermissionDenied,
            _ => QemuError::SpawnFailed(e.to_string()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn sample_vm() -> VmConfig {
        VmConfig {
            id: "test-vm".to_string(),
            name: "Test VM".to_string(),
            iso_path: String::new(),
            architecture: "x86_64".to_string(),
            ram_mb: 1024,
            cpu_cores: 1,
            disk_gb: 10,
            created_at: 0,
            last_used: None,
        }
    }

    #[test]
    fn arch_binary_name_mapping() {
        assert_eq!(arch_binary_name("x86_64"), "qemu-system-x86_64");
        assert_eq!(arch_binary_name("amd64"), "qemu-system-x86_64");
        assert_eq!(arch_binary_name("aarch64"), "qemu-system-aarch64");
        assert_eq!(arch_binary_name("arm64"), "qemu-system-aarch64");
        // Unknown arch defaults to x86_64 binary.
        assert_eq!(arch_binary_name("riscv64"), "qemu-system-x86_64");
    }

    #[test]
    fn qemu_arch_binary_returns_bare_path() {
        assert_eq!(qemu_arch_binary("x86_64"), PathBuf::from("qemu-system-x86_64"));
        assert_eq!(
            qemu_arch_binary("aarch64"),
            PathBuf::from("qemu-system-aarch64")
        );
    }

    #[test]
    fn build_args_minimal_vm() {
        let vm = sample_vm();
        let args = build_qemu_args(&vm, Path::new("/disks/test.qcow2"));

        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"1024".to_string()));
        assert!(args.contains(&"-smp".to_string()));
        assert!(args.contains(&"1".to_string()));
        assert!(args.contains(&"-boot".to_string()));
        assert!(args.contains(&"d".to_string()));
        assert!(args.contains(&"-display".to_string()));
        assert!(args.contains(&"gtk".to_string()));
        assert!(args.contains(&"-vga".to_string()));
        assert!(args.contains(&"virtio".to_string()));
        assert!(args
            .iter()
            .any(|a| a.starts_with("file=/disks/test.qcow2,format=qcow2,if=virtio")));
    }

    #[test]
    fn build_args_headless_uses_display_none() {
        let vm = sample_vm();
        let args = build_qemu_args_with_headless(&vm, Path::new("/d.qcow2"), true);
        let display_idx = args.iter().position(|a| a == "-display").unwrap();
        assert_eq!(args[display_idx + 1], "none");
    }

    #[test]
    fn build_args_skips_cdrom_when_iso_missing() {
        let mut vm = sample_vm();
        vm.iso_path = "/no/such/iso.iso".to_string();
        let args = build_qemu_args(&vm, Path::new("/d.qcow2"));
        assert!(!args.iter().any(|a| a == "-cdrom"));
    }

    #[test]
    fn for_arch_falls_back_to_bare_name_when_sibling_missing() {
        let qb = QemuBinary {
            path: PathBuf::from("qemu-system-x86_64"),
            version: String::new(),
        };
        // No parent dir → falls back to bare name.
        assert_eq!(qb.for_arch("aarch64"), PathBuf::from("qemu-system-aarch64"));
    }
}
