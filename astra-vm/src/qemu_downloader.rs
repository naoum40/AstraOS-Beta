//! Auto-download and locate the QEMU portable binary.

use std::path::PathBuf;

/// Errors raised while locating or downloading QEMU.
#[derive(Debug)]
pub enum QemuDownloadError {
    /// HTTP or network failure while downloading.
    DownloadFailed(String),
    /// Failed to extract the downloaded archive.
    ExtractionFailed(String),
    /// Platform not supported by the auto-downloader.
    PlatformUnsupported,
}

/// Where Astra VM stores its bundled QEMU portable.
fn astra_vm_qemu_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
    PathBuf::from(home)
        .join(".local/share/astra-vm/qemu")
}

/// Names of the QEMU binaries we look for, per architecture.
fn qemu_binary_name(arch: &str) -> &'static str {
    match arch {
        "aarch64" => "qemu-system-aarch64",
        _ => "qemu-system-x86_64",
    }
}

/// Ensure QEMU is available. Checks:
/// 1. `~/.local/share/astra-vm/qemu/qemu-system-x86_64`
/// 2. `qemu-system-x86_64` in `$PATH`
///
/// If not found, returns an error instructing the user to download QEMU
/// portable manually. The auto-download path is a placeholder.
pub fn ensure_qemu_installed() -> Result<PathBuf, QemuDownloadError> {
    let bin_name = qemu_binary_name("x86_64");

    // 1. Check the bundled QEMU directory.
    let bundled = astra_vm_qemu_dir().join(bin_name);
    if bundled.exists() {
        log::info!("Found bundled QEMU at {:?}", bundled);
        return Ok(bundled);
    }

    // 2. Check the system PATH.
    if let Ok(path) = which::which(bin_name) {
        log::info!("Found system QEMU at {:?}", path);
        return Ok(path);
    }

    Err(QemuDownloadError::DownloadFailed(
        "QEMU not installed — please download QEMU portable and extract to ~/.local/share/astra-vm/qemu/".to_string(),
    ))
}

/// Download a file with progress reporting.
///
/// This is a placeholder — the real implementation would use `reqwest` or
/// `ureq` to stream the file to disk while invoking `progress_callback` with
/// bytes downloaded / total bytes.
pub fn download_with_progress(
    _url: &str,
    _dest: &Path,
    _progress_callback: impl Fn(u64, u64),
) -> Result<(), QemuDownloadError> {
    log::info!("TODO: implement HTTP download with reqwest/ureq");
    Err(QemuDownloadError::DownloadFailed(
        "download_with_progress not yet implemented".to_string(),
    ))
}

/// Extract a ZIP archive to a directory.
///
/// Placeholder — would use the `zip` crate in production.
pub fn extract_zip(_zip_path: &Path, _dest_dir: &Path) -> Result<(), QemuDownloadError> {
    log::info!("TODO: implement ZIP extraction with `zip` crate");
    Err(QemuDownloadError::ExtractionFailed(
        "extract_zip not yet implemented".to_string(),
    ))
}
