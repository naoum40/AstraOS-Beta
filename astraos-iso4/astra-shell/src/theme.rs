// theme.rs - Astra Shell CSS theme loader.
//
// Loads the glassmorphism stylesheet for the bar / dock / launcher onto
// the default `gdk::Display`, at `STYLE_PROVIDER_PRIORITY_APPLICATION`
// so user themes (loaded at `_USER` priority) can still override it.
//
// Lookup order:
//   1. `/usr/share/astraos/themes/glassmorphism.css`  - system theme
//      installed by the AstraOS packages (this is what production ISO
//      boots will read).
//   2. Embedded fallback stylesheet (`FALLBACK_CSS` const below) - used
//      during development or when running outside an AstraOS install.
//
// The provider is added globally via
// `gtk4::style_context_add_provider_for_display`, so every GTK window
// created afterwards inherits the Astra styles. This is what makes
// `window.astra-bar { ... }` selectors "just work" on every
// `ApplicationWindow` that has the `astra-bar` CSS class.

use gtk4::CssProvider;

/// System path for the installed glassmorphism stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/glassmorphism.css";

/// Minimal embedded stylesheet, used when the system theme is missing.
///
/// Mirrors the most important rules from `assets/css/glassmorphism.css`
/// (the bar / button / clock surfaces). Production boots read the file
/// at `THEME_PATH` and never use this fallback.
const FALLBACK_CSS: &str = r#"
/* Astra Shell - embedded fallback theme (dev-only). */
window.astra-bar {
    background: rgba(22, 25, 40, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    color: #f1f5f9;
    font-family: 'Outfit', 'Inter', sans-serif;
}
window.astra-dock {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 20px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
window.astra-launcher {
    background: rgba(11, 13, 23, 0.95);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 16px 64px rgba(0, 0, 0, 0.6);
}
button.astra-button {
    background: transparent;
    color: #f1f5f9;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    min-width: 32px;
    min-height: 32px;
    transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
}
button.astra-button:hover {
    background: rgba(129, 140, 248, 0.2);
}
button.astra-button:active {
    background: rgba(129, 140, 248, 0.4);
}
button.astra-start {
    font-size: 18pt;
    color: #818cf8;
}
button.astra-start:hover {
    color: #E040FB;
    text-shadow: 0 0 12px rgba(224, 64, 251, 0.6);
}
.astra-clock {
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 11pt;
    color: #f1f5f9;
    padding: 0 8px;
}
"#;

/// Load the Astra Shell stylesheet onto the default `gdk::Display`.
///
/// Safe to call once at application activate; calling it multiple times
/// will stack additional providers (harmless but mildly wasteful).
///
/// NOTE: `gtk4::CssProvider::load_from_data` takes `&str` (not `&[u8]`)
/// in gtk4-rs 0.9.x and returns `()` - parse errors are swallowed by
/// GTK itself (logged to stderr via g_log). We work around this by
/// validating UTF-8 ourselves before calling the GTK API.
pub fn load_theme() {
    let provider = CssProvider::new();

    // Try the installed system theme first, fall back to the embedded
    // stylesheet so a missing file never crashes the shell.
    match std::fs::read_to_string(THEME_PATH) {
        Ok(css) => {
            provider.load_from_data(&css);
            log::info!("Loaded custom theme from {}", THEME_PATH);
        }
        Err(_) => {
            provider.load_from_data(FALLBACK_CSS);
            log::warn!(
                "Theme file not found at {} - using embedded fallback CSS",
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
            log::error!("Could not get default GdkDisplay - theme not applied");
            return;
        }
    };

    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        // 600 = GTK_STYLE_PROVIDER_PRIORITY_APPLICATION
        // (settings < theme < app < user).
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
