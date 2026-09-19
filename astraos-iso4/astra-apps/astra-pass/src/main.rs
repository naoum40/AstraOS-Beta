// main.rs - AstraPass entry point.
//
// Boots the AstraOS Password Manager — a 3-pane (sidebar / list /
// detail) GTK4 + libadwaita application inspired by Proton Pass.
//
// Pipeline:
//   1. `env_logger::init()` — stderr logging (RUST_LOG=info).
//   2. `gtk4::init()` + `libadwaita::init()` — must run on the GTK
//      main thread before any widget is built.
//   3. Build the `gtk4::Application` (GApplication ID
//      `org.astraos.AstraPass`).
//   4. On activate:
//        a. Load CSS theme (system path with embedded fallback).
//        b. Resolve the vault path:
//             `~/.config/astra/astrapass/vault.bin`
//        c. Build the main `gtk4::Stack` with three named children:
//             - "setup"  — first-time master password creation.
//             - "unlock" — existing vault unlock.
//             - "main"   — the 3-pane manager (sidebar / list / detail).
//        d. Branch on vault existence:
//             - missing → show "setup" child.
//             - present → show "unlock" child.
//        e. Wire the setup/unlock callbacks to attempt vault
//           creation/unlock and switch to the "main" view on success.
//
// Vault state lives in an `Rc<RefCell<Option<Vault>>>` shared between
// the main view's Save/Delete callbacks — they mutate the vault in
// place and re-serialize it to disk on every change.

mod crypto;
mod ui;
mod vault;

use gtk4::prelude::*;
use gtk4::{Application, Box as GtkBox, CssProvider, Orientation, Stack, StackTransitionType};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use ui::detail::Detail;
use ui::list::EntryList;
use ui::setup::SetupScreen;
use ui::sidebar::Sidebar;
use ui::unlock::UnlockScreen;
use vault::{Entry, Vault};

/// GApplication ID — registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.AstraPass";

/// System path for the installed AstraPass stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/astrapass.css";

/// Subdirectory under `~/.config` where AstraOS app configs live.
const ASTRA_CONFIG_SUBDIR: &str = "astra/astrapass";

/// Vault file name inside the config subdir.
const VAULT_FILENAME: &str = "vault.bin";

/// Embedded fallback stylesheet, used when the system theme file is
/// missing (dev runs without the installed theme).
const FALLBACK_CSS: &str = include_str!("../assets/css/astrapass.css");

/// Resolve the on-disk vault path: `~/.config/astra/astrapass/vault.bin`.
///
/// Falls back to the `HOME` env var; if neither `HOME` nor the user's
/// data dir is available we panic — AstraOS runs every app under a
/// logged-in user with a real home directory.
fn vault_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string());
    let mut p = PathBuf::from(home);
    p.push(".config");
    p.push(ASTRA_CONFIG_SUBDIR);
    p.push(VAULT_FILENAME);
    p
}

/// Current Unix timestamp in seconds — used as `created_at` for new
/// entries.
fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Load the AstraPass stylesheet onto the default `gdk::Display`.
fn load_theme() {
    let provider = CssProvider::new();

    match std::fs::read_to_string(THEME_PATH) {
        Ok(css) => {
            provider.load_from_data(&css);
            log::info!("Loaded theme from {}", THEME_PATH);
        }
        Err(_) => {
            provider.load_from_data(FALLBACK_CSS);
            log::warn!(
                "Theme file missing at {} — using embedded fallback",
                THEME_PATH
            );
        }
    }

    let display = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => {
            log::error!("Could not get default GdkDisplay — theme not applied");
            return;
        }
    };

    // gtk4-rs 0.9 free function (replaces the deprecated
    // `StyleContext::add_provider_for_display`).
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/// Build the 3-pane main view (sidebar / list / detail) inside a
/// `gtk4::Box`. Returns the box + the detail pane (so the caller can
/// wire the sidebar's selection callbacks).
fn build_main_view(vault: Vault) -> GtkBox {
    // Shared vault state. The sidebar/list/detail callbacks all
    // mutate this via `borrow_mut()` + re-serialize to disk.
    let vault_state: Rc<RefCell<Option<Vault>>> = Rc::new(RefCell::new(Some(vault)));
    let path = vault_path();

    // --- Sidebar (left, 220px) ----------------------------------------
    let sidebar = Sidebar::new();
    sidebar.widget().set_size_request(220, -1);

    // --- List pane (middle, 280px) -----------------------------------
    // The list pane has its own vertical layout: a "New entry" button
    // at the top, then the scrollable entry list filling the rest.
    let list_pane = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .css_classes(["astra-list-pane"])
        .build();
    list_pane.set_size_request(280, -1);

    let new_btn = gtk4::Button::builder()
        .label("+ New entry")
        .css_classes(["astra-btn", "astra-btn-new"])
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(6)
        .build();
    list_pane.append(&new_btn);

    let entry_list = EntryList::new();
    entry_list.widget().set_vexpand(true);
    entry_list.widget().set_hexpand(true);
    list_pane.append(entry_list.widget());

    // --- Detail pane (right, fills the rest) -------------------------
    let detail = Detail::new();
    detail.widget().set_hexpand(true);
    detail.widget().set_vexpand(true);

    // --- Wire sidebar → entry list -----------------------------------
    let entry_list_for_nav = entry_list.clone();
    sidebar.set_on_nav(move |mode| {
        entry_list_for_nav.set_mode(mode);
    });

    let entry_list_for_search = entry_list.clone();
    sidebar.set_on_search(move |query| {
        entry_list_for_search.set_query(query);
    });

    // --- Wire entry list → detail ------------------------------------
    let detail_for_select = detail.clone();
    let vault_for_select = vault_state.clone();
    entry_list.set_on_select(move |id_opt: Option<&str>| {
        let vault_guard = vault_for_select.borrow();
        let vault = match vault_guard.as_ref() {
            Some(v) => v,
            None => {
                detail_for_select.clear();
                return;
            }
        };
        match id_opt {
            Some(id) => {
                if let Some(entry) = vault.get_entries().iter().find(|e| e.id == id) {
                    detail_for_select.load_entry(entry);
                } else {
                    detail_for_select.clear();
                }
            }
            None => detail_for_select.clear(),
        }
    });

    // --- Wire "+ New entry" button → detail.start_new ---------------
    let detail_for_new = detail.clone();
    new_btn.connect_clicked(move |_| {
        detail_for_new.start_new();
    });

    // --- Wire detail Save --------------------------------------------
    let vault_for_save = vault_state.clone();
    let list_for_save = entry_list.clone();
    let detail_for_save = detail.clone();
    let path_for_save = path.clone();
    detail.set_on_save(move |id_opt: Option<&str>, draft: ui::detail::EntryDraft| {
        let mut vault_guard = vault_for_save.borrow_mut();
        let vault = match vault_guard.as_mut() {
            Some(v) => v,
            None => return,
        };

        // Upsert: if id_opt is Some + matches an existing entry, update
        // it in place; otherwise create a new entry.
        let now = now_unix();
        match id_opt {
            Some(id) => {
                if let Some(entry) = vault.find_entry_mut(id) {
                    entry.title = draft.title;
                    entry.username = draft.username;
                    entry.password = draft.password;
                    entry.notes = draft.notes;
                    entry.favorite = draft.favorite;
                }
            }
            None => {
                let mut entry = Entry::new(now);
                entry.title = draft.title;
                entry.username = draft.username;
                entry.password = draft.password;
                entry.notes = draft.notes;
                entry.favorite = draft.favorite;
                vault.add_entry(entry);
            }
        }

        // Persist + refresh the list. We clone the entries vector out
        // so the list rebuild doesn't hold the vault borrow.
        let entries: Vec<Entry> = vault.get_entries().to_vec();
        if let Err(e) = vault.save(&path_for_save) {
            log::error!("Failed to save vault: {}", e);
        }
        drop(vault_guard);

        list_for_save.set_entries(entries.clone());

        // If we just created a new entry, the detail form should
        // reset to blank for the next entry. For updates we keep the
        // entry loaded.
        if id_opt.is_none() {
            detail_for_save.clear();
        }
    });

    // --- Wire detail Delete ------------------------------------------
    let vault_for_delete = vault_state.clone();
    let list_for_delete = entry_list.clone();
    let detail_for_delete = detail.clone();
    detail.set_on_delete(move |id: &str| {
        let mut vault_guard = vault_for_delete.borrow_mut();
        let vault = match vault_guard.as_mut() {
            Some(v) => v,
            None => return,
        };
        let removed = vault.remove_entry(id);
        if !removed {
            log::warn!("Delete requested for non-existent entry {}", id);
            return;
        }

        let entries: Vec<Entry> = vault.get_entries().to_vec();
        if let Err(e) = vault.save(&path) {
            log::error!("Failed to save vault after delete: {}", e);
        }
        drop(vault_guard);

        list_for_delete.set_entries(entries.clone());
        list_for_delete.list_box().unselect_all();
        detail_for_delete.clear();
    });

    // Populate the list with the vault's initial entries.
    {
        let vault_guard = vault_state.borrow();
        if let Some(v) = vault_guard.as_ref() {
            let entries = v.get_entries().to_vec();
            entry_list.set_entries(entries);
        }
    }
    sidebar.select_first();

    // --- Assemble the 3-pane horizontal layout -----------------------
    let root = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .homogeneous(false)
        .css_classes(["astra-root"])
        .build();
    root.append(sidebar.widget());
    root.append(&list_pane);
    // Thin separator between list and detail panes.
    let sep1 = gtk4::Separator::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["astra-sep"])
        .build();
    root.append(&sep1);
    root.append(detail.widget());

    root
}

fn main() {
    env_logger::init();
    log::info!("AstraPass v0.4.0 starting…");

    gtk4::init().expect("Failed to initialize GTK");
    libadwaita::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        // 1. Load CSS theme.
        load_theme();

        // 2. Resolve the vault path + check if a vault exists.
        let path = vault_path();
        let existing_blob = match Vault::load(&path) {
            Ok(o) => o,
            Err(e) => {
                log::error!("Failed to read vault at {}: {}", path.display(), e);
                None
            }
        };

        // 3. Build the top-level Stack with three named children:
        //    "setup", "unlock", "main".
        let stack = Stack::builder()
            .transition_type(StackTransitionType::Crossfade)
            .transition_duration(200)
            .css_classes(["astra-stack"])
            .build();

        // Setup screen.
        let setup = SetupScreen::new();
        stack.add_named(setup.widget(), Some("setup"));

        // Unlock screen.
        let unlock = UnlockScreen::new();
        stack.add_named(unlock.widget(), Some("unlock"));

        // Default visible child depends on whether a vault exists.
        if existing_blob.is_some() {
            stack.set_visible_child_name("unlock");
            unlock.grab_focus();
        } else {
            stack.set_visible_child_name("setup");
            setup.grab_focus();
        }

        // --- Wire setup → create vault --------------------------------
        let path_for_setup = path.clone();
        let stack_for_setup = stack.clone();
        let setup_for_cb = setup.clone();
        setup.set_on_create(move |master_password: &str| {
            let vault = Vault::create(master_password);
            if let Err(e) = vault.save(&path_for_setup) {
                log::error!("Failed to save new vault: {}", e);
                setup_for_cb.show_error("Could not save vault. Check disk space + permissions.");
                return;
            }
            // Build the main view + add it to the stack under "main".
            let main_view = build_main_view(vault);
            // Remove any old "main" child before adding the new one
            // (defensive — there should never be one here).
            if let Some(child) = stack_for_setup.child_by_name("main") {
                stack_for_setup.remove(&child);
            }
            stack_for_setup.add_named(&main_view, Some("main"));
            stack_for_setup.set_visible_child_name("main");
        });

        // --- Wire unlock → unlock vault -------------------------------
        let path_for_unlock = path.clone();
        let stack_for_unlock = stack.clone();
        let unlock_for_cb = unlock.clone();
        let blob_for_unlock = existing_blob.clone();
        unlock.set_on_unlock(move |master_password: &str| {
            let blob = match &blob_for_unlock {
                Some(b) => b,
                None => {
                    unlock_for_cb.show_error("Vault file vanished. Please restart AstraPass.");
                    return;
                }
            };
            match Vault::unlock(master_password, blob) {
                Ok(vault) => {
                    // Save the vault on first successful unlock so any
                    // schema-version migrations get persisted.
                    let _ = path_for_unlock; // path is for future migrations
                    let main_view = build_main_view(vault);
                    if let Some(child) = stack_for_unlock.child_by_name("main") {
                        stack_for_unlock.remove(&child);
                    }
                    stack_for_unlock.add_named(&main_view, Some("main"));
                    stack_for_unlock.set_visible_child_name("main");
                }
                Err(e) => {
                    log::warn!("Vault unlock failed: {}", e);
                    unlock_for_cb.show_error("Wrong master password — please try again.");
                }
            }
        });

        // 4. Build the window.
        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("AstraPass")
            .default_width(920)
            .default_height(580)
            .content(&stack)
            .build();

        window.add_css_class("astra-pass");
        window.present();
        log::info!("AstraPass ready (vault path: {})", path.display());
    });

    app.run();
}
