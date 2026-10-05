mod bar;
mod config;
mod dock;
mod hyprland_ipc;
mod launcher;
mod screens;
mod theme;

use gtk4::prelude::*;
use gtk4::Application;

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
        // 1. Load CSS theme (glassmorphism)
        theme::load_theme();

        // 2. Welcome screen → on_complete lance onboarding
        let app1 = app.clone();
        screens::welcome::WelcomeScreen::new(app, move || {
            log::info!("Welcome complete, starting onboarding");

            // 3. Onboarding → on_complete lance lock screen
            let app2 = app1.clone();
            screens::onboarding::Onboarding::new(&app1, move || {
                log::info!("Onboarding complete, showing lock screen");

                // 4. Lock screen → on_unlock lance la barre
                let app3 = app2.clone();
                screens::lock_screen::LockScreen::new(&app2, move || {
                    log::info!("Unlocked, showing desktop");

                    // 5. Bureau final
                    let cfg = config::ShellConfig::load();
                    let hyprland = hyprland_ipc::HyprlandClient::new();
                    hyprland.connect_signals();

                    if cfg.taskbar_mode == "mac" {
                        dock::Dock::new(&app3, &cfg, &hyprland).present();
                    } else {
                        bar::Bar::new(&app3, &cfg, &hyprland).present();
                    }
                    let _launcher = launcher::Launcher::new(&app3, &cfg);
                }).present();
            }).present();
        }).present();
    });

    app.run();
}