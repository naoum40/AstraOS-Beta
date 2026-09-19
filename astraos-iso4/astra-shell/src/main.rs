// main.rs - Astra Shell entry point.
//
// Boots the AstraOS desktop shell on top of Hyprland (Wayland). Owns the
// GTK4 `Application` and orchestrates the bar / dock / launcher plus the
// Hyprland IPC event listener.
//
// Pipeline:
//   1. `env_logger::init()` - stderr logging (RUST_LOG=info).
//   2. `gtk4::init()` - must run on the GTK main thread.
//   3. Build the `Application` (GApplication ID `org.astraos.Shell`).
//   4. On activate: load CSS theme, load user config, start the Hyprland
//      IPC listener, build EITHER the bar (Windows mode) OR the dock
//      (Mac mode), then build the hidden launcher.

mod bar;
mod config;
mod dock;
mod hyprland_ipc;
mod launcher;
mod screens;
mod theme;

use gtk4::prelude::*;
use gtk4::Application;

/// GApplication ID - registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.Shell";

fn main() {
    env_logger::init();
    log::info!("AstraOS Shell v0.2.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        // 1. Load CSS theme (glassmorphism) onto the default display.
        theme::load_theme();

        // 2. Load user config from ~/.config/astra/desktop.toml
        //    (falls back to a sensible AstraOS default if missing).
        let cfg = config::ShellConfig::load();
        log::info!("Config: {:?}", cfg);

        // 3. Connect to Hyprland's event socket (.socket2.sock) and start
        //    the async event listener. The listener runs on its own
        //    dedicated OS thread with a private Tokio runtime so the
        //    GTK main thread is never blocked.
        let hyprland = hyprland_ipc::HyprlandClient::new();
        hyprland.connect_signals();

        // 4. Build EITHER the bar (Windows mode) OR the dock (Mac mode),
        //    never both. The unused surface is skipped entirely.
        if cfg.taskbar_mode == "mac" {
            dock::Dock::new(app, &cfg, &hyprland).present();
        } else {
            bar::Bar::new(app, &cfg, &hyprland).present();
        }

        // 5. Build the launcher (hidden by default). Toggled later via
        //    the Super key / D-Bus shortcut (task 2-b). Holding the
        //    launcher in a local binding keeps the GTK window alive for
        //    the lifetime of the activate closure - GTK ref-counts the
        //    underlying GObject, so dropping the Rust wrapper is safe.
        let _launcher = launcher::Launcher::new(app, &cfg);

        log::info!("Astra Shell ready");
    });

    // Blocks here until the shell exits (process teardown).
    app.run();
}
