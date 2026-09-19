// main.rs - AstraOS Screenshot tool.
//
// Small utility to grab screenshots under Hyprland/Wayland. Wraps
// `grim` (image capture), `slurp` (region selection) and
// `hyprctl`/`jq` (active-window geometry). The shell pipeline runs on
// a worker thread so the GTK UI never blocks. On success a GIO
// notification announces the saved file path.
//
// Pipeline:
//   1. `env_logger::init()` - stderr logging (RUST_LOG=info).
//   2. `gtk4::init()` - boots GTK on the main thread.
//   3. Build the `Application` (GApplicationID `org.astraos.Screenshot`).
//   4. On activate: build a 400x300 undecorated window with three
//      vertically-centered capture buttons. Each button spawns a
//      worker thread that runs the shell pipeline; on success the
//      worker uses `glib::idle_add_once` to dispatch a GIO
//      notification on the main thread via a `SendWeakRef<Application>`.

use std::process::Command;
use std::thread;

use chrono::Local;
use gio::ApplicationFlags;
use glib::object::SendWeakRef;
use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box as GtkBox, Button, Orientation};

/// GApplication ID registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.Screenshot";

/// Entry point. Boots GTK, builds the capture window, and runs the
/// GApplication until the window is closed.
fn main() {
    env_logger::init();
    log::info!("Astra Screenshot v0.3.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

/// Construct the main capture window: 400x300, undecorated, three
/// vertically-centered buttons (Full / Region / Window capture).
fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Astra Screenshot")
        .default_width(400)
        .default_height(300)
        .decorated(false)
        .build();
    window.add_css_class("astra-screenshot");

    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(24)
        .build();
    window.set_child(Some(&container));

    // `SendWeakRef<Application>` is `Send+Sync` even though
    // `Application` itself is not, so it can travel to a worker
    // thread. The worker schedules a GIO notification back on the
    // main thread via `glib::idle_add_once`.
    let app_weak: SendWeakRef<Application> = app.downgrade().into();

    // --- Capture Full Screen ---------------------------------------
    //   grim -t png ~/Pictures/Screenshot_YYYYMMDD_HHMMSS.png
    let btn_full = Button::builder()
        .label("Capture Full Screen")
        .css_classes(["astra-button", "astra-primary"])
        .build();
    {
        let app_weak = app_weak.clone();
        btn_full.connect_clicked(move |_| {
            let path = screenshot_path();
            let cmd = format!("grim -t png \"{}\"", path);
            spawn_capture(&cmd, path, "full-screen", app_weak.clone());
        });
    }

    // --- Capture Region --------------------------------------------
    //   slurp | grim -g - ~/Pictures/Screenshot_YYYYMMDD_HHMMSS.png
    let btn_region = Button::builder()
        .label("Capture Region")
        .css_classes(["astra-button"])
        .build();
    {
        let app_weak = app_weak.clone();
        btn_region.connect_clicked(move |_| {
            let path = screenshot_path();
            let cmd = format!("slurp | grim -g - \"{}\"", path);
            spawn_capture(&cmd, path, "region", app_weak.clone());
        });
    }

    // --- Capture Window --------------------------------------------
    //   hyprctl -j activewindow > /tmp/astra-window.json &&
    //   grim -l $(cat /tmp/astra-window.json | jq -r '.address') \
    //        ~/Pictures/Screenshot_YYYYMMDD_HHMMSS.png
    let btn_window = Button::builder()
        .label("Capture Window")
        .css_classes(["astra-button"])
        .build();
    {
        let app_weak = app_weak.clone();
        btn_window.connect_clicked(move |_| {
            let path = screenshot_path();
            let cmd = format!(
                "hyprctl -j activewindow > /tmp/astra-window.json && \
                 grim -l $(cat /tmp/astra-window.json | jq -r '.address') \"{}\"",
                path
            );
            spawn_capture(&cmd, path, "window", app_weak.clone());
        });
    }

    container.append(&btn_full);
    container.append(&btn_region);
    container.append(&btn_window);

    window.present();
}

/// Build a `~/Pictures/Screenshot_YYYYMMDD_HHMMSS.png` path, ensuring
/// the Pictures directory exists. Falls back to `/home/astra` when
/// `HOME` is not set.
fn screenshot_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
    let dir = std::path::PathBuf::from(&home).join("Pictures");
    let _ = std::fs::create_dir_all(&dir);
    let stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    dir.join(format!("Screenshot_{}.png", stamp))
        .to_string_lossy()
        .to_string()
}

/// Spawn a shell pipeline on a worker thread. On success, schedule a
/// GIO notification on the main-thread idle loop. Failures are logged
/// but never crash the UI.
///
/// `app_weak` is dereferenced only inside the `idle_add_once` closure
/// which runs on the main thread, so the `SendWeakRef` thread-id
/// assertion always passes.
fn spawn_capture(
    shell_cmd: &str,
    file_path: String,
    label: &str,
    app_weak: SendWeakRef<Application>,
) {
    let cmd = shell_cmd.to_string();
    let label = label.to_string();
    thread::spawn(move || {
        log::info!("[{}] running: {}", label, cmd);
        let status = Command::new("sh").arg("-c").arg(&cmd).status();
        match status {
            Ok(s) if s.success() => {
                glib::idle_add_once(move || {
                    if let Some(app) = app_weak.upgrade() {
                        let body = format!("Screenshot saved to {}", file_path);
                        log::info!("{}", body);
                        let notification = gio::Notification::new("Screenshot Saved");
                        notification.set_body(Some(&body));
                        app.send_notification(Some("astra-screenshot"), &notification);
                    }
                });
            }
            Ok(s) => {
                log::error!("[{}] command exited with {:?}", label, s.code());
            }
            Err(e) => {
                log::error!("[{}] failed to spawn command: {}", label, e);
            }
        }
    });
}
