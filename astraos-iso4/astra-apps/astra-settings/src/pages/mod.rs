// pages/mod.rs — Astra Settings page modules.
//
// The settings window is split into:
//   * `sidebar`   — left nav rail with 4 buttons (Account / Personalization
//                   / System / About).
//   * `content`   — right-side `gtk4::Stack` switching between the 4 pages.
//   * `account`   — avatar placeholder + "Change password" stub.
//   * `personalization` — theme / accent / wallpaper / taskbar / glass.
//   * `system`    — performance / battery / storage / Defender status.
//   * `about`     — AstraOS version + license + repo link.

pub mod about;
pub mod account;
pub mod content;
pub mod personalization;
pub mod sidebar;
pub mod system;
