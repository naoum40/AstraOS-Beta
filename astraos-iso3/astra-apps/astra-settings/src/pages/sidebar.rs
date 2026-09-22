// sidebar.rs - AstraOS Settings sidebar (left navigation).
//
// Vertical `GtkBox` (240px wide) with 4 navigation buttons:
//   0. Account          (avatar-default-symbolic)
//   1. Personalization   (preferences-desktop-appearance-symbolic)
//   2. System            (preferences-system-symbolic)
//   3. About             (help-about-symbolic)
//
// Each button is a `Button` with an `Image` + `Label` row child. The
// active button gets the CSS class `active` (styled with a magenta
// gradient by `settings.css`); the others stay plain.
//
// Clicking a button:
//   1. Removes `active` from every other button.
//   2. Adds `active` to the clicked button.
//   3. Invokes the registered `on_page_changed` callback with the new
//      index - the callback is responsible for switching the content
//      stack.
//
// The callback registry is a `Rc<RefCell<Option<Box<dyn Fn(usize)>>>>`
// so `Sidebar` can be cloned cheaply (it shares the registry + widget
// refs via the underlying GObject refcount).

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Navigation entries: (display label, GTK icon-name).
const NAV_ENTRIES: [(&str, &str); 4] = [
    ("Account", "avatar-default-symbolic"),
    ("Personalization", "preferences-desktop-appearance-symbolic"),
    ("System", "preferences-system-symbolic"),
    ("About", "help-about-symbolic"),
];

/// Settings sidebar - left-side navigation panel.
pub struct Sidebar {
    container: GtkBox,
    buttons: Vec<Button>,
    on_page_changed: Rc<RefCell<Option<Box<dyn Fn(usize)>>>>,
}

impl Sidebar {
    /// Build a new sidebar with the 4 default navigation entries.
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_top(18)
            .margin_bottom(18)
            .margin_start(12)
            .margin_end(12)
            .width_request(240)
            .css_classes(["astra-sidebar"])
            .build();

        // Header (logo glyph + "Settings" subtitle).
        let title = Label::builder()
            .label("\u{2726} AstraOS") // ✦ AstraOS
            .css_classes(["astra-sidebar-title"])
            .halign(gtk4::Align::Start)
            .margin_bottom(4)
            .build();
        container.append(&title);

        let subtitle = Label::builder()
            .label("Settings")
            .css_classes(["astra-sidebar-subtitle"])
            .halign(gtk4::Align::Start)
            .margin_bottom(18)
            .build();
        container.append(&subtitle);

        // Build each navigation button with an icon + label row child.
        let mut buttons: Vec<Button> = Vec::with_capacity(NAV_ENTRIES.len());
        for (name, icon_name) in NAV_ENTRIES.iter() {
            let btn = Button::builder()
                .css_classes(["astra-nav-button"])
                .hexpand(true)
                .halign(gtk4::Align::Fill)
                .build();

            let row = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(12)
                .margin_start(12)
                .margin_end(12)
                .build();

            let icon = Image::from_icon_name(icon_name);
            let label = Label::builder()
                .label(*name)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();

            row.append(&icon);
            row.append(&label);
            btn.set_child(Some(&row));

            container.append(&btn);
            buttons.push(btn);
        }

        let sidebar = Self {
            container: container.clone(),
            buttons: buttons.clone(),
            on_page_changed: Rc::new(RefCell::new(None)),
        };

        // Wire up click handlers. Each closure owns a clone of the
        // button Vec and the callback Rc, so it can re-highlight the
        // right button and fire the page-changed callback.
        for (i, btn) in buttons.iter().enumerate() {
            let cb = sidebar.on_page_changed.clone();
            let buttons_for_closure = sidebar.buttons.clone();
            btn.connect_clicked(move |_| {
                for (j, b) in buttons_for_closure.iter().enumerate() {
                    if j == i {
                        b.add_css_class("active");
                    } else {
                        b.remove_css_class("active");
                    }
                }
                if let Some(f) = cb.borrow().as_ref() {
                    f(i);
                }
            });
        }

        sidebar
    }

    /// Borrow the root `GtkBox` (for `leaflet.append` etc.).
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Register a callback fired whenever the active page changes.
    /// Replaces any previously registered callback.
    pub fn connect_page_changed<F: Fn(usize) + 'static>(&self, f: F) {
        *self.on_page_changed.borrow_mut() = Some(Box::new(f));
    }

    /// Programmatically set the active page (also fires the callback).
    pub fn set_active(&self, idx: usize) {
        for (i, b) in self.buttons.iter().enumerate() {
            if i == idx {
                b.add_css_class("active");
            } else {
                b.remove_css_class("active");
            }
        }
        if let Some(f) = self.on_page_changed.borrow().as_ref() {
            f(idx);
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Sidebar {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            buttons: self.buttons.clone(),
            on_page_changed: self.on_page_changed.clone(),
        }
    }
}
