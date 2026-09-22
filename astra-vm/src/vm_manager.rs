// vm_manager.rs - VM lifecycle management.
//
// `VmManager` is the high-level orchestrator the UI talks to. It owns:
//   * `AppConfig`    — the persisted TOML (registered VMs + paths)
//   * `QemuBinary`   — the detected QEMU install
//   * running PIDs   — a `RefCell<HashMap<vm_id, pid>>` so that `start_vm`
//                      can stay `&self` (UI callbacks rarely have &mut).
//
// Lifecycle:
//   create_vm()  -> allocate disk, register, persist  -> returns disk path
//   start_vm()   -> build args, spawn QEMU process    -> returns PID
//   stop_vm()    -> SIGTERM the process by PID
//   delete_vm()  -> stop if running, remove disk + config entry, persist

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use thiserror::Error;

use crate::config::{vm_data_dir, AppConfig, VmConfig};
use crate::disk::{self, DiskError};
use crate::qemu::{self, QemuBinary, QemuError};

/// Errors raised by VM lifecycle operations.
#[derive(Debug, Error)]
pub enum VmError {
    #[error("VM not found: {0}")]
    NotFound(String),
    #[error(transparent)]
    QemuError(#[from] QemuError),
    #[error("disk error: {0}")]
    DiskError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

impl From<DiskError> for VmError {
    fn from(e: DiskError) -> Self {
        VmError::DiskError(e.to_string())
    }
}

impl From<std::io::Error> for VmError {
    fn from(e: std::io::Error) -> Self {
        VmError::IoError(e.to_string())
    }
}

/// Convert the `Box<dyn std::error::Error>` returned by `AppConfig::save`
/// into a `VmError`. Kept as a free function so the call sites stay tidy.
fn save_err(e: Box<dyn std::error::Error>) -> VmError {
    VmError::IoError(e.to_string())
}

/// High-level VM manager. Owns config + QEMU binary handle + runtime PID map.
pub struct VmManager {
    /// Persisted application config (VM registry + paths).
    pub config: AppConfig,
    /// Detected QEMU binary (portable or system).
    pub qemu: QemuBinary,
    /// Currently-running VM PIDs, keyed by vm_id. Interior mutability lets
    /// `start_vm`/`stop_vm` take `&self` so UI callbacks can call them.
    running: RefCell<HashMap<String, u32>>,
}

impl VmManager {
    /// Build a new manager: detect QEMU, ensure required directories exist.
    pub fn new(config: AppConfig) -> Result<Self, QemuError> {
        let qemu = qemu::detect_qemu()?;
        // `vm_data_dir()` auto-creates the directory on first call.
        let _ = vm_data_dir();
        Ok(Self {
            config,
            qemu,
            running: RefCell::new(HashMap::new()),
        })
    }

    /// Resolve the qcow2 disk path for a VM id under the standard storage
    /// directory (`~/.local/share/astra-vm/vms/{vm_id}.qcow2`).
    fn disk_path_for(vm_id: &str) -> PathBuf {
        vm_data_dir().join(format!("{}.qcow2", vm_id))
    }

    /// Register a new VM: create its qcow2 disk, persist config, return
    /// the disk path.
    pub fn create_vm(&mut self, vm: VmConfig) -> Result<PathBuf, VmError> {
        let disk_path = Self::disk_path_for(&vm.id);
        disk::create_disk(&disk_path, vm.disk_gb)?;
        self.config.add_vm(vm);
        self.config.save().map_err(save_err)?;
        Ok(disk_path)
    }

    /// Start a registered VM. Returns the QEMU process PID on success.
    ///
    /// The process is detached from this manager (we only remember the PID
    /// so `delete_vm` can stop it). The caller may also call `stop_vm(pid)`
    /// directly with the returned PID.
    pub fn start_vm(&self, vm_id: &str) -> Result<u32, VmError> {
        let vm = self
            .config
            .get_vm(vm_id)
            .ok_or_else(|| VmError::NotFound(vm_id.to_string()))?
            .clone();

        let disk_path = Self::disk_path_for(vm_id);
        if !disk_path.exists() {
            return Err(VmError::IoError(format!(
                "disk image missing for VM {vm_id}: {}",
                disk_path.display()
            )));
        }

        let args = qemu::build_qemu_args(&vm, &disk_path);
        let binary_path = self.qemu.for_arch(&vm.architecture);
        let mut child = qemu::spawn_qemu(args, &binary_path)?;
        let pid = child.id();
        self.running.borrow_mut().insert(vm_id.to_string(), pid);

        // Drop the Child handle: on Unix the process keeps running. Zombies
        // are reaped by init when our process exits, or by an explicit
        // `stop_vm()` + waitpid below.
        drop(child);

        Ok(pid)
    }

    /// Send SIGTERM to a running QEMU process by PID.
    ///
    /// Spawns a short-lived thread that calls `waitpid(pid)` to reap the
    /// resulting zombie, since we no longer own the `Child` handle after
    /// `start_vm()` returned.
    pub fn stop_vm(&self, pid: u32) -> Result<(), VmError> {
        let rc = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        if rc != 0 {
            return Err(VmError::IoError(format!(
                "failed to signal pid {pid}: {}",
                std::io::Error::last_os_error()
            )));
        }
        // Reap the zombie in the background (the process was our child
        // originally, so waitpid is valid).
        let pid_to_reap = pid as libc::pid_t;
        std::thread::spawn(move || {
            unsafe {
                let mut status: libc::c_int = 0;
                // Loop in case waitpid is interrupted by a signal.
                while libc::waitpid(pid_to_reap, &mut status as *mut libc::c_int, 0) == -1 {
                    let err = std::io::Error::last_os_error();
                    if err.raw_os_error() == Some(libc::EINTR) {
                        continue;
                    }
                    break;
                }
            }
        });
        self.running.borrow_mut().retain(|_, &mut p| p != pid);
        Ok(())
    }

    /// Delete a VM: stop if running, remove disk, drop config entry, persist.
    pub fn delete_vm(&mut self, vm_id: &str) -> Result<(), VmError> {
        // Stop if currently running.
        if let Some(&pid) = self.running.borrow().get(vm_id).copied() {
            let _ = self.stop_vm(pid);
        }

        // Resolve disk path before removing the config entry.
        let disk_path = Self::disk_path_for(vm_id);

        // Remove from config + persist (fail fast if save errors).
        if !self.config.remove_vm(vm_id) {
            return Err(VmError::NotFound(vm_id.to_string()));
        }
        self.config.save().map_err(save_err)?;

        // Delete the disk file (ignore NotFound — disk may have been
        // removed out-of-band).
        if disk_path.exists() {
            if let Err(e) = disk::delete_disk(&disk_path) {
                log::warn!("VM {vm_id} deregistered but disk cleanup failed: {e}");
            }
        }

        self.running.borrow_mut().remove(vm_id);
        Ok(())
    }

    /// True if a VM is currently running (tracked by `start_vm`).
    pub fn is_running(&self, vm_id: &str) -> bool {
        self.running.borrow().contains_key(vm_id)
    }

    /// Snapshot of the running PIDs map (vm_id -> pid).
    pub fn running_vms(&self) -> HashMap<String, u32> {
        self.running.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_manager() -> VmManager {
        VmManager {
            config: AppConfig::default(),
            qemu: QemuBinary {
                path: PathBuf::from("qemu-system-x86_64"),
                version: String::new(),
            },
            running: RefCell::new(HashMap::new()),
        }
    }

    #[test]
    fn delete_unknown_vm_returns_not_found() {
        let mut m = stub_manager();
        match m.delete_vm("does-not-exist") {
            Err(VmError::NotFound(id)) => assert_eq!(id, "does-not-exist"),
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn start_unknown_vm_returns_not_found() {
        let m = stub_manager();
        match m.start_vm("missing") {
            Err(VmError::NotFound(id)) => assert_eq!(id, "missing"),
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn is_running_false_for_unknown_vm() {
        let m = stub_manager();
        assert!(!m.is_running("nope"));
    }

    #[test]
    fn running_vms_starts_empty() {
        let m = stub_manager();
        assert!(m.running_vms().is_empty());
    }
}
