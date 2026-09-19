//! "New VM" modal dialog.
//!
//! Collects VM configuration from the user (name, ISO, arch, RAM, CPU, disk)
//! and calls a callback with the resulting `VmConfig`.

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, ComboBoxText, Entry, Label, Orientation, SpinButton, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::VmConfig;

/// Modal dialog for creating a new VM.
pub struct NewVmModal {
    window: Window,
    on_create: Rc<RefCell<Option<Box<dyn Fn(VmConfig)>>>>,
}

impl NewVmModal {
    /// Build the modal window (hidden by default).
    pub fn new(parent: &impl IsA<gtk4::Native>) -> Self {
        let window = Window::builder()
            .title("✦ Nouvelle VM")
            .default_width(480)
            .modal(true)
            .transient_for(parent)
            .build();
        window.add_css_class("astra-vm-modal");

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_top(32)
            .margin_bottom(32)
            .margin_start(32)
            .margin_end(32)
            .build();
        window.set_child(Some(&container));

        // Title
        let title = Label::builder()
            .label("✦ Nouvelle VM")
            .css_classes(["modal-title"])
            .halign(gtk4::Align::Start)
            .build();
        container.append(&title);

        // Name field
        let name_entry = Entry::builder()
            .placeholder_text("Ma VM AstraOS")
            .text("AstraOS Test")
            .build();
        container.append(&field_label("Nom de la VM"));
        container.append(&name_entry);

        // ISO dropdown
        let iso_combo = ComboBoxText::builder().build();
        iso_combo.append_text("astraos-0.4.0-x86_64.iso");
        iso_combo.append_text("astraos-0.3.0-x86_64.iso");
        iso_combo.append_text("windows-11.iso");
        iso_combo.append_text("archlinux-2026.09.iso");
        iso_combo.append_text("+ Télécharger une ISO...");
        iso_combo.set_active(Some(0));
        container.append(&field_label("Système d'exploitation (ISO)"));
        container.append(&iso_combo);

        // Architecture dropdown
        let arch_combo = ComboBoxText::builder().build();
        arch_combo.append_text("x86_64 (Intel/AMD)");
        arch_combo.append_text("aarch64 (ARM — RTX Spark, Raspberry Pi)");
        arch_combo.set_active(Some(0));
        container.append(&field_label("Architecture"));
        container.append(&arch_combo);

        // RAM spin
        let ram_spin = SpinButton::with_range(512.0, 16384.0, 512.0);
        ram_spin.set_value(4096.0);
        container.append(&field_label("RAM (Mo)"));
        container.append(&ram_spin);

        // CPU spin
        let cpu_spin = SpinButton::with_range(1.0, 8.0, 1.0);
        cpu_spin.set_value(2.0);
        container.append(&field_label("CPU (cœurs)"));
        container.append(&cpu_spin);

        // Disk spin
        let disk_spin = SpinButton::with_range(5.0, 100.0, 5.0);
        disk_spin.set_value(20.0);
        container.append(&field_label("Disque (Go)"));
        container.append(&disk_spin);

        // Actions
        let actions = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .halign(gtk4::Align::End)
            .margin_top(8)
            .build();

        let cancel_btn = Button::builder()
            .label("Annuler")
            .css_classes(["action-btn", "secondary"])
            .build();
        let create_btn = Button::builder()
            .label("✦ Créer")
            .css_classes(["action-btn", "primary"])
            .build();
        actions.append(&cancel_btn);
        actions.append(&create_btn);
        container.append(&actions);

        let on_create = Rc::new(RefCell::new(None::<Box<dyn Fn(VmConfig)>>));

        // Wire Cancel
        let window_clone = window.clone();
        cancel_btn.connect_clicked(move |_| {
            window_clone.set_visible(false);
        });

        // Wire Create
        let window_clone = window.clone();
        let on_create_clone = on_create.clone();
        let name_e = name_entry.clone();
        let iso_c = iso_combo.clone();
        let arch_c = arch_combo.clone();
        let ram_s = ram_spin.clone();
        let cpu_s = cpu_spin.clone();
        let disk_s = disk_spin.clone();
        create_btn.connect_clicked(move |_| {
            let name = name_e.text().to_string();
            if name.is_empty() {
                return;
            }
            let iso = iso_c.active_text().unwrap_or_default().to_string();
            let arch_label = arch_c.active_text().unwrap_or_default().to_string();
            let architecture = if arch_label.starts_with("aarch64") {
                "aarch64".to_string()
            } else {
                "x86_64".to_string()
            };
            let vm = VmConfig {
                id: format!("vm-{}", chrono::Utc::now().timestamp()),
                name,
                iso_path: iso,
                architecture,
                ram_mb: ram_s.value() as u32,
                cpu_cores: cpu_s.value() as u32,
                disk_gb: disk_s.value() as u32,
                created_at: chrono::Utc::now().timestamp(),
                last_used: None,
            };
            if let Some(cb) = on_create_clone.borrow_mut().take() {
                cb(vm);
            }
            window_clone.set_visible(false);
        });

        Self { window, on_create }
    }

    /// Show the modal.
    pub fn show(&self) {
        self.window.present();
    }

    /// Register a callback invoked when the user clicks "✦ Créer".
    pub fn set_on_create(&self, callback: Box<dyn Fn(VmConfig)>) {
        *self.on_create.borrow_mut() = Some(callback);
    }
}

fn field_label(text: &str) -> Label {
    Label::builder()
        .label(text)
        .css_classes(["field-label"])
        .halign(gtk4::Align::Start)
        .build()
}
