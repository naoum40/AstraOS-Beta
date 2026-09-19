// detail.rs - AstraPass right pane (entry editor form).
//
// A scrollable vertical form with:
//   - Title  (gtk4::Entry)
//   - Username (gtk4::Entry)
//   - Password (gtk4::Entry with visibility toggle)
//   - Notes (gtk4::TextView, multi-line)
//   - Favorite (gtk4::Switch)
//   - Save / Delete buttons
//
// State:
//   - `current_id`: the id of the entry currently displayed, or `None`
//     if the form is blank (no selection).
//   - `on_save`: callback fired with the (optional) entry id + the
//     field values from the form. The parent decides whether to
//     create a new entry or update an existing one.
//   - `on_delete`: callback fired with the entry id to delete.

use crate::vault::Entry as VaultEntry;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Entry, Label, Orientation, ScrolledWindow, Switch, TextView,
};
use std::cell::RefCell;
use std::rc::Rc;

/// The detail pane widget.
#[derive(Clone)]
pub struct Detail {
    container: ScrolledWindow,
    title: Entry,
    username: Entry,
    password: Entry,
    notes: TextView,
    favorite: Switch,
    save_btn: Button,
    delete_btn: Button,
    /// The id of the entry currently displayed, or `None` for a blank
    /// form. Set by `load_entry` and cleared by `clear`.
    current_id: Rc<RefCell<Option<String>>>,
    on_save: Rc<RefCell<Option<Box<dyn Fn(Option<&str>, EntryDraft)>>>>,
    on_delete: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

/// Snapshot of the form's field values, passed to the save callback.
/// This decouples the callback signature from the GTK widget handles
/// so `main.rs` doesn't need to know about the form internals.
#[derive(Clone, Debug)]
pub struct EntryDraft {
    pub title: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub favorite: bool,
}

impl Detail {
    /// Build the detail form. All fields start empty + read-only
    /// (`set_sensitive(false)`). Call `load_entry` or `start_new` to
    /// activate the form.
    pub fn new() -> Self {
        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(14)
            .margin_start(28)
            .margin_end(28)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let container = ScrolledWindow::builder()
            .child(&root)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .css_classes(["astra-detail-scroll"])
            .hexpand(true)
            .vexpand(true)
            .build();

        // Header label.
        let header = Label::builder()
            .label("Entry")
            .halign(Align::Start)
            .css_classes(["astra-detail-header"])
            .build();
        root.append(&header);

        // --- Title field -------------------------------------------------
        let title_label = field_label("Title");
        root.append(&title_label);
        let title = Entry::builder()
            .placeholder_text("e.g. GitHub")
            .css_classes(["astra-field"])
            .hexpand(true)
            .build();
        root.append(&title);

        // --- Username field ----------------------------------------------
        let username_label = field_label("Username / Email");
        root.append(&username_label);
        let username = Entry::builder()
            .placeholder_text("your.email@example.com")
            .css_classes(["astra-field"])
            .hexpand(true)
            .build();
        root.append(&username);

        // --- Password field (with show/hide toggle) ---------------------
        let password_label = field_label("Password");
        root.append(&password_label);

        let password_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        let password = Entry::builder()
            .placeholder_text("••••••••")
            .visibility(false)
            .css_classes(["astra-field", "astra-password"])
            .hexpand(true)
            .build();
        password_row.append(&password);

        // Track visibility locally to avoid the gtk4-rs ambiguity
        // between `WidgetExt::is_visible` (widget visibility) and
        // `Editable::is_visible` (text visibility) — both exist on
        // `gtk4::Entry` and the trait resolution picks the wrong one.
        let visible_state = Rc::new(RefCell::new(false));
        let show_btn = Button::builder()
            .label("\u{1F441}") // 👁
            .css_classes(["astra-toggle"])
            .build();
        {
            let password_clone = password.clone();
            let visible_state = visible_state.clone();
            show_btn.connect_clicked(move |_| {
                let mut cur = visible_state.borrow_mut();
                *cur = !*cur;
                password_clone.set_visibility(*cur);
            });
        }
        password_row.append(&show_btn);
        root.append(&password_row);

        // --- Notes -------------------------------------------------------
        let notes_label = field_label("Notes");
        root.append(&notes_label);

        let notes = TextView::builder()
            .css_classes(["astra-textview"])
            .wrap_mode(gtk4::WrapMode::WordChar)
            .height_request(120)
            .hexpand(true)
            .build();
        // Pack the TextView inside a ScrolledWindow so it scrolls when
        // the notes grow beyond the height request.
        let notes_scroll = ScrolledWindow::builder()
            .child(&notes)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .css_classes(["astra-notes-scroll"])
            .hexpand(true)
            .build();
        root.append(&notes_scroll);

        // --- Favorite switch --------------------------------------------
        let fav_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .build();
        let fav_label = Label::builder()
            .label("Favorite")
            .halign(Align::Start)
            .hexpand(true)
            .css_classes(["astra-fav-label"])
            .build();
        let favorite = Switch::builder().css_classes(["astra-switch"]).build();
        fav_row.append(&fav_label);
        fav_row.append(&favorite);
        root.append(&fav_row);

        // --- Action buttons ---------------------------------------------
        let actions = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .halign(Align::End)
            .hexpand(true)
            .margin_top(8)
            .build();
        let delete_btn = Button::builder()
            .label("Delete")
            .css_classes(["astra-btn", "astra-btn-danger"])
            .sensitive(false)
            .build();
        let save_btn = Button::builder()
            .label("Save")
            .css_classes(["astra-btn", "astra-btn-primary"])
            .sensitive(false)
            .build();
        actions.append(&delete_btn);
        actions.append(&save_btn);
        root.append(&actions);

        let current_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let on_save: Rc<RefCell<Option<Box<dyn Fn(Option<&str>, EntryDraft)>>>> =
            Rc::new(RefCell::new(None));
        let on_delete: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let detail = Self {
            container,
            title: title.clone(),
            username: username.clone(),
            password: password.clone(),
            notes: notes.clone(),
            favorite: favorite.clone(),
            save_btn: save_btn.clone(),
            delete_btn: delete_btn.clone(),
            current_id: current_id.clone(),
            on_save: on_save.clone(),
            on_delete: on_delete.clone(),
        };

        // Wire the Save button: snapshot the form values and fire the
        // callback. The TextBuffer's `start_iter`/`end_iter` are the
        // gtk4-rs 0.9 API for grabbing the full text range.
        let detail_for_save = detail.clone();
        save_btn.connect_clicked(move |_| {
            let draft = detail_for_save.collect_draft();
            let id_opt = detail_for_save.current_id.borrow().clone();
            let id_ref: Option<&str> = id_opt.as_deref();
            if let Some(cb) = detail_for_save.on_save.borrow().as_ref() {
                cb(id_ref, draft);
            }
        });

        // Wire the Delete button: fire the callback with the current
        // entry id. If `current_id` is None the button is already
        // insensitive — defensive guard anyway.
        let detail_for_delete = detail.clone();
        delete_btn.connect_clicked(move |_| {
            let id_opt = detail_for_delete.current_id.borrow().clone();
            if let Some(id) = id_opt {
                if let Some(cb) = detail_for_delete.on_delete.borrow().as_ref() {
                    cb(&id);
                }
            }
        });

        // Start blank + insensitive.
        detail.set_sensitive(false);
        detail
    }

    /// Install the save callback. The closure receives the entry id
    /// (or `None` for a new entry) and an `EntryDraft` snapshot of
    /// the form's current field values.
    pub fn set_on_save<F: Fn(Option<&str>, EntryDraft) + 'static>(&self, f: F) {
        *self.on_save.borrow_mut() = Some(Box::new(f));
    }

    /// Install the delete callback. The closure receives the entry id
    /// of the entry to delete (never `None` — the Delete button is
    /// insensitive when no entry is loaded).
    pub fn set_on_delete<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_delete.borrow_mut() = Some(Box::new(f));
    }

    /// Load an existing entry into the form. Activates the form,
    /// enables Save/Delete buttons.
    pub fn load_entry(&self, entry: &VaultEntry) {
        *self.current_id.borrow_mut() = Some(entry.id.clone());
        self.title.set_text(&entry.title);
        self.username.set_text(&entry.username);
        self.password.set_text(&entry.password);

        // TextView text goes through its TextBuffer.
        let buf = self.notes.buffer();
        buf.set_text(&entry.notes);

        self.favorite.set_active(entry.favorite);
        self.set_sensitive(true);
        self.save_btn.set_sensitive(true);
        self.delete_btn.set_sensitive(true);
    }

    /// Start a new blank entry. Activates the form + Save button, but
    /// the Delete button stays insensitive (nothing to delete yet).
    pub fn start_new(&self) {
        *self.current_id.borrow_mut() = None;
        self.title.set_text("");
        self.username.set_text("");
        self.password.set_text("");
        self.notes.buffer().set_text("");
        self.favorite.set_active(false);
        self.set_sensitive(true);
        self.save_btn.set_sensitive(true);
        self.delete_btn.set_sensitive(false);
    }

    /// Clear the form (no selection). Deactivates all fields + buttons.
    pub fn clear(&self) {
        *self.current_id.borrow_mut() = None;
        self.title.set_text("");
        self.username.set_text("");
        self.password.set_text("");
        self.notes.buffer().set_text("");
        self.favorite.set_active(false);
        self.set_sensitive(false);
        self.save_btn.set_sensitive(false);
        self.delete_btn.set_sensitive(false);
    }

    /// Snapshot the form's field values into an `EntryDraft`.
    fn collect_draft(&self) -> EntryDraft {
        let notes = self
            .notes
            .buffer()
            .start_iter()
            .text(&self.notes.buffer().end_iter())
            .to_string();
        EntryDraft {
            title: self.title.text().to_string(),
            username: self.username.text().to_string(),
            password: self.password.text().to_string(),
            notes,
            favorite: self.favorite.is_active(),
        }
    }

    /// Enable/disable all editable fields.
    fn set_sensitive(&self, sensitive: bool) {
        self.title.set_sensitive(sensitive);
        self.username.set_sensitive(sensitive);
        self.password.set_sensitive(sensitive);
        self.notes.set_sensitive(sensitive);
        self.favorite.set_sensitive(sensitive);
    }

    /// Borrow the underlying container for layout parenting.
    pub fn widget(&self) -> &ScrolledWindow {
        &self.container
    }
}

/// Build a small form-field label with the `astra-field-label` CSS class.
fn field_label(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(Align::Start)
        .css_classes(["astra-field-label"])
        .build()
}
