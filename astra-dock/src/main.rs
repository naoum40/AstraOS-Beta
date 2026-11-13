// ==============================================================================
// AstraOS Dock — Entry Point (standalone binary pour dev/test)
// ==============================================================================
// En production : le dock est importé comme bibliothèque par astra-shell.
// Ce binaire standalone sert au dev/test (lancer le dock seul).
// ==============================================================================
use astra_dock::{DockManager, DockConfig, DockPosition};
use gtk4::prelude::*;
use gtk4::Application;

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.astraos.dock")
        .build();

    app.connect_activate(move |_| {
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
    });

    let hold = app.hold();
    std::mem::forget(hold);
    app.run()
}
