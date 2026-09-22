// sidebar.rs - Astra Settings sidebar (left navigation rail).
//
// A vertical `gtk4::Box` containing 4 nav buttons (Account /
// Personalization / System / About). Each button is a labeled
// `gtk4::Button` with an emoji icon prefix. The active button carries
// the `active` CSS class (styled via `settings.css`).
//
// Selection routing uses a callback stored in an
// `Rc<RefCell<Option<Box<dyn Fn(&str)>>>>` so the parent window can
// install a closure that switches the content stack without the sidebar
// needing a back-reference to the content widget (keeps the modules
// decoupled, mirroring the `astra-shell` bar/dock pattern).

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Nav entry — internal table of (page-key, emoji, label).
struct NavEntry {
    key: &'static str,
    icon: &'static str,
    label: &'static str,
}

/// The 4 sidebar pages, in display order.
const ENTRIES: &[NavEntry] = &[
    NavEntry {
        key: "account",
        icon: "\u{1F464}", // 👤
        label: "Account",
    },
    NavEntry {
        key: "personalization",
        icon: "\u{1F3A8}", // 🎨
        label: "Personalization",
    },
    NavEntry {
        key: "system",
        icon: "\u{2699}\u{FE0F}", // ⚙️
        label: "System",
    },
    NavEntry {
        key: "about",
        icon: "\u{2139}\u{FE0F}", // ℹ️
        label: "About",
    },
];

/// The sidebar widget — owns the GTK box and a list of its nav buttons
/// (so it can toggle the `active` CSS class when the selection changes).
#[derive(Clone)]
pub struct Sidebar {
    widget: GtkBox,
    buttons: Rc<RefCell<Vec<Button>>>,
    on_select: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl Sidebar {
    /// Build the sidebar with all 4 nav buttons. No callback installed
    /// yet — call [`Sidebar::set_on_select`] to wire it to the content
    /// stack.
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .css_classes(["astra-sidebar"])
            .build();

        // Brand header — AstraOS wordmark at the top of the sidebar.
        let header = Label::builder()
            .label("<b>\u{2726} Astra Settings</b>") // ✦ Astra Settings
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .margin_top(8)
            .margin_bottom(16)
            .margin_start(14)
            .css_classes(["astra-brand"])
            .build();
        widget.append(&header);

        // Selection state held in an Rc<RefCell<...>> so the clicked
        // closures (which move-own a clone of `Sidebar`) can update
        // the active button + invoke the user's callback.
        let buttons: Rc<RefCell<Vec<Button>>> = Rc::new(RefCell::new(Vec::new()));
        let on_select: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let sidebar = Self {
            widget: widget.clone(),
            buttons: buttons.clone(),
            on_select: on_select.clone(),
        };

        // Build one button per entry. Each closure clones `sidebar` so
        // it can both update the active state and invoke the callback.
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

        sidebar
    }

    /// Install the selection callback. Called once by the parent
    /// window after both sidebar + content stack are built.
    ///
    /// The closure receives the page key ("account" / "personalization"
    /// / "system" / "about") and is expected to switch the content
    /// stack's visible child.
    pub fn set_on_select<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_select.borrow_mut() = Some(Box::new(f));
    }

    /// Select a page by key. Updates the `active` CSS class on the
    /// buttons and fires the registered callback.
    ///
    /// Safe to call before `set_on_select` — the callback is optional,
    /// a missing one is silently skipped (useful for the very first
    /// `select_first` call during construction).
    pub fn select(&self, key: &str) {
        // Toggle the active CSS class so only the matching button is
        // highlighted.
        for (idx, btn) in self.buttons.borrow().iter().enumerate() {
            let is_active = ENTRIES[idx].key == key;
            if is_active {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        }

        // Invoke the registered callback (if any). `glib::clone!` is
        // not needed here because `on_select` lives in the same Rc
        // cluster as `self` — the borrow is dropped before the
        // closure runs.
        if let Some(cb) = self.on_select.borrow().as_ref() {
            cb(key);
        }
    }

    /// Select the first page (Account). Convenience for initial state.
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
