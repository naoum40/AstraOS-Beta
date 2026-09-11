// system.rs - System page.
//
// Surfaces system-level knobs and status readouts:
//
//   - Performance mode (\u{00c9}co / Balanced / Performance) - GtkComboBoxText
//   - Battery saver toggle                                    - GtkSwitch
//   - Storage info (text "XX GB used / YY GB total")          - GtkLabel
//   - Astra Defender status (green dot + "Protected" text)    - GtkBox row
//
// The performance-mode combo + battery-saver switch write back to the
// shared `SettingsConfig` cell and persist immediately. The storage
// readout is a static placeholder for now (real disk usage probing
// will be added once we wire up a `sysinfo`-based backend). Astra
// Defender status is also static ("Protected" green dot) until the
// real-time AV daemon is hooked up.

use crate::config::SettingsConfig;
use glib::Propagation;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ComboBoxText, Label, Orientation, Switch};
use std::cell::RefCell;
use std::rc::Rc;

type SharedCfg = Rc<RefCell<SettingsConfig>>;

/// System page - performance + battery + storage + defender readouts.
pub struct SystemPage {
    container: GtkBox,
    cfg: SharedCfg,
}

impl SystemPage {
    /// Build the page. `cfg` is the shared config cell owned by the
    /// content area.
    pub fn new(cfg: &SharedCfg) -> Self {
        let cfg_cell = cfg.clone();

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(18)
            .margin_top(36)
            .margin_bottom(36)
            .margin_start(48)
            .margin_end(48)
            .css_classes(["astra-page", "astra-system"])
            .build();

        // Page header.
        let header = Label::builder()
            .label("System")
            .css_classes(["astra-page-title"])
            .halign(gtk4::Align::Start)
            .build();
        container.append(&header);

        // ---- 1. Performance mode combo ---------------------------
        let perf_row = build_row("Performance mode", "\u{00c9}co / Balanced / Performance");
        let perf_combo = ComboBoxText::builder()
            .css_classes(["astra-combo"])
            .valign(gtk4::Align::Center)
            .build();
        perf_combo.append(Some("eco"), "\u{00c9}co");
        perf_combo.append(Some("balanced"), "Balanced");
        perf_combo.append(Some("performance"), "Performance");
        perf_combo.set_active_id(Some(&cfg_cell.borrow().performance_mode));
        {
            let cfg_clone = cfg_cell.clone();
            perf_combo.connect_changed(move |combo| {
                if let Some(id) = combo.active_id() {
                    let id_str = id.to_string();
                    cfg_clone.borrow_mut().performance_mode = id_str.clone();
                    if let Err(e) = cfg_clone.borrow().save() {
                        log::warn!("Failed to save performance mode: {}", e);
                    }
                    log::info!("Performance mode set to {}", id_str);
                }
            });
        }
        perf_row.append(&perf_combo);
        container.append(&perf_row);

        // ---- 2. Battery saver toggle -----------------------------
        let battery_row = build_row("Battery saver", "Reduces background activity");
        // NOTE: battery_saver is not in SettingsConfig yet (ISO 3 may
        // add it as a runtime-only flag). The switch starts OFF and
        // just logs the change for now.
        let battery_switch = Switch::builder()
            .active(false)
            .css_classes(["astra-switch"])
            .valign(gtk4::Align::Center)
            .build();
        battery_switch.connect_state_set(|_, state| {
            log::info!("Battery saver set to {} (placeholder)", state);
            Propagation::Proceed
        });
        battery_row.append(&battery_switch);
        container.append(&battery_row);

        // ---- 3. Storage info -------------------------------------
        let storage_row = build_row("Storage", "Disk usage summary");
        let storage_text = Label::builder()
            .label("Storage: 12.5 GB used / 64 GB total")
            .css_classes(["astra-storage"])
            .valign(gtk4::Align::Center)
            .build();
        storage_row.append(&storage_text);
        container.append(&storage_row);

        // ---- 4. Astra Defender status ----------------------------
        // The green dot is a CSS-painted widget (see `.astra-defender-dot`
        // in settings.css). The text label carries the green-circle
        // emoji "🟢" so the status is unambiguous even when the CSS dot
        // fails to load (e.g. on bare GTK4 without our stylesheet).
        let defender_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .css_classes(["astra-row", "astra-defender-row"])
            .build();
        let dot = Label::builder()
            .label("")
            .css_classes(["astra-defender-dot"])
            .width_request(12)
            .height_request(12)
            .valign(gtk4::Align::Center)
            .build();
        let status_text = Label::builder()
            .label("\u{1F7E2} Protected")
            .css_classes(["astra-defender-status"])
            .halign(gtk4::Align::Start)
            .build();
        defender_row.append(&dot);
        defender_row.append(&status_text);
        container.append(&defender_row);

        Self {
            container,
            cfg: cfg_cell,
        }
    }

    /// Borrow the root `GtkBox`.
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Borrow the shared config cell (kept for future pages that may
    /// want to read the live config).
    #[allow(dead_code)]
    pub fn config(&self) -> &SharedCfg {
        &self.cfg
    }
}

/// Build a horizontal "row" widget: a titled subtitle on the left
/// (hexpand) + space on the right for a control.
fn build_row(title: &str, subtitle: &str) -> GtkBox {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["astra-row"])
        .build();

    let labels = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();

    let t = Label::builder()
        .label(title)
        .css_classes(["astra-row-title"])
        .halign(gtk4::Align::Start)
        .build();
    let s = Label::builder()
        .label(subtitle)
        .css_classes(["astra-row-subtitle"])
        .halign(gtk4::Align::Start)
        .build();

    labels.append(&t);
    labels.append(&s);
    row.append(&labels);
    row
}
