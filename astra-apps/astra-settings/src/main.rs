// main.rs - Astra Settings entry point.
//
// Boots the AstraOS system configuration panel ("Astra Settings") — the
// Windows-Settings-style control center for the desktop.
//
// Pipeline:
//   1. `env_logger::init()` — stderr logging (RUST_LOG=info).
//   2. `libadwaita::init()` + `gtk4::init()` — must run on the GTK main thread.
//   3. Build the `Application` (GApplication ID `org.astraos.Settings`).
//   4. On activate: load CSS theme, load user config, build the sidebar +
//      content stack inside a `libadwaita::ApplicationWindow` 900x600.
//
// Layout: 240px sidebar on the left (4 nav buttons), content area on the
// right (a `gtk4::Stack` switching between Account / Personalization /
// System / About pages).

mod config;
mod pages;

use gtk4::prelude::*;
use gtk4::{Application, CssProvider};

/// GApplication ID — registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.Settings";

/// System path for the installed settings stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/settings.css";

/// Minimal embedded stylesheet, used when the system theme is missing.
///
/// Mirrors the most important rules from `assets/css/settings.css`.
/// Production boots read the file at `THEME_PATH` and never use this
/// fallback.
const FALLBACK_CSS: &str = r#"
/* Astra Settings — embedded fallback theme (dev-only). */
window.astra-settings {
    background: rgba(11, 13, 23, 0.96);
    color: #f1f5f9;
    font-family: 'Outfit', 'Inter', sans-serif;
}
.astra-sidebar {
    background: rgba(17, 20, 35, 0.7);
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    padding: 16px 8px;
}
button.astra-nav {
    background: transparent;
    color: #cbd5e1;
    border: none;
    border-radius: 10px;
    padding: 12px 14px;
    margin: 2px 0;
    font-size: 11pt;
    text-align: left;
    transition: all 180ms cubic-bezier(0.4, 0, 0.2, 1);
}
button.astra-nav:hover {
    background: rgba(129, 140, 248, 0.18);
    color: #f8fafc;
}
button.astra-nav.active {
    background: linear-gradient(135deg, rgba(129, 140, 248, 0.35), rgba(224, 64, 251, 0.25));
    color: #ffffff;
    box-shadow: 0 4px 16px rgba(129, 140, 248, 0.25);
}
.astra-content {
    background: transparent;
    padding: 28px 36px;
}
.astra-section-title {
    font-size: 13pt;
    font-weight: 700;
    color: #f1f5f9;
    margin-bottom: 8px;
}
.astra-row {
    background: rgba(255, 255, 255, 0.04);
    border-radius: 12px;
    padding: 14px 16px;
    margin-bottom: 10px;
    border: 1px solid rgba(255, 255, 255, 0.04);
}
switch slider {
    background: #f1f5f9;
}
switch:checked {
    background: #818cf8;
}
button.astra-color {
    border-radius: 999px;
    min-width: 36px;
    min-height: 36px;
    padding: 0;
    border: 2px solid rgba(255, 255, 255, 0.15);
}
"#;

/// Load the Astra Settings stylesheet onto the default `gdk::Display`.
///
/// Safe to call once at application activate; calling it multiple times
/// will stack additional providers (harmless but mildly wasteful).
///
/// NOTE: `gtk4::CssProvider::load_from_data` takes `&str` (not `&[u8]`)
/// in gtk4-rs 0.9.x and returns `()` — parse errors are swallowed by
/// GTK itself (logged to stderr via g_log).
fn load_theme() {
    let provider = CssProvider::new();

    // Try the installed system theme first, fall back to the embedded
    // stylesheet so a missing file never crashes the app.
    match std::fs::read_to_string(THEME_PATH) {
        Ok(css) => {
            provider.load_from_data(&css);
            log::info!("Loaded custom theme from {}", THEME_PATH);
        }
        Err(_) => {
            provider.load_from_data(FALLBACK_CSS);
            log::warn!(
                "Theme file not found at {} — using embedded fallback CSS",
                THEME_PATH
            );
        }
    }

    // Attach the provider to the default display. Every ApplicationWindow
    // created on this display will be styled by it.
    //
    // NOTE: the older `StyleContext::add_provider_for_display` method
    // was deprecated in gtk4 0.9.x in favor of the free function
    // `gtk4::style_context_add_provider_for_display` (mirrors the GTK4
    // C deprecation of `gtk_style_context_add_provider_for_display`).
    let display = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => {
            log::error!("Could not get default GdkDisplay — theme not applied");
            return;
        }
    };

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        // 600 = GTK_STYLE_PROVIDER_PRIORITY_APPLICATION
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    env_logger::init();
    log::info!("Astra Settings v0.3.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");
    libadwaita::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        // 1. Load CSS theme (glassmorphism) onto the default display.
        load_theme();

        // 2. Load user config from ~/.config/astra/desktop.toml
        //    (falls back to a sensible AstraOS default if missing).
        let cfg = config::SettingsConfig::load();
        log::info!("Config: {:?}", cfg);

        // 3. Build the sidebar (left, 240px) and the content stack (right).
        let sidebar = pages::sidebar::Sidebar::new();
        let content = pages::content::Content::new();

        // 4. Wire sidebar → content. The sidebar's selection callback
        //    forwards the page key ("account" / "personalization" /
        //    "system" / "about") to the content stack.
        let content_clone = content.clone();
        sidebar.set_on_select(move |page: &str| {
            content_clone.switch_to(page);
        });

        // 5. Default to the Account page.
        sidebar.select_first();
        content.switch_to("account");

        // 6. Assemble the window: horizontal box (sidebar | content).
        let root = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(0)
            .homogeneous(false)
            .build();
        root.append(sidebar.widget());
        root.append(content.widget());

        // Sidebar is fixed-width 240px; content fills the rest.
        sidebar.widget().set_size_request(240, -1);
        content.widget().set_hexpand(true);
        content.widget().set_vexpand(true);

        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("Astra Settings")
            .default_width(900)
            .default_height(600)
            .content(&root)
            .build();

        // The `astra-settings` CSS class is the primary selector for the
        // glassmorphism panel rules in `settings.css`.
        window.add_css_class("astra-settings");

        window.present();
        log::info!("Astra Settings ready");
    });

    // Blocks here until the settings window closes.
    app.run();
}
