// ==============================================================================
// AstraOS Shell — Entry Point (bureau + dock assemblés)
// ==============================================================================
// Assemble :
//   1. Desktop (wallpaper + desktop icons) — astra_desktop.rs
//   2. Dock (macOS-style, traduit de MacOSDock) — astra_dock lib
// ==============================================================================
use gtk4::prelude::*;
use gtk4::Application;

mod desktop;
// Import la lib astra_dock (le dock traduit)
// En production : astra_dock serait un crate séparé (dependency)
// Pour le dev : copier les modules ici ou utiliser path dependency
use astra_dock::{DockManager, DockConfig, DockPosition};

const APP_ID: &str = "org.astraos.Shell";
const WALLPAPER_PATH: &str = "/usr/share/backgrounds/astraos/default-violet.png";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(move |_| {
        // 1. Bureau : wallpaper + desktop icons
        desktop::Desktop::new(WALLPAPER_PATH);

        // 2. Dock : macOS-style (traduit de MacOSDock)
        let config = DockConfig {
            icon_size: 48,
            dock_opacity: 0.6,
            background_color: (0x1e, 0x1e, 0x1e),
            border_radius: 16,
            blur_enabled: true,
            position: DockPosition::Bottom,
            auto_hide: true,
            magnification_enabled: true,
            magnification_scale: 1.4,
            magnification_falloff: 100,
            magnification_framerate: 60,
            bounce_on_launch: true,
            animation_duration_ms: 200,
            show_threshold: 25,
            running_indicators: true,
            show_running_apps: true,
        };
        let _dock = DockManager::new(config);

        println!("[astra-shell] Desktop + Dock ready");
    });

    // Hold : le shell n'a pas de fenêtre principale (layer-shell gère tout)
    let hold = app.hold();
    std::mem::forget(hold);

    app.run()
}
