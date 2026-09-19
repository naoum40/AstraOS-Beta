// main_area.rs - Astra VM right pane (VM details + actions + config + stats).
//
// A scrollable vertical `gtk4::Box` laid out top → bottom:
//
//   1. VM detail header (icon 64x64 + title + subtitle + status badge)
//   2. Actions row (Démarrer / Paramètres / Capture / Supprimer)
//   3. Configuration section (grid: Architecture, CPU, RAM, Disque, ISO,
//      3D acceleration)
//   4. Stats section (hidden by default — shown when VM is running):
//      CPU, RAM, Disk I/O, Uptime
//
// State held:
//   - `current_id`: id of the VM currently displayed (or `None`).
//   - `running`: whether the displayed VM is running — drives the
//     stats section visibility + action button labels.
//   - Label widgets for the detail header, config grid, and stats so
//     `show_vm` + `update_stats` can mutate them in place.
//
// Public API:
//   - `new() -> Self`
//   - `show_vm(&self, vm: &VmConfig)` — load VM details into the view.
//   - `set_running(&self, running: bool)` — toggle stats visibility +
//     action button labels.
//   - `update_stats(&self, cpu, ram_mb, disk_mbps, uptime_sec)` —
//     refresh the stats cards (no-op when running is false).

use crate::config::VmConfig;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Grid, Label, Orientation, PolicyType, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;

/// The main area widget.
#[derive(Clone)]
pub struct MainArea {
    container: ScrolledWindow,
    root: GtkBox,
    // --- Detail header widgets ---
    detail_icon: Label,
    detail_icon_box: GtkBox,
    detail_title: Label,
    detail_subtitle: Label,
    detail_status: Label,
    // --- Action buttons ---
    start_btn: Button,
    settings_btn: Button,
    capture_btn: Button,
    delete_btn: Button,
    // --- Config grid value labels ---
    cfg_arch: Label,
    cfg_cpu: Label,
    cfg_ram: Label,
    cfg_disk: Label,
    cfg_iso: Label,
    cfg_3d: Label,
    // --- Stats section container + value labels ---
    stats_section: GtkBox,
    stat_cpu: Label,
    stat_ram: Label,
    stat_disk: Label,
    stat_uptime: Label,
    // --- State ---
    current_id: Rc<RefCell<Option<String>>>,
    running: Rc<RefCell<bool>>,
    // --- Callbacks (installed by App::new) ---
    on_start: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    on_settings: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    on_capture: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    on_delete: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl MainArea {
    /// Build the main area. Empty until `show_vm` is called.
    pub fn new() -> Self {
        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(18)
            .margin_start(28)
            .margin_end(28)
            .margin_top(24)
            .margin_bottom(24)
            .css_classes(["main"])
            .build();

        let container = ScrolledWindow::builder()
            .child(&root)
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .css_classes(["main-scroll"])
            .build();

        // --- 1. VM detail header -------------------------------------
        let detail_header = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(16)
            .css_classes(["vm-detail-header"])
            .build();

        let detail_icon_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .halign(Align::Center)
            .valign(Align::Center)
            .width_request(64)
            .height_request(64)
            .css_classes(["vm-detail-icon", "os-unknown"])
            .build();
        let detail_icon = Label::builder()
            .label("\u{1F4E6}") // 📦 (default Unknown)
            .css_classes(["vm-detail-icon-text"])
            .build();
        detail_icon_box.append(&detail_icon);
        detail_header.append(&detail_icon_box);

        let title_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .halign(Align::Fill)
            .build();
        let detail_title = Label::builder()
            .label("Aucune VM s\u{E9}lectionn\u{E9}e") // Aucune VM sélectionnée
            .halign(Align::Start)
            .css_classes(["vm-detail-title"])
            .build();
        let detail_subtitle = Label::builder()
            .label("S\u{E9}lectionnez une VM dans la liste.") // Sélectionnez une VM dans la liste.
            .halign(Align::Start)
            .css_classes(["vm-detail-subtitle"])
            .build();
        title_box.append(&detail_title);
        title_box.append(&detail_subtitle);
        detail_header.append(&title_box);

        let detail_status = Label::builder()
            .label("\u{25CF} Arr\u{EA}t\u{E9}e") // ● Arrêtée
            .valign(Align::Center)
            .css_classes(["vm-detail-status", "status-stopped"])
            .build();
        detail_header.append(&detail_status);

        root.append(&detail_header);

        // --- 2. Actions row -----------------------------------------
        let actions = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .halign(Align::Start)
            .css_classes(["actions"])
            .build();

        let start_btn = Button::builder()
            .label("\u{25B6} D\u{E9}marrer") // ▶ Démarrer
            .css_classes(["action-btn", "action-btn-primary"])
            .sensitive(false)
            .build();
        let settings_btn = Button::builder()
            .label("Param\u{E8}tres") // Paramètres
            .css_classes(["action-btn", "action-btn-secondary"])
            .sensitive(false)
            .build();
        let capture_btn = Button::builder()
            .label("Capture")
            .css_classes(["action-btn", "action-btn-secondary"])
            .sensitive(false)
            .build();
        let delete_btn = Button::builder()
            .label("Supprimer")
            .css_classes(["action-btn", "action-btn-danger"])
            .sensitive(false)
            .build();
        actions.append(&start_btn);
        actions.append(&settings_btn);
        actions.append(&capture_btn);
        actions.append(&delete_btn);
        root.append(&actions);

        // --- 3. Configuration section --------------------------------
        let cfg_section = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .css_classes(["section"])
            .build();
        let cfg_title = Label::builder()
            .label("Configuration")
            .halign(Align::Start)
            .css_classes(["section-title"])
            .build();
        cfg_section.append(&cfg_title);

        let cfg_grid = Grid::builder()
            .orientation(Orientation::Vertical)
            .row_spacing(6)
            .column_spacing(24)
            .css_classes(["config-grid"])
            .build();
        // Row layout: label (col 0) | value (col 1)
        let cfg_arch = build_config_row(&cfg_grid, 0, "Architecture");
        let cfg_cpu = build_config_row(&cfg_grid, 1, "CPU");
        let cfg_ram = build_config_row(&cfg_grid, 2, "RAM");
        let cfg_disk = build_config_row(&cfg_grid, 3, "Disque");
        let cfg_iso = build_config_row(&cfg_grid, 4, "ISO");
        let cfg_3d = build_config_row(&cfg_grid, 5, "Acc\u{E9}l\u{E9}ration 3D"); // Accélération 3D
        cfg_section.append(&cfg_grid);
        root.append(&cfg_section);

        // --- 4. Stats section (hidden by default) -------------------
        let stats_section = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .css_classes(["section", "stats-section"])
            .build();
        let stats_title = Label::builder()
            .label("Statistiques")
            .halign(Align::Start)
            .css_classes(["section-title"])
            .build();
        stats_section.append(&stats_title);

        let stats_grid = Grid::builder()
            .orientation(Orientation::Vertical)
            .row_spacing(10)
            .column_spacing(12)
            .css_classes(["stats-grid"])
            .build();
        let stat_cpu = build_stat_card(&stats_grid, 0, "CPU");
        let stat_ram = build_stat_card(&stats_grid, 1, "RAM");
        let stat_disk = build_stat_card(&stats_grid, 2, "Disque I/O");
        let stat_uptime = build_stat_card(&stats_grid, 3, "Uptime");
        stats_section.append(&stats_grid);

        // Hidden by default until `set_running(true)`.
        stats_section.set_visible(false);
        root.append(&stats_section);

        // --- State + callbacks --------------------------------------
        let current_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let running: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let on_start: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let on_settings: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let on_capture: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let on_delete: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let main_area = Self {
            container: container.clone(),
            root: root.clone(),
            detail_icon: detail_icon.clone(),
            detail_icon_box: detail_icon_box.clone(),
            detail_title: detail_title.clone(),
            detail_subtitle: detail_subtitle.clone(),
            detail_status: detail_status.clone(),
            start_btn: start_btn.clone(),
            settings_btn: settings_btn.clone(),
            capture_btn: capture_btn.clone(),
            delete_btn: delete_btn.clone(),
            cfg_arch: cfg_arch.clone(),
            cfg_cpu: cfg_cpu.clone(),
            cfg_ram: cfg_ram.clone(),
            cfg_disk: cfg_disk.clone(),
            cfg_iso: cfg_iso.clone(),
            cfg_3d: cfg_3d.clone(),
            stats_section: stats_section.clone(),
            stat_cpu: stat_cpu.clone(),
            stat_ram: stat_ram.clone(),
            stat_disk: stat_disk.clone(),
            stat_uptime: stat_uptime.clone(),
            current_id: current_id.clone(),
            running: running.clone(),
            on_start: on_start.clone(),
            on_settings: on_settings.clone(),
            on_capture: on_capture.clone(),
            on_delete: on_delete.clone(),
        };

        // Wire action buttons. Each callback looks up the current VM id
        // from `current_id` and forwards to the registered closure.
        let ma_for_start = main_area.clone();
        start_btn.connect_clicked(move |_| {
            let id_opt = ma_for_start.current_id.borrow().clone();
            if let Some(id) = id_opt {
                if let Some(cb) = ma_for_start.on_start.borrow().as_ref() {
                    cb(&id);
                }
            }
        });

        let ma_for_settings = main_area.clone();
        settings_btn.connect_clicked(move |_| {
            let id_opt = ma_for_settings.current_id.borrow().clone();
            if let Some(id) = id_opt {
                if let Some(cb) = ma_for_settings.on_settings.borrow().as_ref() {
                    cb(&id);
                }
            }
        });

        let ma_for_capture = main_area.clone();
        capture_btn.connect_clicked(move |_| {
            let id_opt = ma_for_capture.current_id.borrow().clone();
            if let Some(id) = id_opt {
                if let Some(cb) = ma_for_capture.on_capture.borrow().as_ref() {
                    cb(&id);
                }
            }
        });

        let ma_for_delete = main_area.clone();
        delete_btn.connect_clicked(move |_| {
            let id_opt = ma_for_delete.current_id.borrow().clone();
            if let Some(id) = id_opt {
                if let Some(cb) = ma_for_delete.on_delete.borrow().as_ref() {
                    cb(&id);
                }
            }
        });

        main_area
    }

    /// Load a VM into the detail view. Updates icon, title, subtitle,
    /// status badge, config grid, and enables action buttons.
    pub fn show_vm(&self, vm: &VmConfig) {
        *self.current_id.borrow_mut() = Some(vm.id.clone());

        let os = vm.os_family();

        // --- Detail header ---
        self.detail_icon.set_label(os.emoji());
        // Swap the OS class on the icon box.
        for cls in ["os-astra", "os-windows", "os-linux", "os-arm", "os-unknown"] {
            self.detail_icon_box.remove_css_class(cls);
        }
        self.detail_icon_box.add_css_class(os.css_class());
        self.detail_title.set_label(&vm.name);
        self.detail_subtitle.set_label(&format!(
            "{} \u{2022} {} \u{2022} {}",
            vm.architecture, vm.cpu_cores, vm.ram_mb
        ));

        // Status badge reflects `running` state (don't override here
        // unless we're loading a fresh VM — set_running handles it).
        if !*self.running.borrow() {
            self.detail_status.set_label("\u{25CF} Arr\u{EA}t\u{E9}e"); // ● Arrêtée
            self.detail_status.remove_css_class("status-running");
            self.detail_status.add_css_class("status-stopped");
        }

        // --- Config grid ---
        self.cfg_arch.set_label(&vm.architecture);
        self.cfg_cpu
            .set_label(&format!("{} c\u{15C}urs", vm.cpu_cores)); // {} cœurs
        self.cfg_ram.set_label(&format!("{} Mo", vm.ram_mb));
        self.cfg_disk.set_label(&format!("{} Go", vm.disk_gb));
        self.cfg_iso.set_label(&vm.iso_path);
        // 3D acceleration default = true for x86_64, false for aarch64.
        let accel_3d = vm.architecture.eq_ignore_ascii_case("x86_64");
        self.cfg_3d.set_label(if accel_3d {
            "Activ\u{E9}e"
        } else {
            "D\u{E9}sactiv\u{E9}e"
        }); // Activée / Désactivée

        // --- Enable action buttons ---
        self.start_btn.set_sensitive(true);
        self.settings_btn.set_sensitive(true);
        self.capture_btn.set_sensitive(*self.running.borrow());
        self.delete_btn.set_sensitive(true);
    }

    /// Toggle the running state. Reveals/hides the stats section and
    /// updates the action button labels + status badge.
    pub fn set_running(&self, running: bool) {
        *self.running.borrow_mut() = running;
        self.stats_section.set_visible(running);
        if running {
            self.detail_status.set_label("\u{25CF} En cours"); // ● En cours
            self.detail_status.remove_css_class("status-stopped");
            self.detail_status.add_css_class("status-running");
            self.start_btn.set_label("\u{23F9} Arr\u{EA}ter"); // ⏹ Arrêter
        } else {
            self.detail_status.set_label("\u{25CF} Arr\u{EA}t\u{E9}e"); // ● Arrêtée
            self.detail_status.remove_css_class("status-running");
            self.detail_status.add_css_class("status-stopped");
            self.start_btn.set_label("\u{25B6} D\u{E9}marrer"); // ▶ Démarrer
        }
        self.capture_btn.set_sensitive(running);
    }

    /// Update the stats cards. No-op when `running` is false (the
    /// stats section is hidden anyway).
    pub fn update_stats(&self, cpu: u32, ram_mb: u32, disk_mbps: u32, uptime_sec: u64) {
        if !*self.running.borrow() {
            return;
        }
        self.stat_cpu.set_label(&format!("{} %", cpu));
        self.stat_ram.set_label(&format!("{} Mo", ram_mb));
        self.stat_disk.set_label(&format!("{} Mo/s", disk_mbps));
        self.stat_uptime.set_label(&format_uptime(uptime_sec));
    }

    /// Install the "Démarrer/Arrêter" action callback.
    pub fn set_on_start<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_start.borrow_mut() = Some(Box::new(f));
    }

    /// Install the "Paramètres" action callback.
    pub fn set_on_settings<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_settings.borrow_mut() = Some(Box::new(f));
    }

    /// Install the "Capture" action callback.
    pub fn set_on_capture<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_capture.borrow_mut() = Some(Box::new(f));
    }

    /// Install the "Supprimer" action callback.
    pub fn set_on_delete<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_delete.borrow_mut() = Some(Box::new(f));
    }

    /// Borrow the underlying container for layout parenting.
    pub fn widget(&self) -> &ScrolledWindow {
        &self.container
    }

    /// Borrow the inner root box (used by `App` if it needs to inject
    /// additional sections, e.g. a console widget).
    #[allow(dead_code)]
    pub fn root(&self) -> &GtkBox {
        &self.root
    }
}

/// Build one config-grid row: a label on column 0 + a value label on
/// column 1 at the given row index. Returns the value label so the
/// caller can update it later (e.g. `cfg_iso.set_label(...)`).
fn build_config_row(grid: &Grid, row: i32, label: &str) -> Label {
    let key = Label::builder()
        .label(label)
        .halign(Align::Start)
        .css_classes(["config-label"])
        .build();
    let value = Label::builder()
        .label("\u{2014}") // —
        .halign(Align::Start)
        .hexpand(true)
        .css_classes(["config-value"])
        .selectable(true)
        .ellipsize(gtk4::pango::EllipsizeMode::Middle)
        .build();
    grid.attach(&key, 0, row, 1, 1);
    grid.attach(&value, 1, row, 1, 1);
    value
}

/// Build one stat card: a vertical box with the stat name (label) +
/// the big value, attached to the stats grid at the given row.
fn build_stat_card(grid: &Grid, row: i32, label: &str) -> Label {
    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .hexpand(true)
        .halign(Align::Fill)
        .css_classes(["stat-card"])
        .build();
    let name = Label::builder()
        .label(label)
        .halign(Align::Start)
        .css_classes(["stat-label"])
        .build();
    let value = Label::builder()
        .label("\u{2014}") // —
        .halign(Align::Start)
        .css_classes(["stat-value"])
        .build();
    card.append(&name);
    card.append(&value);
    grid.attach(&card, 0, row, 1, 1);
    value
}

/// Format a duration in seconds as `Hh Mm Ss` (e.g. `1h 12m 04s`).
fn format_uptime(total_sec: u64) -> String {
    let h = total_sec / 3600;
    let m = (total_sec % 3600) / 60;
    let s = total_sec % 60;
    format!("{}h {:02}m {:02}s", h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_uptime_zero() {
        assert_eq!(format_uptime(0), "0h 00m 00s");
    }

    #[test]
    fn format_uptime_under_minute() {
        assert_eq!(format_uptime(7), "0h 00m 07s");
    }

    #[test]
    fn format_uptime_with_minutes() {
        assert_eq!(format_uptime(72), "0h 01m 12s");
    }

    #[test]
    fn format_uptime_with_hours() {
        assert_eq!(format_uptime(3600 + 12 * 60 + 4), "1h 12m 04s");
    }
}
