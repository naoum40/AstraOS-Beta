// main.rs - Astra VM entry point.
//
// Boots the AstraOS Virtual Machine Manager — a 3-region GTK4 +
// libadwaita window (header / sidebar / main area) that wraps a
// portable QEMU install (no admin required).
//
// Pipeline:
//   1. `env_logger::init()` — stderr logging (RUST_LOG=info).
//   2. `gtk4::init()` + `libadwaita::init()` — must run on the GTK
//      main thread before any widget is built.
//   3. Build the `gtk4::Application` (GApplication ID
//      `org.astraos.AstraVM`).
//   4. On activate:
//        a. Load the glassmorphism CSS theme (system path with
//           embedded fallback).
//        b. Build the `App` struct (header + sidebar + main area +
//           `AppConfig` + shared state in `Rc<RefCell<…>>`).
//        c. Wire UI signals (header "Nouvelle VM" → modal, sidebar
//           select → main area detail, action buttons → placeholders
//           for subagents B/C/D).
//        d. Auto-select the first VM if any.
//        e. `present()` the window.
//
// The `App` struct holds the GTK widgets + a `RefCell<AppConfig>` so
// subagents (B: new-VM modal + ISO picker, C: QEMU launcher + stats,
// D: settings + recent) can mutate the config + persist it without
// needing their own copy of the config path.

mod config;
mod ui;
mod qmp;
mod stats;
mod capture;
mod qemu_downloader;

use config::{AppConfig, VmConfig};
use gtk4::prelude::*;
use gtk4::{Application, CssProvider, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

use ui::header::Header;
use ui::main_area::MainArea;
use ui::sidebar::Sidebar;

/// GApplication ID — registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.AstraVM";

/// System path for the installed Astra VM stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/astra-vm.css";

/// Embedded fallback stylesheet — used when the system theme file is
/// missing (dev runs without the installed theme).
const FALLBACK_CSS: &str = include_str!("../assets/css/astra-vm.css");

/// The top-level Astra VM application. Owns the GTK window + all UI
/// region structs + the user's `AppConfig` (wrapped in `RefCell` so
/// subagents can mutate it from a `&self` context).
pub struct App {
    window: libadwaita::ApplicationWindow,
    vm_list: RefCell<Vec<VmConfig>>,
    selected_vm: RefCell<Option<String>>,
    sidebar: Sidebar,
    main_area: MainArea,
    header: Header,
    config: RefCell<AppConfig>,
}

impl App {
    /// Build the application UI + wire all signals. Returns an `Rc<App>`
    /// so closures can hold weak references (breaks reference cycles
    /// between the App + the closures installed on its child widgets).
    pub fn new(app: &Application) -> Rc<App> {
        // 1. Load the CSS theme onto the default display.
        load_theme();

        // 2. Load the user's config from ~/.config/astra-vm/config.toml.
        let cfg = AppConfig::load();

        // Ensure the data directories exist (best-effort side effect of
        // resolving the paths).
        log::debug!("VM data dir: {}", config::vm_data_dir().display());
        log::debug!("ISO data dir: {}", config::iso_data_dir().display());
        log::debug!("QEMU data dir: {}", config::qemu_data_dir().display());

        // 3. Detect QEMU (portable install / system binary).
        let qemu_detected = detect_qemu(&cfg);
        log::info!(
            "QEMU detection: {}",
            if qemu_detected { "found" } else { "missing" }
        );

        // 4. Build the UI region structs.
        let header = Header::new();
        header.set_qemu_detected(qemu_detected);

        let sidebar = Sidebar::new();
        let main_area = MainArea::new();

        // 5. Assemble the layout:
        //      ┌──────────────────────────────────────────────────┐
        //      │ header (56px)                                    │
        //      ├──────────────┬───────────────────────────────────┤
        //      │ sidebar      │ main_area                          │
        //      │ (280px)      │ (fills the rest)                  │
        //      └──────────────┴───────────────────────────────────┘
        let body = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .homogeneous(false)
            .css_classes(["body"])
            .hexpand(true)
            .vexpand(true)
            .build();
        body.append(sidebar.widget());
        body.append(main_area.widget());

        let content = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .css_classes(["content"])
            .build();
        content.append(header.widget());
        content.append(&body);

        // 6. Build the libadwaita application window (1200x750).
        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("Astra VM")
            .default_width(1200)
            .default_height(750)
            .content(&content)
            .build();
        window.add_css_class("astra-vm");

        // 7. Wrap state in the App struct + Rc.
        let vm_list = RefCell::new(cfg.vms.clone());
        let selected_vm = RefCell::new(None);
        let config = RefCell::new(cfg);

        let app_struct = App {
            window,
            vm_list,
            selected_vm,
            sidebar,
            main_area,
            header,
            config,
        };
        let app_rc = Rc::new(app_struct);

        // 8. Wire UI signals using weak references so closures don't
        //    keep the App alive after the window is destroyed.

        // Header "Nouvelle VM" → App::show_new_vm_modal.
        let weak = Rc::downgrade(&app_rc);
        app_rc.header.set_on_new_vm(move || {
            if let Some(a) = weak.upgrade() {
                a.show_new_vm_modal();
            }
        });

        // Sidebar select → App::show_vm_details.
        let weak = Rc::downgrade(&app_rc);
        app_rc.sidebar.set_on_select(move |id: &str| {
            if let Some(a) = weak.upgrade() {
                a.show_vm_details(id);
            }
        });

        // Main area "Démarrer/Arrêter" → App::toggle_vm_running
        // (placeholder until subagent C implements real QEMU control).
        let weak = Rc::downgrade(&app_rc);
        app_rc.main_area.set_on_start(move |id: &str| {
            if let Some(a) = weak.upgrade() {
                a.toggle_vm_running(id);
            }
        });

        // Main area "Paramètres" → App::show_settings_modal (subagent D).
        let weak = Rc::downgrade(&app_rc);
        app_rc.main_area.set_on_settings(move |id: &str| {
            if let Some(a) = weak.upgrade() {
                a.show_settings_modal(id);
            }
        });

        // Main area "Capture" → subagent C will implement screenshot.
        let weak = Rc::downgrade(&app_rc);
        app_rc.main_area.set_on_capture(move |id: &str| {
            if let Some(a) = weak.upgrade() {
                a.capture_vm(id);
            }
        });

        // Main area "Supprimer" → App::delete_vm.
        let weak = Rc::downgrade(&app_rc);
        app_rc.main_area.set_on_delete(move |id: &str| {
            if let Some(a) = weak.upgrade() {
                a.delete_vm(id);
            }
        });

        // 9. Populate the sidebar with the initial VM list.
        app_rc.refresh_vm_list();

        // 10. Auto-select the first VM if any (fires the select callback
        //     which loads the VM into the main area).
        if let Some(first) = app_rc.vm_list.borrow().first().cloned() {
            app_rc.sidebar.select_vm(&first.id);
        }

        app_rc
    }

    /// Show the main window (called once during `connect_activate`).
    pub fn present(&self) {
        self.window.present();
    }

    /// Refresh the sidebar's VM list from the current `AppConfig`.
    /// Called after `add_vm` / `remove_vm` / config reload.
    pub fn refresh_vm_list(&self) {
        let vms = self.config.borrow().vms.clone();
        *self.vm_list.borrow_mut() = vms.clone();
        self.sidebar.set_vms(&vms);
    }

    /// Load a VM into the main area detail view + mark it as selected.
    pub fn show_vm_details(&self, vm_id: &str) {
        *self.selected_vm.borrow_mut() = Some(vm_id.to_string());
        let cfg = self.config.borrow();
        if let Some(vm) = cfg.get_vm(vm_id) {
            self.main_area.show_vm(vm);
        }
    }

    /// Open the "Nouvelle VM" modal. Stub implementation — subagent B
    /// replaces this with a real form (name + ISO picker + arch + RAM
    /// + CPU + disk inputs).
    pub fn show_new_vm_modal(&self) {
        log::info!("show_new_vm_modal — subagent B will implement the real form");

        let dialog = gtk4::Window::builder()
            .title("Nouvelle VM")
            .default_width(440)
            .default_height(260)
            .resizable(false)
            .build();
        dialog.set_transient_for(Some(&self.window));
        dialog.set_modal(true);

        let box_ = gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .css_classes(["new-vm-modal"])
            .build();

        let title = gtk4::Label::builder()
            .label("<span size=\"x-large\" weight=\"bold\">Nouvelle VM</span>")
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .build();
        box_.append(&title);

        let body = gtk4::Label::builder()
            .label("Le formulaire de cr\u{E9}ation de VM (nom, ISO, architecture,\nRAM, CPU, disque) sera impl\u{E9}ment\u{E9} par le sous-agent B.")
            .halign(gtk4::Align::Start)
            .wrap(true)
            .xalign(0.0)
            .css_classes(["new-vm-modal-hint"])
            .build();
        box_.append(&body);

        let close_btn = gtk4::Button::builder()
            .label("Fermer")
            .css_classes(["action-btn", "action-btn-secondary"])
            .halign(gtk4::Align::End)
            .build();
        let dialog_clone = dialog.clone();
        close_btn.connect_clicked(move |_| {
            dialog_clone.close();
        });
        box_.append(&close_btn);

        dialog.set_child(Some(&box_));
        dialog.present();
    }

    /// Toggle the running state of a VM. Placeholder — subagent C
    /// replaces the body with real QEMU spawn/kill logic.
    fn toggle_vm_running(&self, vm_id: &str) {
        // Resolve the disk path so subagent C can hand it to QEMU. We
        // log it here for visibility during the prototype phase.
        let disk_path = config::vm_disk_path(vm_id);
        let running = self.is_vm_running(vm_id);
        if running {
            log::info!(
                "Stopping VM {} (disk: {}) — subagent C will implement QEMU kill",
                vm_id,
                disk_path.display()
            );
            self.main_area.set_running(false);
            self.sidebar.set_vm_running(vm_id, false);
        } else {
            log::info!(
                "Starting VM {} (disk: {}) — subagent C will implement QEMU spawn",
                vm_id,
                disk_path.display()
            );
            // Touch last_used before launching.
            {
                let mut cfg = self.config.borrow_mut();
                if let Some(vm) = cfg.get_vm_mut(vm_id) {
                    vm.last_used = Some(chrono::Utc::now().timestamp());
                }
                if let Err(e) = cfg.save() {
                    log::error!("Failed to save config after start: {}", e);
                }
            }
            self.main_area.set_running(true);
            self.sidebar.set_vm_running(vm_id, true);
        }
    }

    /// Placeholder — subagent C will track real running state via the
    /// QEMU process handle. Returns `false` for now (no VM is running
    /// until the launcher is wired in).
    fn is_vm_running(&self, _vm_id: &str) -> bool {
        false
    }

    /// Show the settings modal for a VM. Stub — subagent D implements
    /// the real settings form (edit RAM, CPU, disk, ISO, 3D accel).
    fn show_settings_modal(&self, vm_id: &str) {
        log::info!(
            "show_settings_modal for VM {} — subagent D will implement",
            vm_id
        );
    }

    /// Capture a screenshot of the running VM. Stub — subagent C
    /// implements the real QEMU monitor-based screenshot.
    fn capture_vm(&self, vm_id: &str) {
        log::info!("capture_vm for VM {} — subagent C will implement", vm_id);
    }

    /// Delete a VM from the config + refresh the sidebar. The disk
    /// image is left in place (subagent C will add an option to wipe it).
    fn delete_vm(&self, vm_id: &str) {
        log::info!("Deleting VM {}", vm_id);
        {
            let mut cfg = self.config.borrow_mut();
            let removed = cfg.remove_vm(vm_id);
            if !removed {
                log::warn!("Delete requested for non-existent VM {}", vm_id);
                return;
            }
            if let Err(e) = cfg.save() {
                log::error!("Failed to save config after delete: {}", e);
            }
        }
        // Borrow released — safe to refresh.
        self.refresh_vm_list();

        // Clear selection if the deleted VM was selected.
        let was_selected = self
            .selected_vm
            .borrow()
            .as_deref()
            .map(|s| s == vm_id)
            .unwrap_or(false);
        if was_selected {
            *self.selected_vm.borrow_mut() = None;
            // Auto-select the first remaining VM if any.
            if let Some(first) = self.vm_list.borrow().first().cloned() {
                self.sidebar.select_vm(&first.id);
            }
        }
    }

    // --- Public accessors for subagents B, C, D ----------------------

    /// Read-only access to the underlying config. Returns a `Ref` so
    /// the caller can read the config without copying it.
    pub fn config(&self) -> std::cell::Ref<'_, AppConfig> {
        self.config.borrow()
    }

    /// Mutable access to the underlying config. Returns a `RefMut`.
    /// The caller is responsible for calling `save()` after mutation.
    pub fn config_mut(&self) -> std::cell::RefMut<'_, AppConfig> {
        self.config.borrow_mut()
    }

    /// Persist the current config to disk. Convenience wrapper around
    /// `AppConfig::save`.
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.borrow().save()
    }

    /// Borrow the main area widget (for subagent C to call
    /// `set_running` + `update_stats` directly).
    pub fn main_area(&self) -> &MainArea {
        &self.main_area
    }

    /// Borrow the sidebar widget (for subagent C to call
    /// `set_vm_running` directly).
    pub fn sidebar(&self) -> &Sidebar {
        &self.sidebar
    }

    /// Borrow the header widget (for subagent B to refresh the QEMU
    /// detection pill after installing QEMU portable).
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Borrow the application window (for subagents to parent their
    /// modals via `set_transient_for`).
    pub fn window(&self) -> &libadwaita::ApplicationWindow {
        &self.window
    }
}

/// Detect whether a usable QEMU binary is available.
///
/// Checks in this order:
///   1. `AppConfig::qemu_path` (if set + the file exists).
///   2. Portable QEMU at `~/.local/share/astra-vm/qemu/qemu-system-x86_64`.
///   3. System `qemu-system-x86_64` in PATH (try to spawn `--version`).
fn detect_qemu(cfg: &AppConfig) -> bool {
    // 1. Config-provided path.
    if let Some(p) = &cfg.qemu_path {
        if std::path::Path::new(p).exists() {
            log::debug!("QEMU detected via config path: {}", p);
            return true;
        }
    }

    // 2. Portable QEMU in the data dir.
    let portable = config::qemu_data_dir().join("qemu-system-x86_64");
    if portable.exists() {
        log::debug!("QEMU detected at portable path: {}", portable.display());
        return true;
    }

    // 3. System QEMU in PATH.
    if let Ok(output) = std::process::Command::new("qemu-system-x86_64")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            log::debug!("QEMU detected in PATH");
            return true;
        }
    }

    false
}

/// Load the Astra VM stylesheet onto the default `gdk::Display`.
///
/// Tries the installed system theme at `THEME_PATH` first; falls back
/// to the embedded stylesheet (`FALLBACK_CSS`) so a missing file never
/// crashes the app.
fn load_theme() {
    let provider = CssProvider::new();

    match std::fs::read_to_string(THEME_PATH) {
        Ok(css) => {
            provider.load_from_data(&css);
            log::info!("Loaded theme from {}", THEME_PATH);
        }
        Err(_) => {
            provider.load_from_data(FALLBACK_CSS);
            log::warn!(
                "Theme file missing at {} — using embedded fallback",
                THEME_PATH
            );
        }
    }

    let display = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => {
            log::error!("Could not get default GdkDisplay — theme not applied");
            return;
        }
    };

    // gtk4-rs 0.9 free function (replaces the deprecated
    // `StyleContext::add_provider_for_display`).
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    env_logger::init();
    log::info!("Astra VM v0.4.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");
    libadwaita::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        let astra_app = App::new(app);
        astra_app.present();
        log::info!("Astra VM ready");
    });

    app.run();
}
