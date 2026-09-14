// vm_item.rs - A single VM row rendered inside the sidebar list.
//
// Each item is a horizontal `gtk4::Box`:
//
//   [32x32 gradient icon] [name]                [status dot]
//
// The icon is a `gtk4::Label` wrapped in a fixed-size `gtk4::Box` with
// a CSS class derived from the VM's `OsFamily` (`.os-astra`, `.os-windows`,
// `.os-linux`, `.os-arm`, `.os-unknown`) — the gradient palette rules
// live in `astra-vm.css`.
//
// The status dot is a `gtk4::Label` showing "● En cours" (running) or
// "● Arrêtée" (stopped). The state is set by `set_running` and defaults
// to stopped.
//
// The whole row is wrapped in a `gtk4::Button` (flat, no chrome) so
// `App` can hook `clicked` to drive the sidebar selection + main area
// detail view. We expose the underlying button via `widget()` so the
// sidebar can pack many of these into a `gtk4::Box`.

use crate::config::VmConfig;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// One VM row widget.
#[derive(Clone)]
pub struct VmItem {
    /// Flat button wrapping the row contents — receives clicks.
    button: Button,
    /// The status label so `set_running` can update its text + CSS class.
    status_label: Label,
    /// Current running state, mirrored so callers can query it.
    running: Rc<RefCell<bool>>,
    /// The VM id this item represents — recovered on click by `Sidebar`.
    id: Rc<String>,
}

impl VmItem {
    /// Build a VM item for the given config. Defaults to stopped state.
    pub fn new(vm: &VmConfig) -> Self {
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .css_classes(["vm-item-inner"])
            .margin_start(8)
            .margin_end(8)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        // --- Icon (32x32 gradient box) -------------------------------
        let os = vm.os_family();
        let icon_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .halign(Align::Center)
            .valign(Align::Center)
            .width_request(32)
            .height_request(32)
            .css_classes(["vm-item-icon", os.css_class()])
            .build();
        let icon = Label::builder()
            .label(os.emoji())
            .css_classes(["vm-item-icon-text"])
            .build();
        icon_box.append(&icon);
        row.append(&icon_box);

        // --- Name + status (vertical stack) --------------------------
        let text_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .halign(Align::Fill)
            .build();

        let name = Label::builder()
            .label(&vm.name)
            .xalign(0.0)
            .halign(Align::Start)
            .css_classes(["vm-item-name"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        text_box.append(&name);

        let status_label = Label::builder()
            .label("\u{25CF} Arr\u{EA}t\u{E9}e") // ● Arrêtée
            .xalign(0.0)
            .halign(Align::Start)
            .css_classes(["vm-item-status", "status-stopped"])
            .build();
        text_box.append(&status_label);
        row.append(&text_box);

        // --- Wrap the row in a flat Button ----------------------------
        let button = Button::builder()
            .css_classes(["vm-item"])
            .child(&row)
            .hexpand(true)
            .halign(Align::Fill)
            .build();
        // Tag the button with the VM id via `widget_name` so the sidebar
        // can recover which VM was clicked.
        button.set_widget_name(&vm.id);

        let running = Rc::new(RefCell::new(false));
        let id = Rc::new(vm.id.clone());

        Self {
            button,
            status_label,
            running,
            id,
        }
    }

    /// Update the running-state indicator. Toggles the status text
    /// between "● En cours" (running) and "● Arrêtée" (stopped) and
    /// swaps the CSS class (`status-running` ↔ `status-stopped`).
    pub fn set_running(&self, running: bool) {
        *self.running.borrow_mut() = running;
        if running {
            self.status_label.set_label("\u{25CF} En cours"); // ● En cours
            self.status_label.remove_css_class("status-stopped");
            self.status_label.add_css_class("status-running");
        } else {
            self.status_label.set_label("\u{25CF} Arr\u{EA}t\u{E9}e"); // ● Arrêtée
            self.status_label.remove_css_class("status-running");
            self.status_label.add_css_class("status-stopped");
        }
    }

    /// Mark this item as the active selection (adds the `active` CSS
    /// class — drives the gradient highlight rule in CSS).
    pub fn set_active(&self, active: bool) {
        if active {
            self.button.add_css_class("active");
        } else {
            self.button.remove_css_class("active");
        }
    }

    /// Borrow the underlying button for click wiring + layout parenting.
    pub fn widget(&self) -> &Button {
        &self.button
    }

    /// The VM id this item represents.
    pub fn vm_id(&self) -> &str {
        self.id.as_str()
    }
}
