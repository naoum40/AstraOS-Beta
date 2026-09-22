// Stub wired binary — task vm-d replaces this with the integrated
// Astra VM entry point that pulls in `astra_vm::backend::*` +
// `astra_vm::backend_stats::*` + the App controller from `crate::app`.
//
// The stub exists so `cargo check --features gtk` succeeds even before
// vm-d delivers the full wiring.

fn main() {
    eprintln!(
        "astra-vm-wire: stub binary. Task vm-d replaces src/main_wire.rs \
         with the integrated GTK4 + backend wiring."
    );
}
