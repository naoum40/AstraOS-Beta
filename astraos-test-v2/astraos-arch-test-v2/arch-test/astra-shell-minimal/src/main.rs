// astra-shell-minimal — Barre des tâches glassmorphism (Arch vs Debian test)
// Juste une barre : logo AstraOS à gauche + horloge à droite
// Pas de dock, pas de launcher, pas d'IPC — minimal

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, CssProvider, Label,
    StyleContext, Orientation,
};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_ID: &str = "org.astraos.ShellMinimal";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        // ── CSS glassmorphism ──────────────────────────────
        let css = r#"
            window {
                background-color: #0b0d17;
            }
            .bar {
                background-color: rgba(22, 25, 40, 0.85);
                padding: 6px 20px;
                border-bottom: 1px solid rgba(129, 140, 248, 0.12);
            }
            .logo {
                font-family: 'Outfit', 'Segoe UI', sans-serif;
                font-size: 15px;
                font-weight: 700;
                color: #818cf8;
            }
            .clock {
                font-family: 'JetBrains Mono', 'Consolas', monospace;
                font-size: 13px;
                font-weight: 500;
                color: #f1f5f9;
            }
            .spacer {
                hexpand: true;
            }
        "#;

        let provider = CssProvider::new();
        provider.load_from_data(css.as_bytes());
        if let Some(display) = app.default_display() {
            StyleContext::add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        // ── Window (undecorated, full width, 36px height) ───
        let window = ApplicationWindow::builder()
            .application(app)
            .title("AstraOS Shell")
            .default_width(1920)
            .default_height(36)
            .decorated(false)
            .build();

        // ── Bar (horizontal box) ───────────────────────────
        let bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .css_classes(["bar"])
            .hexpand(true)
            .vexpand(true)
            .build();

        // Logo à gauche
        let logo = Label::builder()
            .label("✦ AstraOS")
            .css_classes(["logo"])
            .build();
        bar.append(&logo);

        // Spacer au centre
        let spacer = GtkBox::builder()
            .css_classes(["spacer"])
            .hexpand(true)
            .build();
        bar.append(&spacer);

        // Horloge à droite
        let clock = Label::builder()
            .label("00:00")
            .css_classes(["clock"])
            .build();
        bar.append(&clock);

        // Update clock every second
        let clock_clone = clock.clone();
        update_clock(&clock_clone);
        glib::timeout_add_local(
            std::time::Duration::from_secs(1),
            move || {
                update_clock(&clock_clone);
                glib::ControlFlow::Continue
            },
        );

        window.set_child(Some(&bar));
        window.present();
    });

    app.run();
}

fn update_clock(label: &Label) {
    if let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let secs = dur.as_secs();
        let h = (secs / 3600 + 1) % 24; // UTC+1 (Europe/Paris)
        let m = (secs / 60) % 60;
        label.set_label(&format!("{:02}:{:02}", h, m));
    }
}
