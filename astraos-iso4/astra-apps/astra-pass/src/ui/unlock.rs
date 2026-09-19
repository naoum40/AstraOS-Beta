// unlock.rs - Unlock screen shown when a vault already exists on disk.
//
// A centered card containing:
//   - The AstraPass brand mark
//   - A password entry (visibility=false, masked)
//   - An "Unlock" button (default, activated on Enter)
//   - An error label (initially hidden) shown on wrong-password attempts
//
// State:
//   - `on_unlock`: callback fired with the typed password; the parent
//     attempts the unlock and calls `show_error` if it failed.
//
// The unlock screen does NOT keep the password in any long-lived state
// — it's read straight off the entry and handed to the callback. The
// `Rc<RefCell<...>>` here is only for cross-closure access to the
// error label + entry widgets.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Entry, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// The unlock screen widget.
#[derive(Clone)]
pub struct UnlockScreen {
    widget: GtkBox,
    password: Entry,
    error_label: Label,
    on_unlock: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl UnlockScreen {
    /// Build the unlock screen. No callback installed yet — wire via
    /// `set_on_unlock`.
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(14)
            .halign(Align::Center)
            .valign(Align::Center)
            .css_classes(["astra-unlock"])
            .build();

        // Brand mark.
        let brand = Label::builder()
            .label("<b>\u{2726} AstraPass</b>") // ✦ AstraPass
            .use_markup(true)
            .css_classes(["astra-brand-large"])
            .build();
        widget.append(&brand);

        let subtitle = Label::builder()
            .label("Enter your master password to unlock the vault")
            .css_classes(["astra-subtitle"])
            .build();
        widget.append(&subtitle);

        // Password entry.
        let password = Entry::builder()
            .placeholder_text("Master password")
            .visibility(false)
            .width_chars(28)
            .css_classes(["astra-field", "astra-password"])
            .build();
        widget.append(&password);

        // Error label — initially hidden.
        let error_label = Label::builder()
            .label("")
            .css_classes(["astra-error"])
            .visible(false)
            .build();
        widget.append(&error_label);

        // Unlock button.
        let unlock_btn = Button::builder()
            .label("Unlock vault")
            .css_classes(["astra-btn", "astra-btn-primary"])
            .build();
        widget.append(&unlock_btn);

        let on_unlock: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let screen = Self {
            widget: widget.clone(),
            password: password.clone(),
            error_label: error_label.clone(),
            on_unlock: on_unlock.clone(),
        };

        // Wire the Unlock button: grab the password text + fire the
        // callback. Don't clear the field — the parent decides whether
        // to do so on success vs leave it on failure.
        let screen_for_btn = screen.clone();
        unlock_btn.connect_clicked(move |_| {
            let text = screen_for_btn.password.text().to_string();
            if let Some(cb) = screen_for_btn.on_unlock.borrow().as_ref() {
                cb(&text);
            }
        });

        // Wire Enter on the password field → click the Unlock button.
        // `connect_activate` is the gtk4-rs 0.9 idiom for the
        // `::activate` signal on `gtk4::Entry`.
        let unlock_btn_clone = unlock_btn.clone();
        password.connect_activate(move |_| {
            unlock_btn_clone.emit_clicked();
        });

        screen
    }

    /// Install the unlock callback. The closure receives the typed
    /// password (clear-text).
    pub fn set_on_unlock<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_unlock.borrow_mut() = Some(Box::new(f));
    }

    /// Show an error message (e.g. "wrong master password"). Makes the
    /// error label visible + clears the password field so the user
    /// can retry.
    pub fn show_error(&self, message: &str) {
        self.error_label.set_label(message);
        self.error_label.set_visible(true);
        self.password.set_text("");
    }

    /// Grab keyboard focus on the password entry (called by main.rs
    /// right after presenting the unlock screen).
    pub fn grab_focus(&self) {
        self.password.grab_focus();
    }

    /// Borrow the underlying GTK widget for layout parenting.
    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
}
