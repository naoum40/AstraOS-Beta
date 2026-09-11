// content.rs - Right-side content area (page switcher).
//
// A `GtkBox` vertical wrapper holding a `GtkStack`. The stack has one
// named child per settings page:
//
//   - "account"         -> AccountPage
//   - "personalization" -> PersonalizationPage
//   - "system"          -> SystemPage
//   - "about"           -> AboutPage
//
// `switch_to(idx)` flips the visible stack child. Sidebar clicks route
// here via the page-changed callback wired in `main.rs`.

use crate::config::SettingsConfig;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Stack};
use std::cell::RefCell;
use std::rc::Rc;

/// Map sidebar indices to stack child names (kept in sync with the
/// `NAV_ENTRIES` order in `sidebar.rs`).
const PAGE_NAMES: [&str; 4] = ["account", "personalization", "system", "about"];

/// Content area - owns the page stack and a shared `SettingsConfig`
/// cell so any page can mutate + persist the user config.
pub struct Content {
    container: GtkBox,
    stack: Stack,
    cfg: Rc<RefCell<SettingsConfig>>,
}

impl Content {
    /// Build the content area + all four sub-pages. `cfg` is shared
    /// with the personalization page so toggles / picks can write back
    /// to `~/.config/astra/desktop.toml`.
    pub fn new(cfg: &SettingsConfig) -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .css_classes(["astra-content"])
            .build();

        let stack = Stack::builder()
            .hexpand(true)
            .vexpand(true)
            .transition_type(gtk4::StackTransitionType::Crossfade)
            .build();

        let cfg_cell = Rc::new(RefCell::new(cfg.clone()));

        let account_page = super::account::AccountPage::new();
        stack.add_named(account_page.widget(), Some(PAGE_NAMES[0]));

        let personalization_page = super::personalization::PersonalizationPage::new(&cfg_cell);
        stack.add_named(personalization_page.widget(), Some(PAGE_NAMES[1]));

        let system_page = super::system::SystemPage::new(&cfg_cell);
        stack.add_named(system_page.widget(), Some(PAGE_NAMES[2]));

        let about_page = super::about::AboutPage::new();
        stack.add_named(about_page.widget(), Some(PAGE_NAMES[3]));

        container.append(&stack);

        Self {
            container,
            stack,
            cfg: cfg_cell,
        }
    }

    /// Borrow the root `GtkBox` (for `leaflet.append` etc.).
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Switch the visible page by sidebar index. Out-of-range indices
    /// fall back to "account" (always safe).
    pub fn switch_to(&self, idx: usize) {
        let name = PAGE_NAMES.get(idx).copied().unwrap_or(PAGE_NAMES[0]);
        self.stack.set_visible_child_name(name);
    }

    /// Borrow the shared config cell. Exposed for future pages that
    /// need to read/write the user config (e.g. an account page that
    /// persists the user avatar path).
    #[allow(dead_code)]
    pub fn config(&self) -> &Rc<RefCell<SettingsConfig>> {
        &self.cfg
    }
}

impl Clone for Content {
    fn clone(&self) -> Self {
        Self {
            container: self.container.clone(),
            stack: self.stack.clone(),
            cfg: self.cfg.clone(),
        }
    }
}
