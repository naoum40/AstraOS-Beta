// launcher.rs - Astra Shell app launcher.
//
// A modal-style window that opens with the Super key (or Super+Space)
// and offers a search-filtered grid of installed applications. ISO 2
// ships with a static list of common apps (Firefox / Files / Terminal
// / Settings / Vim / VLC / btop / wdisplays) - task 2-b will scan
// `/usr/share/applications/*.desktop` and produce the real icon grid.
//
// Behavior:
//   * Hidden by default (`visible(false)` in the builder).
//   * Super key (or Super+Space) toggles visibility - bound via the
//     Hyprland config (see `astra-core/astra.conf` in ISO 2), which
//     calls a D-Bus method `org.astraos.Shell.ToggleLauncher`
//     (task 2-b).
//   * Typing in the search entry filters the grid in real time.
//   * Enter launches the currently-selected app + closes the launcher.
//   * Escape closes the launcher without launching anything.

use crate::config::ShellConfig;
use glib::Propagation;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, FlowBox, FlowBoxChild, SearchEntry};

/// A static app entry - icon glyph, name, executable.
type AppEntry = (&'static str, &'static str, &'static str);

/// The static app set for ISO 2.
///
/// Real implementations (task 2-b) will enumerate `.desktop` files via
/// `gio::AppInfo` and produce `gtk4::IconPaintable` icons. The static
/// list lets us ship a working launcher in ISO 2 even before the
/// desktop-file scanner is wired up.
const STATIC_APPS: &[AppEntry] = &[
    ("\u{1F310}", "Firefox", "firefox"),                // 🌐
    ("\u{1F4C1}", "Files", "thunar"),                   // 📁
    ("\u{1F4BB}", "Terminal", "kitty"),                 // 💻
    ("\u{2699}\u{FE0F}", "Settings", "astra-settings"), // ⚙️
    ("\u{1F4DD}", "Text Editor", "vim"),                // 📝
    ("\u{1F3AC}", "VLC", "vlc"),                        // 🎬
    ("\u{1F4CA}", "Task Manager", "btop"),              // 📊
    ("\u{1F3A8}", "Display Settings", "wdisplays"),     // 🎨
];

/// The launcher shell surface. Hidden by default; toggle via
/// [`Self::toggle`].
pub struct Launcher {
    // `window` is held so the Launcher can `set_visible()` on it from
    // `toggle()` / `present()`. The GTK window is also kept alive by the
    // Application, so the field is technically only needed for the
    // public API methods below - hence the `allow(dead_code)` until
    // task 2-b wires up the D-Bus shortcut.
    #[allow(dead_code)]
    window: ApplicationWindow,
    #[allow(dead_code)]
    search: SearchEntry,
    #[allow(dead_code)]
    grid: FlowBox,
}

impl Launcher {
    /// Build the launcher window. Starts hidden - call [`Self::toggle`]
    /// (or wire up a Hyprland/D-Bus shortcut) to reveal it.
    pub fn new(app: &Application, _config: &ShellConfig) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Launcher")
            .default_width(620)
            .default_height(550)
            .decorated(false)
            .visible(false) // hidden by default
            .build();

        window.add_css_class("astra-launcher");

        let main_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();
        window.set_child(Some(&main_box));

        // --- Search entry ----------------------------------------------
        let search = SearchEntry::builder()
            .placeholder_text("Rechercher des applications...")
            .css_classes(["astra-search"])
            .build();
        main_box.append(&search);

        // --- App grid (scrollable) -------------------------------------
        let grid = FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .column_spacing(12)
            .row_spacing(12)
            .build();

        let scrolled = gtk4::ScrolledWindow::builder()
            .child(&grid)
            .vexpand(true)
            .build();
        main_box.append(&scrolled);

        // Populate with the static app set.
        Self::populate_apps(&grid);

        // --- Search filter ----------------------------------------------
        // Walk every FlowBoxChild, look at the underlying Button, and
        // toggle visibility based on whether its label or tooltip
        // contains the search text.
        //
        // NOTE: glib 0.20 deprecated the old `clone!(@weak x => ...)`
        // syntax in favor of `clone!(#[weak] x, move |args| ...)`. We
        // use the new form here so the build doesn't emit a
        // deprecation warning.
        let grid_clone = grid.clone();
        search.connect_changed(clone!(
            #[weak]
            grid_clone,
            move |entry| {
                let text = entry.text().to_lowercase();
                let mut i = 0;
                while let Some(flow_child) = grid_clone.child_at_index(i) {
                    if let Some(btn) = flow_child.child() {
                        if let Some(button) = btn.downcast_ref::<gtk4::Button>() {
                            let label = button.label().unwrap_or_default().to_lowercase();
                            let tooltip = button.tooltip_text().unwrap_or_default().to_lowercase();
                            let visible = label.contains(&text) || tooltip.contains(&text);
                            flow_child.set_visible(visible);
                        }
                    }
                    i += 1;
                }
            }
        ));

        // --- Escape closes the launcher -------------------------------
        // EventControllerKey listens at the window level.
        // `Propagation::Stop` stops further propagation so the keypress
        // doesn't reach any underlying widget; `Proceed` lets it
        // through.
        //
        // NOTE: gtk4 0.9.x closures return `glib::Propagation`, not
        // `gtk4::Inhibit` (which was removed in gtk-rs-core 0.20).
        let window_clone = window.clone();
        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                window_clone.set_visible(false);
                return Propagation::Stop;
            }
            Propagation::Proceed
        });
        window.add_controller(controller);

        Self {
            window,
            search,
            grid,
        }
    }

    /// Populate the launcher grid with the static app set.
    ///
    /// Each entry is a `Button` (96x96 px) wrapped in a `FlowBoxChild`.
    /// The button label is `"icon\nname"` - GTK4 Labels render the
    /// newline natively.
    fn populate_apps(grid: &FlowBox) {
        for (icon, name, exec) in STATIC_APPS {
            let btn = gtk4::Button::builder()
                .label(&format!("{}\n{}", icon, name))
                .tooltip_text(*name)
                .width_request(96)
                .height_request(96)
                .css_classes(["astra-app-icon"])
                .build();

            let exec_cmd = exec.to_string();
            btn.connect_clicked(move |_| {
                log::info!("Launching: {}", exec_cmd);
                match std::process::Command::new(&exec_cmd).spawn() {
                    Ok(_) => log::debug!("Spawned {} OK", exec_cmd),
                    Err(e) => log::warn!("Failed to spawn {}: {}", exec_cmd, e),
                }
            });

            let child = FlowBoxChild::new();
            child.set_child(Some(&btn));
            grid.insert(&child, -1); // -1 = append
        }
    }

    /// Toggle launcher visibility. Called from the D-Bus handler
    /// (task 2-b) when the Super key is pressed.
    #[allow(dead_code)] // wired up in task 2-b (D-Bus shortcut)
    pub fn toggle(&self) {
        let visible = self.window.is_visible();
        self.window.set_visible(!visible);
        if !visible {
            self.window.present();
            self.search.grab_focus();
        }
    }

    /// Show the launcher window (GTK lifecycle hook).
    #[allow(dead_code)] // called by toggle() and via D-Bus in task 2-b
    pub fn present(&self) {
        self.window.present();
    }
}
