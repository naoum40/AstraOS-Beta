// disk.rs - qcow2 disk image management.
//
// Wraps `qemu-img` to create / delete virtual disks. The qemu-img binary
// is resolved with the same portable-first strategy as `qemu::detect_qemu`:
//   1. ~/.local/share/astra-vm/qemu/qemu-img
//   2. system `qemu-img` in $PATH
//
// `get_disk_usage()` returns the actual size on disk (not the virtual
// capacity) — useful for the UI to show how much space a VM is consuming.

use std::fs;
use std::path::Path;
use std::process::Command;

use thiserror::Error;

use crate::config::qemu_data_dir;

/// Errors that can occur during disk operations.
#[derive(Debug, Error)]
pub enum DiskError {
    #[error("qemu-img command failed: {0}")]
    QemuImgFailed(String),
    #[error("disk file not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for DiskError {
    fn from(e: std::io::Error) -> Self {
        DiskError::IoError(e.to_string())
    }
}

/// Resolve the qemu-img binary path (portable first, then $PATH).
fn qemu_img_binary() -> String {
    let portable = qemu_data_dir().join("qemu-img");
    if portable.is_file() {
        portable.to_string_lossy().into_owned()
    } else {
        "qemu-img".to_string()
    }
}

/// Create a new qcow2 disk image of the given size (in gigabytes).
///
/// Runs: `qemu-img create -f qcow2 <path> <size_gb>G`
///
/// Parent directories are created on demand.
pub fn create_disk(path: &Path, size_gb: u32) -> Result<(), DiskError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let output = Command::new(qemu_img_binary())
        .arg("create")
        .arg("-f")
        .arg("qcow2")
        .arg(path)
        .arg(format!("{}G", size_gb))
        .output()
        .map_err(|e| DiskError::QemuImgFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DiskError::QemuImgFailed(stderr));
    }
    Ok(())
}

/// Delete a disk image file. Returns `NotFound` if the file doesn't exist.
pub fn delete_disk(path: &Path) -> Result<(), DiskError> {
    if !path.exists() {
        return Err(DiskError::NotFound(path.to_string_lossy().into_owned()));
    }
    fs::remove_file(path).map_err(|e| DiskError::IoError(e.to_string()))?;
    Ok(())
}

/// Return the actual size on disk (in bytes) of the qcow2 file.
///
/// This is *not* the virtual capacity reported by `qemu-img info` — it's
/// the filesystem block count. For sparse qcow2 files, this is typically
/// much smaller than the virtual size.
pub fn get_disk_usage(path: &Path) -> Result<u64, DiskError> {
    if !path.exists() {
        return Err(DiskError::NotFound(path.to_string_lossy().into_owned()));
    }
    let meta = fs::metadata(path)?;
    Ok(meta.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_missing_disk_returns_not_found() {
        let bogus = Path::new("/tmp/astra-vm-never-exists-12345.qcow2");
        match delete_disk(bogus) {
            Err(DiskError::NotFound(_)) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn get_disk_usage_missing_returns_not_found() {
        let bogus = Path::new("/tmp/astra-vm-never-exists-12345.qcow2");
        match get_disk_usage(bogus) {
            Err(DiskError::NotFound(_)) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }
}
