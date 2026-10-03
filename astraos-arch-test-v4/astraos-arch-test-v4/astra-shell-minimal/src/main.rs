use gtk4::prelude::*;
use gtk4::gdk;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Label,
    MenuButton, Popover, Orientation,
    style_context_add_provider_for_display,
};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;

const APP_ID: &str = "org.astraos.ShellMinimal";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let css = r#"
            window { background-color: #0b0d17; }
            .bar {
                background-color: rgba(22, 25, 40, 0.85);
                border-top: 1px solid rgba(129, 140, 248, 0.12);
                padding: 4px 12px;
            }
            .start-btn {
                color: #818cf8;
                font-family: 'Outfit', 'Segoe UI', sans-serif;
                font-size: 18px;
                font-weight: 700;
                background: transparent;
                border: none;
                padding: 6px 12px;
                border-radius: 6px;
                min-width: 40px;
            }
            .start-btn:hover { background: rgba(129, 140, 248, 0.15); }
            .dock-btn {
                background: transparent;
                border: none;
                padding: 6px;
                border-radius: 6px;
                color: #f1f5f9;
                min-width: 36px;
                min-height: 36px;
            }
            .dock-btn:hover { background: rgba(255, 255, 255, 0.1); }
            .clock {
                font-family: 'JetBrains Mono', 'Consolas', monospace;
                font-size: 13px;
                font-weight: 500;
                color: #f1f5f9;
                padding: 6px 12px;
            }
            popover contents {
                background-color: rgba(20, 22, 35, 0.95);
                border-radius: 12px;
                padding: 12px;
                color: #f1f5f9;
            }
            .popover-btn {
                background: transparent;
                color: #f1f5f9;
                border: none;
                padding: 8px 12px;
                border-radius: 6px;
                font-size: 13px;
                min-width: 140px;
            }
            .popover-btn:hover { background: rgba(129, 140, 248, 0.2); }
            .popover-power { color: #ef4444; }
        "#;

        let provider = CssProvider::new();
        provider.load_from_data(css);
        let display = gdk::Display::default();
        if let Some(d) = display {
            style_context_add_provider_for_display(
                &d,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let window = ApplicationWindow::builder()
            .application(app)
            .title("AstraOS Shell")
            .default_width(1920)
            .default_height(40)
            .decorated(false)
            .build();

        let bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .css_classes(["bar"])
            .hexpand(true)
            .vexpand(true)
            .build();

        let center_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .hexpand(true)
            .build();

        let start_btn = MenuButton::builder()
            .label("✦")
            .css_classes(["start-btn"])
            .build();

        let popover_content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();

        let menu_apps = [
            ("Firefox", "applications-internet", "firefox"),
            ("Terminal", "utilities-terminal", "kitty"),
            ("Files", "folder", "thunar"),
        ];

        for (label, _icon_name, command) in menu_apps.iter() {
            let btn = Button::builder()
                .label(*label)
                .css_classes(["popover-btn"])
                .build();
            let cmd = command.to_string();
            btn.connect_clicked(move |_| {
                let _ = Command::new(&cmd).spawn();
            });
            popover_content.append(&btn);
        }

        let sep = GtkBox::builder()
            .height_request(1)
            .css_classes(["separator"])
            .build();
        popover_content.append(&sep);

        let power_btn = Button::builder()
            .label("⏻ Power")
            .css_classes(["popover-btn", "popover-power"])
            .build();
        power_btn.connect_clicked(|_| {
            let _ = Command::new("systemctl").arg("poweroff").spawn();
        });
        popover_content.append(&power_btn);

        let popover = Popover::new();
        popover.set_child(Some(&popover_content));
        start_btn.set_popover(Some(&popover));

        center_box.append(&start_btn);

        let pinned_apps = [
            ("Firefox", "applications-internet", "firefox"),
            ("Terminal", "utilities-terminal", "kitty"),
            ("Files", "folder", "thunar"),
        ];

        for (label, icon_name, command) in pinned_apps.iter() {
            let btn = Button::builder()
                .icon_name(*icon_name)
                .tooltip_text(*label)
                .css_classes(["dock-btn"])
                .build();
            let cmd = command.to_string();
            btn.connect_clicked(move |_| {
                let _ = Command::new(&cmd).spawn();
            });
            center_box.append(&btn);
        }

        bar.append(&center_box);

        let clock = Label::builder()
            .label("00:00")
            .css_classes(["clock"])
            .halign(gtk4::Align::End)
            .build();
        bar.append(&clock);

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
        let h = (secs / 3600 + 1) % 24;
        let m = (secs / 60) % 60;
        label.set_label(&format!("{:02}:{:02}", h, m));
    }
}