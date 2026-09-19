use gtk4::CssProvider;

const THEME_PATH: &str = "/usr/share/astraos/themes/glassmorphism.css";

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

pub fn load_theme() {
    let provider = CssProvider::new();

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
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}