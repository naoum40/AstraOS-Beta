// sidebar.rs - AstraPass left navigation rail.
//
// A vertical `gtk4::Box` (220px wide) hosting:
//
//   1. The AstraPass brand header ("✦ AstraPass").
//   2. A `gtk4::SearchEntry` that filters the entry list (the search
//      callback is installed by `main.rs`).
//   3. Three navigation buttons — All / Favorites / Trash — that
//      switch the entry list's filter mode.
//
// Selection routing uses the same `Rc<RefCell<Option<Box<dyn Fn>>>`
// pattern as `astra-settings/src/pages/sidebar.rs` — keeps the sidebar
// decoupled from the list widget itself.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, SearchEntry};
use std::cell::RefCell;
use std::rc::Rc;

/// One nav entry — internal table of (mode key, emoji, label).
struct NavEntry {
    key: &'static str,
    icon: &'static str,
    label: &'static str,
}

const ENTRIES: &[NavEntry] = &[
    NavEntry {
        key: "all",
        icon: "\u{1F5C2}\u{FE0F}", // 🗂️
        label: "All",
    },
    NavEntry {
        key: "favorites",
        icon: "\u{2B50}", // ⭐
        label: "Favorites",
    },
    NavEntry {
        key: "trash",
        icon: "\u{1F5D1}\u{FE0F}", // 🗑️
        label: "Trash",
    },
];

/// The sidebar widget.
///
/// Holds:
///   - the GTK box (returned to the parent for layout),
///   - the list of nav buttons (so we can toggle the `active` CSS
///     class when the selection changes),
///   - two optional callbacks: `on_nav` (mode change) and `on_search`
///     (search text change).
#[derive(Clone)]
pub struct Sidebar {
    widget: GtkBox,
    buttons: Rc<RefCell<Vec<Button>>>,
    on_nav: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
    on_search: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl Sidebar {
    /// Build the sidebar. No callbacks are installed yet — the parent
    /// window wires them via `set_on_nav` and `set_on_search`.
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .css_classes(["astra-sidebar"])
            .build();

        // Brand header.
        let header = Label::builder()
            .label("<b>\u{2726} AstraPass</b>") // ✦ AstraPass
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .margin_top(10)
            .margin_bottom(14)
            .margin_start(14)
            .css_classes(["astra-brand"])
            .build();
        widget.append(&header);

        // Search bar.
        let search = SearchEntry::builder()
            .placeholder_text("Search entries…")
            .margin_start(10)
            .margin_end(10)
            .margin_bottom(10)
            .css_classes(["astra-search"])
            .build();
        widget.append(&search);

        // Nav buttons.
        let buttons: Rc<RefCell<Vec<Button>>> = Rc::new(RefCell::new(Vec::new()));
        let on_nav: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let on_search: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let sidebar = Self {
            widget: widget.clone(),
            buttons: buttons.clone(),
            on_nav: on_nav.clone(),
            on_search: on_search.clone(),
        };

        for entry in ENTRIES {
            let btn = Button::builder()
                .label(&format!("{}  {}", entry.icon, entry.label))
                .css_classes(["astra-nav"])
                .hexpand(true)
                .halign(gtk4::Align::Fill)
                .build();

            let key = entry.key;
            let sidebar_clone = sidebar.clone();
            btn.connect_clicked(move |_| {
                sidebar_clone.select(key);
            });

            widget.append(&btn);
            buttons.borrow_mut().push(btn);
        }

        // Wire the search entry — fires the on_search callback on
        // every text change. `connect_search_changed` is the gtk4-rs
        // 0.9 idiom (SearchEntry is-a Editable + emits
        // `search-changed` rather than `changed`).
        let sidebar_for_search = sidebar.clone();
        search.connect_search_changed(move |e| {
            let text = e.text().to_string();
            if let Some(cb) = sidebar_for_search.on_search.borrow().as_ref() {
                cb(&text);
            }
        });

        sidebar
    }

    /// Install the nav-mode-change callback. The closure receives one
    /// of the three keys: `"all"`, `"favorites"`, `"trash"`.
    pub fn set_on_nav<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_nav.borrow_mut() = Some(Box::new(f));
    }

    /// Install the search-text-change callback. The closure receives
    /// the current text in the search entry (may be empty).
    pub fn set_on_search<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_search.borrow_mut() = Some(Box::new(f));
    }

    /// Select a nav mode by key. Updates the `active` CSS class on
    /// the buttons and fires the registered callback.
    pub fn select(&self, key: &str) {
        for (idx, btn) in self.buttons.borrow().iter().enumerate() {
            let is_active = ENTRIES[idx].key == key;
            if is_active {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        }

        if let Some(cb) = self.on_nav.borrow().as_ref() {
            cb(key);
        }
    }

    /// Select the first nav entry ("All"). Convenience for initial
    /// state — called once during window construction.
    pub fn select_first(&self) {
        if let Some(first) = ENTRIES.first() {
            self.select(first.key);
        }
    }

    /// Borrow the underlying GTK widget for layout parenting.
    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
}
