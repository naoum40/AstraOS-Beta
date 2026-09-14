// lib.rs - AstraVM backend library root.
//
// !!! DO NOT DELETE !!! This file is referenced by `[lib] path = "src/lib.rs"`
// in `Cargo.toml`. Removing it breaks `cargo check` / `cargo build` for the
// whole package.
//
// The library exposes the QEMU wrapper, VM lifecycle manager and disk
// operations as a reusable library consumed by the GTK4 binaries
// (`src/main.rs`, `src/main_wire.rs`) via `use astra_vm::backend::*`.
// The UI layer is wired in by subagent vm-d.
//
// The library is independent of GTK4 system libraries so it can be
// `cargo check`-ed on hosts without `gtk4.pc` installed.
//
// QEMU portable layout (no admin required):
//   ~/.local/share/astra-vm/qemu/qemu-system-x86_64
//   ~/.local/share/astra-vm/qemu/qemu-img
//
// Config file:  ~/.config/astra-vm/config.toml
// Disk images:  ~/.local/share/astra-vm/vms/{vm_id}.qcow2
//
// Module map:
//   config      -> VmConfig + AppConfig (shared with the GTK4 binary)
//   qemu        -> binary detection, arg builder, process spawn
//   disk        -> qcow2 create / delete / usage
//   vm_manager  -> VmManager orchestrating the above
//   backend     -> aggregator re-exporting the public API

pub mod backend;
pub mod config;
pub mod disk;
pub mod qemu;
pub mod vm_manager;
