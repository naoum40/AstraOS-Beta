// bar.rs - Astra Shell taskbar (Windows mode).
//
// Anchored at the bottom of the screen, the bar surfaces:
//   * The AstraOS start button (✦) - toggles the launcher.
//   * Workspace indicators (1-9) - click to switch Hyprland workspaces.
//   * A right-aligned clock that ticks every second.
//
// In Windows mode the bar is always visible. In Mac mode the bar is
// not built at all - the `Dock` takes its place (see `dock.rs`).
//
// NOTE on layer-shell: AstraOS v0.2 ships without `gtk4-layer-shell`
// bindings, so the bar is a regular `ApplicationWindow`
// (`decorated = false`). A follow-up task (2-c) will swap in
// `gtk4-layer-shell-rs` to anchor it as a proper layer surface
// (above normal windows, no focus, no input region) so it can't be
// moved / closed by the user.

use crate::config::ShellConfig;
use crate::hyprland_ipc::HyprlandClient;
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox};
use gtk4::{Application, ApplicationWindow, Button, Label, Orientation};

/// The taskbar shell surface.
pub struct Bar {
    window: ApplicationWindow,
}

impl Bar {
    /// Build the bar. `app` owns the resulting window (GTK ref-counting),
    /// so the returned `Bar` wrapper can be dropped immediately after
    /// `present()` - the underlying window stays alive until the
    /// `Application` is released.
    pub fn new(app: &Application, _config: &ShellConfig, _hyprland: &HyprlandClient) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Shell Bar")
            .default_width(800)
            .default_height(48)
            .decorated(false)
            .build();

        // The `astra-bar` CSS class is the primary selector for the
        // glassmorphism panel rules in `glassmorphism.css`.
        window.add_css_class("astra-bar");

        // Main horizontal container - start button | separator |
        // workspaces 1..=9 | spacer (hexpand) | clock.
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        window.set_child(Some(&container));

        // --- Start button (✦) -------------------------------------------
        // Clicking it toggles the launcher. The launcher is a singleton
        // kept by the `Application`; we send a `toggle` action via the
        // GApplication action map (set up in task 2-b). For ISO 2 we
        // log the click - the launcher wires up its own shortcut.
        let start_button = Button::builder()
            .label("\u{2726}") // ✦ four-pointed star
            .tooltip_text("AstraOS Menu")
            .css_classes(["astra-button", "astra-start"])
            .build();
        start_button.connect_clicked(|_| {
            log::info!("Start button clicked - launcher toggle (TODO: action map)");
        });
        container.append(&start_button);

        // --- Vertical separator -----------------------------------------
        let sep = Label::new(Some("|"));
        sep.set_opacity(0.3);
        container.append(&sep);

        // --- Workspace indicators (1..=9) --------------------------------
        // Each button logs the workspace switch; task 2-b will replace
        // the log with a `hyprctl dispatch workspace N` call.
        let workspace_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();
        for i in 1..=9 {
            let ws_btn = Button::builder()
                .label(&i.to_string())
                .tooltip_text(&format!("Switch to workspace {}", i))
                .css_classes(["astra-button", "astra-workspace"])
                .build();
            let ws_id = i;
            ws_btn.connect_clicked(move |_| {
                log::info!("Switch to workspace {}", ws_id);
                // TODO task 2-b: dispatch Hyprland workspace switch via
                // the command socket (`dispatch workspace N`).
            });
            workspace_box.append(&ws_btn);
        }
        container.append(&workspace_box);

        // --- Expanding spacer (pushes the clock to the right) -----------
        let spacer = GtkBox::builder().hexpand(true).build();
        container.append(&spacer);

        // --- Clock (right-aligned, updates every second) ---------------
        // `glib::source::timeout_add_local` runs the closure on the GTK
        // main loop - no thread-safety concerns, we can clone + mutate
        // the Label directly.
        let clock = Label::builder()
            .label("--:--")
            .css_classes(["astra-clock"])
            .build();
        container.append(&clock);

        let clock_clone = clock.clone();
        glib::source::timeout_add_local(std::time::Duration::from_secs(1), move || {
            // `chrono::Local::now()` returns the wall-clock local time.
            // We format as HH:MM (24h) - AstraOS default locale is fr_FR.
            let now = chrono::Local::now();
            clock_clone.set_label(&now.format("%H:%M").to_string());
            glib::ControlFlow::Continue
        });

        Self { window }
    }

    /// Show the bar window.
    pub fn present(&self) {
        self.window.present();
    }
}
