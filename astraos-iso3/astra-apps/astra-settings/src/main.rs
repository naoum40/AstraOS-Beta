// main.rs - Astra Settings entry point.
//
// Boots the AstraOS Settings app on top of GTK4 + libadwaita. Owns the
// `gtk4::Application` (GApplication ID `org.astraos.Settings`) and builds
// a single `libadwaita::ApplicationWindow` with a `Leaflet` split view:
//
//   ┌──────────────┬─────────────────────────────┐
//   │  Sidebar     │  Content (Stack)             │
//   │  240px       │  switches between pages      │
//   │  - Account   │  on sidebar selection.       │
//   │  - Perso.    │                              │
//   │  - System    │                              │
//   │  - About     │                              │
//   └──────────────┴─────────────────────────────┘
//
// Pipeline:
//   1. `env_logger::init()` - stderr logging (RUST_LOG=info).
//   2. `gtk4::init()` + `libadwaita::init()` - GTK + libadwaita bootstrap.
//   3. Build the `Application` with GApplication flags = NONE.
//   4. On activate: load CSS theme, load user config, build the
//      `AdwApplicationWindow`, attach sidebar + content to a Leaflet,
//      present the window.
//   5. `app.run()` blocks until the window is closed.

mod config;
mod pages;

use config::SettingsConfig;
use gtk4::prelude::*;
use gtk4::{Application, CssProvider};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow as AdwWindow, Leaflet, LeafletTransitionType};

// `gio` is a direct Cargo dependency (see Cargo.toml), so the bare
// crate name `gio::ApplicationFlags` resolves without an explicit
// `use gio;` import (Rust 2018+ crate path resolution).

/// GApplication ID - registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.Settings";

/// System path for the installed settings stylesheet.
/// Mirrors the layout used by the Astra Shell theme loader.
const CSS_PATH: &str = "/usr/share/astraos/themes/settings.css";

/// Embedded fallback stylesheet - used in dev or when the system theme
/// is missing. Pulled in at compile time via `include_str!`.
const FALLBACK_CSS: &str = include_str!("../assets/css/settings.css");

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
        // 1. Load CSS theme (glassmorphism dark + magenta accent) onto
        //    the default display. Every AdwApplicationWindow created
        //    afterwards inherits the Astra styles.
        load_css();

        // 2. Load user config from ~/.config/astra/desktop.toml
        //    (falls back to AstraOS defaults if missing or unparseable).
        let cfg = SettingsConfig::load();
        log::info!("Config: {:?}", cfg);

        // 3. Build the main window - libadwaita ApplicationWindow with
        //    a 900x600 default size and the "astra-settings" CSS class
        //    (drives the glassmorphism background).
        let window = AdwWindow::builder()
            .application(app)
            .title("Astra Settings")
            .default_width(900)
            .default_height(600)
            .build();
        window.add_css_class("astra-settings");

        // 4. Build the responsive Leaflet (sidebar | content). We set
        //    `can_unfold(false)` so both panes are always visible side
        //    by side on the 900px-wide window - the equivalent of the
        //    old `LeafletFoldPolicy::Never` value (the fold-policy enum
        //    was dropped from libadwaita when `Leaflet` was reshaped
        //    in 1.4 and re-exposed as a deprecated widget in libadwaita-rs
        //    0.7). On a narrow display the leaflet would otherwise fold
        //    to a single pane and require navigation back/forward.
        let leaflet = Leaflet::builder()
            .css_classes(["astra-leaflet"])
            .can_unfold(false)
            .can_navigate_back(true)
            .transition_type(LeafletTransitionType::Over)
            .build();

        // 5. Build sidebar (left, 240px) and content area (right, fills).
        let sidebar = pages::sidebar::Sidebar::new();
        let content = pages::content::Content::new(&cfg);

        // 6. Wire sidebar selection -> content stack switch. The
        //    closure captures a clone of `content` (GTK widgets are
        //    Rc-refcounted internally, so cloning is cheap).
        {
            let content_for_cb = content.clone();
            sidebar.connect_page_changed(move |idx| {
                content_for_cb.switch_to(idx);
            });
        }

        // 7. Set the initial selection (Account = index 0). This also
        //    triggers the callback above, switching the content stack
        //    to the account page on first paint.
        sidebar.set_active(0);

        // 8. Append sidebar + content to the leaflet. The leaflet
        //    takes its own reference; the Rust wrappers can drop later.
        leaflet.append(sidebar.widget());
        leaflet.append(content.widget());

        // 9. Attach the leaflet as the AdwApplicationWindow's content
        //    child (libadwaita replaces `set_child` with `set_content`
        //    on ApplicationWindow so the bottom-bar / titlebar styling
        //    stays consistent with libadwaita's chrome).
        window.set_content(Some(&leaflet));
        window.present();

        log::info!("Astra Settings window presented");
    });

    // Blocks here until the window is closed (process teardown).
    app.run();
}

/// Load the Astra Settings stylesheet onto the default `gdk::Display`.
///
/// Lookup order:
///   1. `/usr/share/astraos/themes/settings.css`  - system theme
///      installed by the AstraOS packages (this is what production ISO
///      boots will read).
///   2. Embedded fallback stylesheet (`FALLBACK_CSS` const, pulled from
///      `assets/css/settings.css` via `include_str!`) - used during
///      development or when running outside an AstraOS install.
///
/// NOTE: `gtk4::CssProvider::load_from_data` takes `&str` (not `&[u8]`)
/// in gtk4-rs 0.9.x and returns `()` - parse errors are swallowed by
/// GTK itself (logged to stderr via g_log).
fn load_css() {
    let provider = CssProvider::new();

    let css = match std::fs::read_to_string(CSS_PATH) {
        Ok(css) => {
            log::info!("Loaded settings CSS from {}", CSS_PATH);
            css
        }
        Err(_) => {
            log::warn!(
                "Settings CSS not found at {} - using embedded fallback CSS",
                CSS_PATH
            );
            FALLBACK_CSS.to_string()
        }
    };
    provider.load_from_data(&css);

    // Attach the provider to the default display. Every window created
    // on this display will be styled by it.
    //
    // NOTE: the older `StyleContext::add_provider_for_display` method
    // was deprecated in gtk4 0.9.x in favor of the free function
    // `gtk4::style_context_add_provider_for_display`.
    let display = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => {
            log::error!("Could not get default GdkDisplay - CSS not applied");
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
