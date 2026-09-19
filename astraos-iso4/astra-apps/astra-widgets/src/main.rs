// main.rs - Astra Widgets entry point.
//
// Boots the AstraOS desktop-widget host. Owns the GTK4 `Application` and
// spawns one undecorated `ApplicationWindow` per enabled widget (clock,
// calendar, weather, battery) as specified by
// `~/.config/astra/widgets.toml`.
//
// Pipeline:
//   1. `env_logger::init()` — stderr logging (RUST_LOG=info).
//   2. `gtk4::init()` — must run on the GTK main thread.
//   3. Build the `Application` (GApplication ID `org.astraos.Widgets`).
//   4. On activate: load the glassmorphism stylesheet, load the user
//      config, then dispatch `widgets::spawn(name, app)` for each entry
//      in `enabled_widgets`.
//
// Windows are undecorated (`decorated = false`) and tagged with the
// `astra-widget` CSS class so the shared stylesheet paints the
// glassmorphism background on them. Each widget is also wired up with
// a `GestureDrag` controller (see `widgets::attach_drag`) so the user
// can reposition it; the new position is persisted to
// `~/.config/astra/widgets.toml` on drag-end.

mod config;
mod widgets;

use gtk4::prelude::*;
use gtk4::{gio, CssProvider};

/// GApplication ID — registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.Widgets";

/// System path for the installed widgets stylesheet. Production ISO 4
/// boots read this file; dev runs fall back to the embedded CSS.
const THEME_PATH: &str = "/usr/share/astraos/themes/widgets.css";

/// Embedded fallback stylesheet — pulled in at compile time via
/// `include_str!` so the binary is self-contained and boots even when
/// the system theme file is missing.
const FALLBACK_CSS: &str = include_str!("../assets/css/widgets.css");

fn main() {
    env_logger::init();
    log::info!("Astra Widgets v0.4.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        // 1. Load the glassmorphism stylesheet onto the default display
        //    so every widget window inherits the Astra look.
        load_theme();

        // 2. Load user config from ~/.config/astra/widgets.toml (falls
        //    back to the AstraOS default if missing/unparseable).
        let cfg = config::WidgetsConfig::load();
        log::info!("Enabled widgets: {:?}", cfg.enabled_widgets);

        // 3. Spawn each enabled widget in its own GTK window.
        for name in &cfg.enabled_widgets {
            widgets::spawn(name, app);
        }

        log::info!(
            "Astra Widgets ready ({} widgets spawned)",
            cfg.enabled_widgets.len()
        );
    });

    // Blocks here until the user quits (process teardown).
    app.run();
}

/// Load the widgets stylesheet onto the default `gdk::Display` at
/// `STYLE_PROVIDER_PRIORITY_APPLICATION` so user themes (loaded at
/// `_USER` priority) can still override it.
///
/// Lookup order:
///   1. `/usr/share/astraos/themes/widgets.css` — system theme
///      installed by the AstraOS packages (production ISO boots).
///   2. Embedded fallback stylesheet (`FALLBACK_CSS` const) — used in
///      dev runs and when the system file is missing.
///
/// NOTE: `gtk4::CssProvider::load_from_data` takes `&str` (not `&[u8]`)
/// in gtk4-rs 0.9.x and returns `()` — parse errors are swallowed by
/// GTK itself (logged via `g_log`).
fn load_theme() {
    let provider = CssProvider::new();

    match std::fs::read_to_string(THEME_PATH) {
        Ok(css) => {
            provider.load_from_data(&css);
            log::info!("Loaded theme from {}", THEME_PATH);
        }
        Err(_) => {
            provider.load_from_data(FALLBACK_CSS);
            log::warn!(
                "Theme file missing at {} — using embedded fallback CSS",
                THEME_PATH
            );
        }
    }

    // Attach the provider to the default display so every widget window
    // created afterwards picks up the `window.astra-widget` rules.
    //
    // The free function `gtk4::style_context_add_provider_for_display`
    // replaces the deprecated `StyleContext::add_provider_for_display`
    // method in gtk4 0.9.x.
    match gtk4::gdk::Display::default() {
        Some(display) => {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        None => {
            log::error!("No default GdkDisplay — stylesheet not applied");
        }
    }
}
