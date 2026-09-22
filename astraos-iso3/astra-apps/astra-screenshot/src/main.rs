use chrono::Local;
use gtk4::prelude::*;
use gtk4::{
    gio, glib, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Label,
    Orientation,
};
use std::path::PathBuf;
use std::time::Duration;

const APP_ID: &str = "com.astraos.Screenshot";

fn timestamp() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
}

fn pictures_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join("Pictures")
}

fn build_command(mode: &str, path_str: &str) -> String {
    match mode {
        "region" => format!("slurp | grim -g - '{}'", path_str),
        "window" => format!(
            "hyprctl -j activewindow > /tmp/window.json && grim -l $(cat /tmp/window.json | grep -o '\"address\":[^,]*' | head -1 | cut -d'\"' -f4) '{}'",
            path_str
        ),
        _ => format!("grim -t png '{}'", path_str),
    }
}

fn trigger_capture(mode: &str, status: &Label, app: &Application) {
    let dir = pictures_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        status.set_text(&format!("Cannot create Pictures dir: {}", e));
        return;
    }
    let path = dir.join(format!("Screenshot_{}.png", timestamp()));
    let path_str = path.to_string_lossy().to_string();
    let cmd = build_command(mode, &path_str);

    status.set_text("Capturing...");

    let mut child = match std::process::Command::new("sh").arg("-c").arg(&cmd).spawn() {
        Ok(c) => c,
        Err(e) => {
            status.set_text(&format!("Spawn error: {}", e));
            return;
        }
    };

    let status_clone = status.clone();
    let app_clone = app.clone();

    glib::timeout_add_local(Duration::from_millis(100), move || match child.try_wait() {
        Ok(Some(exit_status)) => {
            if exit_status.success() {
                status_clone.set_text(&format!("Saved: {}", path_str));
                let notif = gio::Notification::new("Screenshot saved");
                notif.set_body(Some(&format!("Screenshot saved to {}", path_str)));
                app_clone.send_notification(None, &notif);
            } else {
                status_clone.set_text(&format!("Capture failed: {}", exit_status));
            }
            glib::ControlFlow::Break
        }
        Ok(None) => glib::ControlFlow::Continue,
        Err(e) => {
            status_clone.set_text(&format!("Wait error: {}", e));
            glib::ControlFlow::Break
        }
    });
}

fn main() {
    env_logger::init();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Screenshot")
            .default_width(400)
            .default_height(300)
            .build();

        let css = CssProvider::new();
        css.load_from_data(
            "
            .astra-screenshot {
                background: rgba(18, 18, 32, 0.85);
            }
            .astra-screenshot box {
                padding: 24px;
            }
            .astra-screenshot button {
                background: rgba(120, 80, 200, 0.45);
                color: white;
                border-radius: 12px;
                padding: 12px 16px;
                font-weight: 600;
            }
            .astra-screenshot button:hover {
                background: rgba(160, 100, 220, 0.65);
            }
            .astra-screenshot button:active {
                background: rgba(100, 60, 180, 0.7);
            }
            .astra-screenshot label.title {
                color: white;
                font-size: 20px;
                font-weight: 700;
                margin-bottom: 12px;
            }
            .astra-screenshot label.status {
                color: rgba(255, 255, 255, 0.7);
                font-size: 12px;
                margin-top: 12px;
            }
            ",
        );
        let display = WidgetExt::display(&window);
        gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .build();

        let title = Label::builder()
            .label("Astra Screenshot")
            .css_classes(["title"])
            .build();

        let btn_full = Button::builder().label("Capture Full Screen").build();
        let btn_region = Button::builder().label("Capture Region").build();
        let btn_window = Button::builder().label("Capture Window").build();

        let status = Label::builder()
            .label("Ready")
            .css_classes(["status"])
            .build();

        main_box.append(&title);
        main_box.append(&btn_full);
        main_box.append(&btn_region);
        main_box.append(&btn_window);
        main_box.append(&status);

        window.set_child(Some(&main_box));

        {
            let status_clone = status.clone();
            let app_clone = app.clone();
            btn_full.connect_clicked(move |_| {
                trigger_capture("full", &status_clone, &app_clone);
            });
        }
        {
            let status_clone = status.clone();
            let app_clone = app.clone();
            btn_region.connect_clicked(move |_| {
                trigger_capture("region", &status_clone, &app_clone);
            });
        }
        {
            let status_clone = status.clone();
            let app_clone = app.clone();
            btn_window.connect_clicked(move |_| {
                trigger_capture("window", &status_clone, &app_clone);
            });
        }

        window.add_css_class("astra-screenshot");
        window.present();
    });

    app.run();
}
