// list.rs - AstraPass middle pane (entries list).
//
// A scrollable `gtk4::ListBox` rendering one row per vault entry. Each
// row shows the title (bold), the username (muted), and a star icon
// (★) if the entry is favorited.
//
// State held:
//   - `entries`: a full `Rc<RefCell<Vec<Entry>>>` snapshot of the
//     vault entries (refreshed by `set_entries`).
//   - `mode`: the active nav mode ("all" / "favorites" / "trash").
//   - `query`: the current search text.
//   - `on_select`: callback fired when the user clicks a row —
//     receives the selected entry id (or `None` if the selection is
//     cleared).
//
// The ListBox's `bind_model` is intentionally NOT used here — the
// entry count is small (typically < 500) and a manual rebuild on
// `refresh()` keeps the code simple + makes per-row styling (favorite
// star, etc.) straightforward.

use crate::vault::Entry;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SelectionMode};
use std::cell::RefCell;
use std::rc::Rc;

/// The list pane widget.
#[derive(Clone)]
pub struct EntryList {
    container: ScrolledWindow,
    list: ListBox,
    state: Rc<RefCell<ListState>>,
    on_select: Rc<RefCell<Option<Box<dyn Fn(Option<&str>)>>>>,
}

/// Internal list state — entries snapshot + current filter mode/query.
struct ListState {
    entries: Vec<Entry>,
    mode: String,
    query: String,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            mode: "all".to_string(),
            query: String::new(),
        }
    }
}

impl EntryList {
    /// Build the list pane (empty). Populate via `set_entries`.
    pub fn new() -> Self {
        let list = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .css_classes(["astra-list"])
            .show_separators(false)
            .build();

        let container = ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .css_classes(["astra-list-scroll"])
            .build();

        let state = Rc::new(RefCell::new(ListState::default()));
        let on_select: Rc<RefCell<Option<Box<dyn Fn(Option<&str>)>>>> = Rc::new(RefCell::new(None));

        let list_clone = list.clone();
        let state_clone = state.clone();
        let on_select_clone = on_select.clone();
        list.connect_row_selected(move |_, row: Option<&ListBoxRow>| {
            // Look up the entry id stored on the row via `name`
            // (a cheap, type-erased way to tag a row with its entry id
            // — `ListBoxRow` has no `set_data` API in gtk4-rs).
            let id = row.map(|r| r.widget_name().to_string());
            let id_ref: Option<&str> = id.as_deref();

            // Sanity check: ensure the id is still in the entries
            // vector before firing the callback (the list might have
            // been refreshed mid-click).
            if let Some(id_str) = id_ref {
                let exists = state_clone.borrow().entries.iter().any(|e| e.id == id_str);
                if !exists {
                    let _ = list_clone.unselect_all();
                    return;
                }
            }
            if let Some(cb) = on_select_clone.borrow().as_ref() {
                cb(id_ref);
            }
        });

        Self {
            container,
            list,
            state,
            on_select,
        }
    }

    /// Install the selection callback. Receives the entry id of the
    /// selected row, or `None` if the selection was cleared.
    pub fn set_on_select<F: Fn(Option<&str>) + 'static>(&self, f: F) {
        *self.on_select.borrow_mut() = Some(Box::new(f));
    }

    /// Replace the entries snapshot and rebuild the visible rows.
    pub fn set_entries(&self, entries: Vec<Entry>) {
        self.state.borrow_mut().entries = entries;
        self.refresh();
    }

    /// Set the active nav mode ("all" / "favorites" / "trash") and
    /// rebuild the visible rows.
    pub fn set_mode(&self, mode: &str) {
        self.state.borrow_mut().mode = mode.to_string();
        self.refresh();
    }

    /// Set the search query and rebuild the visible rows.
    pub fn set_query(&self, query: &str) {
        self.state.borrow_mut().query = query.to_lowercase();
        self.refresh();
    }

    /// Rebuild the list box rows from the current state, applying the
    /// filter (mode + query) before rendering.
    ///
    /// This is a full rebuild — fine for the typical < 500-entry vault.
    /// For very large vaults we'd switch to a `gio::ListStore` +
    /// `gtk4::SignalListItemFactory`.
    fn refresh(&self) {
        // Clear existing rows.
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }

        let state = self.state.borrow();
        let q = state.query.trim().to_lowercase();

        // Filter + sort: favorites first, then alphabetical by title.
        let mut filtered: Vec<&Entry> = state
            .entries
            .iter()
            .filter(|e| match state.mode.as_str() {
                "favorites" => e.favorite,
                "trash" => false, // trash not implemented for ISO 4
                _ => true,
            })
            .filter(|e| {
                if q.is_empty() {
                    return true;
                }
                e.title.to_lowercase().contains(&q) || e.username.to_lowercase().contains(&q)
            })
            .collect();

        filtered.sort_by(|a, b| {
            b.favorite
                .cmp(&a.favorite)
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });

        // Empty-state placeholder.
        if filtered.is_empty() {
            let empty = Label::builder()
                .label("<span alpha=\"55%\">No entries</span>")
                .use_markup(true)
                .css_classes(["astra-list-empty"])
                .margin_top(40)
                .build();
            self.list.append(&empty);
            return;
        }

        for entry in filtered {
            let row = build_row(entry);
            // Tag the row with the entry id via `widget_name` so the
            // selection callback can recover it.
            row.set_widget_name(&entry.id);
            self.list.append(&row);
        }
    }

    /// Borrow the underlying container for layout parenting.
    pub fn widget(&self) -> &ScrolledWindow {
        &self.container
    }

    /// Borrow the inner ListBox (used by main.rs to clear the
    /// selection after a delete).
    pub fn list_box(&self) -> &ListBox {
        &self.list
    }
}

/// Build a single ListBoxRow for an entry.
fn build_row(entry: &Entry) -> ListBoxRow {
    let row = ListBoxRow::builder()
        .css_classes(["astra-list-row"])
        .build();

    let hbox = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_start(12)
        .margin_end(12)
        .margin_top(10)
        .margin_bottom(10)
        .build();

    // Star icon (only shown if favorited).
    if entry.favorite {
        let star = Label::builder()
            .label("\u{2605}") // ★
            .css_classes(["astra-star"])
            .build();
        hbox.append(&star);
    }

    // Title + username stacked vertically.
    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .halign(gtk4::Align::Fill)
        .build();

    let title_text = if entry.title.is_empty() {
        "(untitled)".to_string()
    } else {
        entry.title.clone()
    };
    let title = Label::builder()
        .label(&title_text)
        .xalign(0.0)
        .halign(gtk4::Align::Start)
        .css_classes(["astra-list-title"])
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    vbox.append(&title);

    if !entry.username.is_empty() {
        let username = Label::builder()
            .label(&entry.username)
            .xalign(0.0)
            .halign(gtk4::Align::Start)
            .css_classes(["astra-list-username"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        vbox.append(&username);
    }
    hbox.append(&vbox);

    row.set_child(Some(&hbox));
    row
}
