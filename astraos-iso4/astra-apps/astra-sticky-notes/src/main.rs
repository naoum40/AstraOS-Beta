// main.rs - AstraOS Sticky Notes.
//
// A floating-notes desktop app: each note is a borderless,
// draggable `ApplicationWindow` carrying:
//   * a colored background (6 pastel colors — yellow, red, blue,
//     green, purple, cyan),
//   * an editable title `Entry`,
//   * an editable `TextView` body (word-wrapped),
//   * a 6-color dot picker to recolor the note,
//   * a "+" button to spawn a new note,
//   * a "x" close button.
//
// Dragging is wired via a `GestureDrag` on the header box which
// calls `gdk::Toplevel::begin_move` on the underlying surface (the
// GTK4-native way to begin a window move without client-side
// position bookkeeping).
//
// State (id, title, content, color, x/y/width/height) is persisted to
// `~/.config/astra/sticky-notes.json` on every change and reloaded on
// startup, where every saved note gets its own window restored.

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Entry, GestureDrag,
    Orientation, ScrolledWindow, TextView, WrapMode,
};
use serde::{Deserialize, Serialize};

/// GApplication ID — registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.StickyNotes";

/// Relative path (from $HOME) of the JSON state file.
const CONFIG_REL: &str = ".config/astra/sticky-notes.json";

/// System path for the installed sticky-notes stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/sticky-notes.css";

/// Note color palette — (name, hex) pairs. Names are used as CSS class
/// suffixes (`sticky-color-<name>` on the window, `dot-<name>` on the
/// color picker buttons).
const COLORS: &[(&str, &str)] = &[
    ("yellow", "#fef08a"),
    ("red", "#fca5a5"),
    ("blue", "#a5b4fc"),
    ("green", "#86efac"),
    ("purple", "#d8b4fe"),
    ("cyan", "#67e8f9"),
];

/// Minimal embedded stylesheet, used when the system theme is missing.
/// Mirrors the most important rules from `assets/css/sticky-notes.css`.
const FALLBACK_CSS: &str = r#"
window.astra-sticky {
    border-radius: 10px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    font-family: 'Outfit', 'Inter', 'Cantarell', sans-serif;
}
.sticky-header {
    padding: 4px 6px;
}
.sticky-body {
    background: transparent;
}
.sticky-body text {
    background: transparent;
    color: #1f2937;
}
.sticky-title {
    background: transparent;
    border: none;
    box-shadow: none;
    font-weight: 700;
    color: #1f2937;
}
.sticky-title:focus {
    background: rgba(255, 255, 255, 0.4);
    border-radius: 4px;
}
button.sticky-color-dot {
    min-width: 14px;
    min-height: 14px;
    border-radius: 999px;
    padding: 0;
    margin: 2px;
    border: 1px solid rgba(0, 0, 0, 0.15);
}
button.sticky-color-dot.dot-yellow { background: #fef08a; }
button.sticky-color-dot.dot-red    { background: #fca5a5; }
button.sticky-color-dot.dot-blue   { background: #a5b4fc; }
button.sticky-color-dot.dot-green  { background: #86efac; }
button.sticky-color-dot.dot-purple { background: #d8b4fe; }
button.sticky-color-dot.dot-cyan   { background: #67e8f9; }
window.sticky-color-yellow { background: #fef08a; }
window.sticky-color-red    { background: #fca5a5; }
window.sticky-color-blue   { background: #a5b4fc; }
window.sticky-color-green  { background: #86efac; }
window.sticky-color-purple { background: #d8b4fe; }
window.sticky-color-cyan   { background: #67e8f9; }
button.sticky-close, button.sticky-add {
    min-width: 22px;
    min-height: 22px;
    border-radius: 6px;
    padding: 0;
    background: rgba(0, 0, 0, 0.10);
    color: #1f2937;
    font-weight: 700;
}
button.sticky-close:hover, button.sticky-add:hover {
    background: rgba(0, 0, 0, 0.22);
}
"#;

// =============================================================================
// Persistence model
// =============================================================================

/// One serialized sticky note.
#[derive(Serialize, Deserialize, Clone)]
struct NoteData {
    id: String,
    title: String,
    content: String,
    color: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl NoteData {
    /// Build a new note with a fresh id and a sensible default position
    /// offset by `seed` so multiple new notes don't stack on top of each
    /// other.
    fn new_default(seed: usize) -> Self {
        let offset = seed as i32 * 30;
        Self {
            id: format!(
                "note-{}",
                chrono::Utc::now().timestamp_millis() + seed as i64
            ),
            title: "New Note".to_string(),
            content: String::new(),
            color: "yellow".to_string(),
            x: 120 + offset,
            y: 120 + offset,
            w: 240,
            h: 240,
        }
    }
}

/// Top-level JSON file shape.
#[derive(Serialize, Deserialize, Default, Clone)]
struct NotesFile {
    notes: Vec<NoteData>,
}

/// Shared app state — holds the in-memory note list and the lock for
/// serializing mutations from UI callbacks.
struct AppCtx {
    notes: RefCell<NotesFile>,
}

impl AppCtx {
    /// Absolute path of the JSON state file under `$HOME`.
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
        PathBuf::from(home).join(CONFIG_REL)
    }

    /// Load notes from disk. Missing file or parse error → empty list
    /// (logs at warn level so the user knows why their notes vanished).
    fn load() -> NotesFile {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
                log::warn!("Failed to parse {}: {}", path.display(), e);
                NotesFile::default()
            }),
            Err(e) => {
                log::info!(
                    "No sticky-notes config at {} ({}), starting fresh",
                    path.display(),
                    e
                );
                NotesFile::default()
            }
        }
    }

    /// Persist the current notes to disk (pretty-printed JSON).
    /// Failure is logged but does not crash the app.
    fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                log::error!("Failed to create {}: {}", parent.display(), e);
                return;
            }
        }
        let snapshot = self.notes.borrow().clone();
        match serde_json::to_string_pretty(&snapshot) {
            Ok(s) => {
                if let Err(e) = fs::write(&path, s) {
                    log::error!("Failed to write {}: {}", path.display(), e);
                }
            }
            Err(e) => log::error!("Failed to serialize notes: {}", e),
        }
    }

    /// Insert or replace a note by id.
    fn upsert(&self, updated: &NoteData) {
        let mut notes = self.notes.borrow_mut();
        if let Some(slot) = notes.notes.iter_mut().find(|n| n.id == updated.id) {
            *slot = updated.clone();
        } else {
            notes.notes.push(updated.clone());
        }
    }

    /// Remove a note by id.
    fn remove(&self, id: &str) {
        let mut notes = self.notes.borrow_mut();
        notes.notes.retain(|n| n.id != id);
    }
}

// =============================================================================
// Theme loading
// =============================================================================

/// Load the sticky-notes stylesheet onto the default `gdk::Display`.
/// Falls back to an embedded stylesheet when the system theme file is
/// missing so the app is still usable in a dev sandbox.
fn load_theme() {
    let provider = CssProvider::new();
    let css = match std::fs::read_to_string(THEME_PATH) {
        Ok(c) => {
            log::info!("Loaded sticky-notes theme from {}", THEME_PATH);
            c
        }
        Err(_) => {
            log::warn!(
                "Theme not found at {} — using embedded fallback CSS",
                THEME_PATH
            );
            FALLBACK_CSS.to_string()
        }
    };
    provider.load_from_data(&css);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    } else {
        log::error!("Could not get default GdkDisplay — theme not applied");
    }
}

// =============================================================================
// Note window construction
// =============================================================================

/// Build and present a single note window from `data`.
///
/// The window is borderless (`decorated = false`) and uses a
/// `GestureDrag` on its header row to drive the window move via
/// `gdk::Toplevel::begin_move`. All mutations (title / body / color /
/// resize / close) are mirrored back into the shared `AppCtx` and
/// saved to disk.
fn build_note_window(app: &Application, ctx: &Rc<AppCtx>, data: NoteData) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title(&data.title)
        .default_width(data.w.max(180))
        .default_height(data.h.max(160))
        .decorated(false)
        .build();
    window.add_css_class("astra-sticky");
    window.add_css_class(&format!("sticky-color-{}", data.color));

    // --- Header row (title + color picker + add + close) ---------------
    let header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .css_classes(["sticky-header"])
        .build();

    let title_entry = Entry::builder()
        .text(&data.title)
        .hexpand(true)
        .css_classes(["sticky-title"])
        .build();

    let color_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .build();
    for (name, _hex) in COLORS {
        let dot = Button::builder()
            .css_classes(["sticky-color-dot", &format!("dot-{}", name)])
            .tooltip_text(*name)
            .build();
        let win_clone = window.clone();
        let ctx_clone = ctx.clone();
        let id = data.id.clone();
        let new_color = name.to_string();
        dot.connect_clicked(move |_| {
            for (other, _) in COLORS {
                win_clone.remove_css_class(&format!("sticky-color-{}", other));
            }
            win_clone.add_css_class(&format!("sticky-color-{}", new_color));
            let mut notes = ctx_clone.notes.borrow_mut();
            if let Some(slot) = notes.notes.iter_mut().find(|n| n.id == id) {
                slot.color = new_color.clone();
            }
            drop(notes);
            ctx_clone.save();
        });
        color_box.append(&dot);
    }

    let btn_add = Button::builder()
        .label("+")
        .tooltip_text("New note")
        .css_classes(["sticky-add"])
        .build();
    let btn_close = Button::builder()
        .label("×")
        .tooltip_text("Close")
        .css_classes(["sticky-close"])
        .build();

    header.append(&title_entry);
    header.append(&color_box);
    header.append(&btn_add);
    header.append(&btn_close);

    // --- Body (TextView in a ScrolledWindow) ---------------------------
    let text_view = TextView::builder()
        .css_classes(["sticky-body"])
        .hexpand(true)
        .vexpand(true)
        .wrap_mode(WrapMode::Word)
        .top_margin(4)
        .bottom_margin(4)
        .left_margin(6)
        .right_margin(6)
        .build();
    let buffer = text_view.buffer();
    buffer.set_text(&data.content);

    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    scrolled.set_child(Some(&text_view));

    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();
    root.append(&header);
    root.append(&scrolled);
    window.set_child(Some(&root));

    // --- Drag-to-move via GestureDrag → gdk::Toplevel::begin_move -------
    // `begin_move(device, button, x, y, timestamp)` initiates a compositor
    // window move. We pull the originating device + event timestamp from
    // the gesture itself so the request carries a valid input sequence.
    let drag = GestureDrag::new();
    header.add_controller(drag.clone());
    let win_for_drag = window.clone();
    drag.connect_drag_begin(move |gesture, _, _| {
        let Some(surface) = win_for_drag.surface() else {
            return;
        };
        let Ok(toplevel) = surface.downcast::<gtk4::gdk::Toplevel>() else {
            return;
        };
        let Some(device) = gesture.current_event_device() else {
            return;
        };
        let timestamp = gesture.current_event_time();
        toplevel.begin_move(&device, 1, 0.0, 0.0, timestamp);
    });

    // --- Title change → save -------------------------------------------
    {
        let ctx = ctx.clone();
        let id = data.id.clone();
        title_entry.connect_changed(move |e| {
            let text = e.text().to_string();
            let mut notes = ctx.notes.borrow_mut();
            if let Some(slot) = notes.notes.iter_mut().find(|n| n.id == id) {
                slot.title = text;
            }
            drop(notes);
            ctx.save();
        });
    }

    // --- Body change → save --------------------------------------------
    {
        let ctx = ctx.clone();
        let id = data.id.clone();
        let buf = buffer.clone();
        let buf_for_cb = buf.clone();
        buf.connect_changed(move |_| {
            let (start, end) = buf_for_cb.bounds();
            let text = buf_for_cb.text(&start, &end, false).to_string();
            let mut notes = ctx.notes.borrow_mut();
            if let Some(slot) = notes.notes.iter_mut().find(|n| n.id == id) {
                slot.content = text;
            }
            drop(notes);
            ctx.save();
        });
    }

    // --- Resize → save width/height ------------------------------------
    // Track both default-width and default-height notifies so resize
    // is captured on both axes. (Position is compositor-controlled on
    // Wayland so we don't try to restore x/y at runtime — only on
    // startup via the saved file is the size honored.)
    {
        let ctx = ctx.clone();
        let id = data.id.clone();
        let win_for_resize = window.clone();
        window.connect_notify_local(Some("default-width"), move |_, _| {
            let w = win_for_resize.default_width();
            let mut notes = ctx.notes.borrow_mut();
            if let Some(slot) = notes.notes.iter_mut().find(|n| n.id == id) {
                slot.w = w;
            }
            drop(notes);
            ctx.save();
        });
    }
    {
        let ctx = ctx.clone();
        let id = data.id.clone();
        let win_for_resize = window.clone();
        window.connect_notify_local(Some("default-height"), move |_, _| {
            let h = win_for_resize.default_height();
            let mut notes = ctx.notes.borrow_mut();
            if let Some(slot) = notes.notes.iter_mut().find(|n| n.id == id) {
                slot.h = h;
            }
            drop(notes);
            ctx.save();
        });
    }

    // --- "+" → spawn a new note ----------------------------------------
    {
        let app = app.clone();
        let ctx = ctx.clone();
        btn_add.connect_clicked(move |_| {
            let count = ctx.notes.borrow().notes.len();
            let new = NoteData::new_default(count + 1);
            ctx.upsert(&new);
            ctx.save();
            build_note_window(&app, &ctx, new);
        });
    }

    // --- "x" → close window, remove note from store --------------------
    {
        let ctx = ctx.clone();
        let id = data.id.clone();
        let win_for_close = window.clone();
        btn_close.connect_clicked(move |_| {
            ctx.remove(&id);
            ctx.save();
            win_for_close.close();
        });
    }

    window.present();
}

// =============================================================================
// Entry point
// =============================================================================

fn main() {
    env_logger::init();
    log::info!("Astra Sticky Notes v0.4.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        load_theme();

        let ctx = Rc::new(AppCtx {
            notes: RefCell::new(AppCtx::load()),
        });

        // If the store is empty, seed with one default note so the user
        // sees something to start with. Otherwise restore one window per
        // saved note.
        let initial: Vec<NoteData> = {
            let notes = ctx.notes.borrow();
            if notes.notes.is_empty() {
                vec![NoteData::new_default(0)]
            } else {
                notes.notes.clone()
            }
        };
        // If we just seeded, persist it so the file exists.
        if ctx.notes.borrow().notes.is_empty() {
            for note in &initial {
                ctx.upsert(note);
            }
            ctx.save();
        }
        for note in initial {
            build_note_window(app, &ctx, note);
        }
    });

    app.run();
}
