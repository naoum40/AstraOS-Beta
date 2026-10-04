use gtk4::prelude::*;
use gtk4::gdk;
use gtk4::{
    Application, ApplicationWindow, Button, CenterBox, CssProvider, Label,
    Box as GtkBox, Orientation,
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
        // Barre pleine largeur, ancrée en bas — style Windows 11 :
        // icônes de lancement groupées et centrées, horloge calée à droite.
        let css = r#"
            window { background-color: transparent; }
            .taskbar {
                background-color: rgba(20, 22, 34, 0.78);
                border-top: 1px solid rgba(255, 255, 255, 0.06);
                padding: 0 10px;
            }
            .taskbar-center {
                padding: 0 4px;
            }
            .taskbar-btn {
                background: transparent;
                border: none;
                padding: 8px;
                border-radius: 8px;
                color: #f1f5f9;
                min-width: 40px;
                min-height: 40px;
            }
            .taskbar-btn:hover {
                background: rgba(255, 255, 255, 0.10);
            }
            .taskbar-btn:active {
                background: rgba(255, 255, 255, 0.16);
            }
            .star-btn {
                color: #818cf8;
                font-size: 17px;
                font-weight: 700;
            }
            .clock {
                font-family: 'JetBrains Mono', 'Consolas', monospace;
                font-size: 12px;
                font-weight: 500;
                color: #f1f5f9;
                padding: 4px 14px;
                border-radius: 6px;
            }
            .clock:hover {
                background: rgba(255, 255, 255, 0.08);
            }
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
            .title("AstraOS Taskbar")
            .default_width(1920)
            .default_height(48)
            .decorated(false)
            .build();

        // CenterBox = layout à 3 zones (gauche / centre / droite), comme la
        // vraie barre des tâches Windows 11 : le centre reste parfaitement
        // centré quel que soit ce qu'il y a à droite (horloge, tray, etc.)
        let taskbar = CenterBox::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(["taskbar"])
            .hexpand(true)
            .vexpand(true)
            .build();

        // -- Zone centrale : étoile (placeholder du futur menu démarrer) + apps épinglées --
        let center_group = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(2)
            .css_classes(["taskbar-center"])
            .build();

        let star_btn = Button::builder()
            .label("✦")
            .css_classes(["taskbar-btn", "star-btn"])
            .tooltip_text("AstraOS (menu démarrer à venir)")
            .build();
        // Pas de menu pour l'instant — l'étoile ne fait rien au clic.
        center_group.append(&star_btn);

        let pinned_apps = [
            ("Firefox", "applications-internet", "firefox"),
            ("Terminal", "utilities-terminal", "kitty"),
            ("Fichiers", "folder", "thunar"),
        ];

        for (label, icon_name, command) in pinned_apps.iter() {
            let btn = Button::builder()
                .icon_name(*icon_name)
                .tooltip_text(*label)
                .css_classes(["taskbar-btn"])
                .build();
            let cmd = command.to_string();
            btn.connect_clicked(move |_| {
                let _ = Command::new(&cmd).spawn();
            });
            center_group.append(&btn);
        }

        taskbar.set_center_widget(Some(&center_group));

        // -- Zone droite : horloge, façon tray système Windows 11 --
        let clock = Label::builder()
            .label("00:00")
            .css_classes(["clock"])
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::Center)
            .build();
        taskbar.set_end_widget(Some(&clock));

        let clock_clone = clock.clone();
        update_clock(&clock_clone);
        glib::timeout_add_local(
            std::time::Duration::from_secs(1),
            move || {
                update_clock(&clock_clone);
                glib::ControlFlow::Continue
            },
        );

        window.set_child(Some(&taskbar));
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