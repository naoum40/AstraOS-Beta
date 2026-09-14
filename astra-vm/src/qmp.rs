//! QEMU Machine Protocol (QMP) client.
//!
//! Communicates with a running QEMU instance via its Unix socket
//! to query status, CPU/RAM stats, capture screenshots, and shut down.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

/// QMP errors that can occur while talking to a running QEMU.
#[derive(Debug)]
pub enum QmpError {
    /// Could not connect to the QMP socket.
    ConnectionFailed,
    /// JSON parse error or malformed QMP response.
    ParseError,
    /// QEMU rejected the command or returned an error.
    CommandFailed(String),
}

/// CPU usage stats reported by QMP.
#[derive(Debug, Clone)]
pub struct CpuStats {
    pub usage_percent: u32,
}

/// RAM (balloon) stats reported by QMP.
#[derive(Debug, Clone)]
pub struct BalloonStats {
    pub actual_mb: u32,
    pub total_mb: u32,
}

/// QMP client over a Unix domain socket.
pub struct QmpClient {
    socket_path: PathBuf,
}

impl QmpClient {
    /// Open a connection to the QMP socket at `socket_path`.
    pub fn connect(socket_path: PathBuf) -> Result<Self, QmpError> {
        // Verify we can at least reach the socket file.
        if !socket_path.exists() {
            return Err(QmpError::ConnectionFailed);
        }
        Ok(Self { socket_path })
    }

    /// Path of the QMP socket.
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Execute a QMP command and return the parsed JSON response.
    ///
    /// Opens a fresh connection per call (QMP is line-based JSON-RPC).
    pub fn execute(
        &mut self,
        command: &str,
        args: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, QmpError> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|_| QmpError::ConnectionFailed)?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|_| QmpError::ConnectionFailed)?;

        let mut reader = BufReader::new(stream.try_clone().map_err(|_| QmpError::ConnectionFailed)?);

        // 1) Read the QMP greeting line.
        let mut greeting = String::new();
        reader
            .read_line(&mut greeting)
            .map_err(|_| QmpError::ParseError)?;

        // 2) Send `qmp_capabilities` to enter command mode.
        let capabilities_cmd = serde_json::json!({
            "execute": "qmp_capabilities"
        });
        let capabilities_line = format!("{}\n", capabilities_cmd);
        stream
            .write_all(capabilities_line.as_bytes())
            .map_err(|_| QmpError::ConnectionFailed)?;

        let mut cap_response = String::new();
        reader
            .read_line(&mut cap_response)
            .map_err(|_| QmpError::ParseError)?;

        // 3) Build the real command.
        let mut cmd_obj = serde_json::json!({ "execute": command });
        if let Some(args_obj) = args {
            cmd_obj["arguments"] = args_obj;
        }
        let cmd_line = format!("{}\n", cmd_obj);
        stream
            .write_all(cmd_line.as_bytes())
            .map_err(|_| QmpError::ConnectionFailed)?;

        // 4) Read response lines, skipping async events (start with "{"event":).
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).map_err(|_| QmpError::ParseError)? == 0 {
                return Err(QmpError::ConnectionFailed); // EOF
            }
            let value: serde_json::Value =
                serde_json::from_str(line.trim()).map_err(|_| QmpError::ParseError)?;
            if value.get("event").is_some() {
                // Skip async events.
                continue;
            }
            if let Some(err) = value.get("error") {
                let msg = err
                    .get("desc")
                    .and_then(|d| d.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                return Err(QmpError::CommandFailed(msg));
            }
            return Ok(value);
        }
    }

    /// Query the VM run state ("running", "paused", "shutdown", etc.).
    pub fn query_status(&mut self) -> Result<String, QmpError> {
        let resp = self.execute("query-status", None)?;
        let status = resp
            .get("return")
            .and_then(|r| r.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        Ok(status)
    }

    /// Query CPU usage (placeholder: returns 0% — real impl needs a
    /// CPU-usage aggregator; QMP doesn't expose it directly).
    pub fn query_cpu(&mut self) -> Result<CpuStats, QmpError> {
        // QMP doesn't expose CPU usage directly. A real impl would poll
        // query-cpus and aggregate deltas; for now we return 0%.
        let _ = self.execute("query-cpus-fast", None)?;
        Ok(CpuStats { usage_percent: 0 })
    }

    /// Query RAM usage via the balloon driver.
    pub fn query_balloon(&mut self) -> Result<BalloonStats, QmpError> {
        let resp = self.execute("query-balloon", None)?;
        let actual_bytes = resp
            .get("return")
            .and_then(|r| r.get("actual"))
            .and_then(|a| a.as_i64())
            .unwrap_or(0);
        let actual_mb = (actual_bytes / 1_048_576) as u32;
        Ok(BalloonStats {
            actual_mb,
            total_mb: actual_mb,
        })
    }

    /// Capture the VM screen to `filename` (PPM format by default).
    pub fn screendump(&mut self, filename: &str) -> Result<(), QmpError> {
        let args = serde_json::json!({ "filename": filename });
        self.execute("screendump", Some(args))?;
        Ok(())
    }

    /// Tell QEMU to quit gracefully.
    pub fn quit(&mut self) -> Result<(), QmpError> {
        self.execute("quit", None)?;
        Ok(())
    }
}
