// ui/mod.rs — UI module barrel.
//
// AstraPass follows the Astra Settings layout convention: one module
// per major UI region, all declared here so `main.rs` can pull them in
// with a single `mod ui;`.
pub mod detail;
pub mod list;
pub mod setup;
pub mod sidebar;
pub mod unlock;
