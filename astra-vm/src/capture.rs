//! VM screenshot capture via QMP `screendump`.

use std::path::Path;

use crate::qmp::{QmpClient, QmpError};

/// Errors raised by capture operations.
#[derive(Debug)]
pub enum CaptureError {
    /// Underlying QMP error.
    Qmp(QmpError),
    /// Failed to convert/rename the captured image.
    ConversionFailed,
}

/// Capture the VM screen to `output_path` (PPM format by default).
pub fn capture_vm_screen(
    qmp: &mut QmpClient,
    output_path: &Path,
) -> Result<(), CaptureError> {
    let filename = output_path
        .to_str()
        .ok_or(CaptureError::ConversionFailed)?;
    qmp.screendump(filename).map_err(CaptureError::Qmp)
}

/// Convert (or rename) a PPM capture to PNG.
///
/// For now this is a simple rename; a real implementation would decode the
/// PPM and re-encode as PNG using the `image` crate.
pub fn convert_ppm_to_png(ppm_path: &Path, png_path: &Path) -> Result<(), CaptureError> {
    log::info!(
        "TODO: real PPM→PNG conversion (would use `image` crate) — renaming {:?} → {:?} for now",
        ppm_path,
        png_path
    );
    std::fs::rename(ppm_path, png_path).map_err(|_| CaptureError::ConversionFailed)
}
