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
        theme::load_theme();

        let cfg = config::ShellConfig::load();
        log::info!("Config: {:?}", cfg);

        let hyprland = hyprland_ipc::HyprlandClient::new();
        hyprland.connect_signals();

        if cfg.taskbar_mode == "mac" {
            dock::Dock::new(app, &cfg, &hyprland).present();
        } else {
            bar::Bar::new(app, &cfg, &hyprland).present();
        }

        let _launcher = launcher::Launcher::new(app, &cfg);

        log::info!("Astra Shell ready");
    });

    app.run();
}
