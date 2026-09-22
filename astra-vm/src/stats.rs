//! Live VM stats monitoring.
//!
//! Polls QMP once per call to compute CPU/RAM usage and uptime.

use std::time::Instant;

use crate::qmp::{QmpClient, QmpError};

/// Snapshot of VM resource usage at a point in time.
#[derive(Debug, Clone, Default)]
pub struct VmStats {
    pub cpu_percent: u32,
    pub ram_used_mb: u32,
    pub ram_total_mb: u32,
    pub disk_io_mbps: u32,
    pub uptime_seconds: u64,
}

impl VmStats {
    /// CSS-friendly color token based on CPU usage.
    pub fn cpu_color(&self) -> &'static str {
        if self.cpu_percent > 70 {
            "red"
        } else if self.cpu_percent > 40 {
            "yellow"
        } else {
            "green"
        }
    }
}

/// Polls QMP for VM stats and tracks uptime.
pub struct StatsMonitor {
    qmp: QmpClient,
    uptime_start: Instant,
}

impl StatsMonitor {
    /// Connect to the QMP socket at `socket_path` and start the uptime clock.
    pub fn new(socket_path: std::path::PathBuf) -> Result<Self, QmpError> {
        let qmp = QmpClient::connect(socket_path)?;
        Ok(Self {
            qmp,
            uptime_start: Instant::now(),
        })
    }

    /// Poll QMP once and return the current stats.
    pub fn get_stats(&mut self) -> Result<VmStats, QmpError> {
        let cpu = self.qmp.query_cpu()?;
        let balloon = self.qmp.query_balloon()?;
        let uptime = self.uptime_start.elapsed().as_secs();

        Ok(VmStats {
            cpu_percent: cpu.usage_percent,
            ram_used_mb: balloon.actual_mb,
            ram_total_mb: balloon.total_mb,
            disk_io_mbps: 0, // Not directly exposed by QMP; placeholder.
            uptime_seconds: uptime,
        })
    }
}
