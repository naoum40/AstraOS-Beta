use crate::config::ShellConfig;
use crate::hyprland_ipc::HyprlandClient;
use gtk4::prelude::*;
use gtk4::Box as GtkBox;
use gtk4::{Application, ApplicationWindow, Button, Orientation};

pub struct Dock {
    window: ApplicationWindow,
}

type PinnedApp = (&'static str, &'static str, &'static str);

const PINNED_APPS: &[PinnedApp] = &[
    ("\u{1F310}", "Firefox", "firefox"),                // 🌐
    ("\u{1F4C1}", "Files", "thunar"),                   // 📁
    ("\u{1F4BB}", "Terminal", "kitty"),                 // 💻
    ("\u{2699}\u{FE0F}", "Settings", "astra-settings"), // ⚙️
];

impl Dock {
    pub fn new(app: &Application, _config: &ShellConfig, _hyprland: &HyprlandClient) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Dock")
            .default_width(400)
            .default_height(64)
            .decorated(false)
            .build();

        window.add_css_class("astra-dock");

        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::End)
            .build();
        window.set_child(Some(&container));

        for (icon, tooltip, exec) in PINNED_APPS {
            let btn = Button::builder()
                .label(*icon)
                .tooltip_text(*tooltip)
                .css_classes(["astra-button", "astra-dock-item"])
                .build();

            let exec_cmd = exec.to_string();
            btn.connect_clicked(move |_| {
                log::info!("Launching: {}", exec_cmd);
                match std::process::Command::new(&exec_cmd).spawn() {
                    Ok(_) => log::debug!("Spawned {} OK", exec_cmd),
                    Err(e) => log::warn!("Failed to spawn {}: {}", exec_cmd, e),
                }
            });

            container.append(&btn);
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
