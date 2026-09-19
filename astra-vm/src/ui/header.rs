// header.rs - Astra VM top header bar.
//
// A 56px-tall horizontal `gtk4::Box` laid out left → right:
//
//   [✦ Astra VM logo]            [QEMU status pill]  [+ Nouvelle VM]
//
// The logo is a `gtk4::Label` with markup (`<span class="logo-text">`)
// rendering the gradient + Outfit font rules from `astra-vm.css`.
//
// The QEMU status pill is a flat `gtk4::Button` whose label + CSS class
// flip between two states:
//   - found   → "🐧 QEMU Portable détecté"   + class `qemu-found`
//   - missing → "⚠ QEMU manquant"            + class `qemu-missing`
//
// The "Nouvelle VM" button on the right is the primary action — its
// `clicked` signal is forwarded to a callback installed by `App::new`.
//
// All state lives behind `Rc<RefCell<…>>` so a single `Header` clone
// can be held by the parent window + by the closure that fires the
// "new VM" callback.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// The header widget.
#[derive(Clone)]
pub struct Header {
    widget: GtkBox,
    qemu_btn: Button,
    /// "New VM" callback — installed by `App::new`. Receives no args;
    /// the parent opens the new-VM modal in its own scope.
    on_new_vm: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl Header {
    /// Build the header. Default QEMU state = "missing" until
    /// `set_qemu_detected` is called by `App::new` after detection.
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .hexpand(true)
            .halign(gtk4::Align::Fill)
            .css_classes(["header"])
            .height_request(56)
            .build();

        // --- Logo ----------------------------------------------------
        // Markup is styled by `.logo .logo-text` rules in the CSS.
        let logo = Label::builder()
            .label("<span class=\"logo-text\">\u{2726} Astra VM</span>") // ✦ Astra VM
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .margin_start(16)
            .css_classes(["logo"])
            .build();
        widget.append(&logo);

        // --- Spacer (fills the middle) -------------------------------
        let spacer = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true)
            .build();
        widget.append(&spacer);

        // --- QEMU status pill ----------------------------------------
        let qemu_btn = Button::builder()
            .label("\u{26A0} QEMU manquant") // ⚠ QEMU manquant
            .css_classes(["qemu-pill", "qemu-missing"])
            .valign(gtk4::Align::Center)
            .margin_end(8)
            .build();
        widget.append(&qemu_btn);

        // --- "Nouvelle VM" primary button ---------------------------
        let new_vm_btn = Button::builder()
            .label("+ Nouvelle VM")
            .css_classes(["action-btn", "action-btn-primary"])
            .valign(gtk4::Align::Center)
            .margin_end(16)
            .build();
        widget.append(&new_vm_btn);

        let on_new_vm: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        let header = Self {
            widget: widget.clone(),
            qemu_btn: qemu_btn.clone(),
            on_new_vm: on_new_vm.clone(),
        };

        // Forward the "Nouvelle VM" click to the registered callback.
        let header_for_cb = header.clone();
        new_vm_btn.connect_clicked(move |_| {
            if let Some(cb) = header_for_cb.on_new_vm.borrow().as_ref() {
                cb();
            }
        });

        header
    }

    /// Update the QEMU status pill to reflect detection result.
    ///
    /// `true`  → "🐧 QEMU Portable détecté"  + `qemu-found` class.
    /// `false` → "⚠ QEMU manquant"          + `qemu-missing` class.
    pub fn set_qemu_detected(&self, detected: bool) {
        if detected {
            self.qemu_btn
                .set_label("\u{1F427} QEMU Portable d\u{E9}tect\u{E9}"); // 🐧 QEMU Portable détecté
            self.qemu_btn.remove_css_class("qemu-missing");
            self.qemu_btn.add_css_class("qemu-found");
        } else {
            self.qemu_btn.set_label("\u{26A0} QEMU manquant"); // ⚠ QEMU manquant
            self.qemu_btn.remove_css_class("qemu-found");
            self.qemu_btn.add_css_class("qemu-missing");
        }
    }

    /// Install the "new VM" click callback. The closure receives no
    /// arguments — the parent decides how to open the modal.
    pub fn set_on_new_vm<F: Fn() + 'static>(&self, f: F) {
        *self.on_new_vm.borrow_mut() = Some(Box::new(f));
    }

    /// Borrow the underlying GTK widget for layout parenting.
    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
}
