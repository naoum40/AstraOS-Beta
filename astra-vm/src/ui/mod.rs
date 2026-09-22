// ui/mod.rs — UI module barrel.
//
// One module per major UI region, declared here so `main.rs` can pull
// them in with a single `mod ui;`.
pub mod header;
pub mod main_area;
pub mod sidebar;
pub mod vm_item;
pub mod new_vm_modal;
