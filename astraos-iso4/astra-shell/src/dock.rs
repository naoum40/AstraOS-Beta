// dock.rs - Astra Shell dock (Mac mode).
//
// A floating, bottom-center pill of pinned-app icons. Visible only when
// `ShellConfig::taskbar_mode == "mac"` (the bar is not built in that
// mode).
//
// ISO 2 ships a static list of pinned apps (Firefox / Files / Terminal /
// Settings). Each click spawns the underlying binary via
// `std::process::Command`. The CSS `:hover` rule grows the icon 1.4x
// and lifts it 8 px (macOS-style magnification). Active-window
// indicators (the small dot below the icon) will be wired in task 2-b
// via the Hyprland event channel.
//
// NOTE: just like the bar, the dock is a regular `ApplicationWindow`
// (`decorated = false`) for v0.2. Real layer-shell anchoring comes
// with `gtk4-layer-shell-rs` in task 2-c.

use crate::config::ShellConfig;
use crate::hyprland_ipc::HyprlandClient;
use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use gtk4::{Application, ApplicationWindow, Button, Orientation};

/// The dock shell surface.
pub struct Dock {
    window: ApplicationWindow,
}

/// A pinned application in the dock - icon glyph, human-readable name,
/// and the executable to spawn when clicked.
type PinnedApp = (&'static str, &'static str, &'static str);

/// The default pinned-app set shipped with AstraOS v0.2.
///
/// These mirror the apps preinstalled by the ISO 2 package list
/// (kitty, thunar, firefox) plus the future `astra-settings` binary
/// (placeholder for task 3-x). Emoji glyphs are used as a stand-in for
/// proper `.desktop`-derived icons - task 2-b will load real icons via
/// `gtk4::IconTheme`.
const PINNED_APPS: &[PinnedApp] = &[
    ("\u{1F310}", "Firefox", "firefox"),                // 🌐
    ("\u{1F4C1}", "Files", "thunar"),                   // 📁
    ("\u{1F4BB}", "Terminal", "kitty"),                 // 💻
    ("\u{2699}\u{FE0F}", "Settings", "astra-settings"), // ⚙️
];

impl Dock {
    /// Build the dock. Same ownership model as `Bar` - the window is
    /// owned by the `Application` and stays alive after this function
    /// returns.
    pub fn new(app: &Application, _config: &ShellConfig, _hyprland: &HyprlandClient) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Dock")
            .default_width(400)
            .default_height(64)
            .decorated(false)
            .build();

        window.add_css_class("astra-dock");

        // Horizontal pill container, centered (floating bottom-center).
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::End)
            .build();
        window.set_child(Some(&container));

        // Build a button per pinned app. Each closure captures the
        // executable name (a `String` so the closure is `'static`).
        for (icon, tooltip, exec) in PINNED_APPS {
            let btn = Button::builder()
                .label(*icon)
                .tooltip_text(*tooltip)
                .css_classes(["astra-button", "astra-dock-item"])
                .build();

            let exec_cmd = exec.to_string();
            btn.connect_clicked(move |_| {
                log::info!("Launching: {}", exec_cmd);
                // `spawn()` is non-blocking; we drop the `Child` handle
                // immediately - the app runs independently of the shell.
                match std::process::Command::new(&exec_cmd).spawn() {
                    Ok(_) => log::debug!("Spawned {} OK", exec_cmd),
                    Err(e) => log::warn!("Failed to spawn {}: {}", exec_cmd, e),
                }
            });

            container.append(&btn);
        }

        Self { window }
    }

    /// Show the dock window.
    pub fn present(&self) {
        self.window.present();
    }
}
