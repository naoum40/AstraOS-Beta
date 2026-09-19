// clock.rs - AstraOS Clock widget.
//
// Floating glassmorphism panel that shows the current local time as
// `HH:MM` in a large Outfit font (48pt), with the long-form date
// (`Monday 15 January`) directly below.
//
// Pipeline:
//   1. Build an undecorated `ApplicationWindow` (CSS class `astra-widget`
//      + `clock`).
//   2. Pack a vertical `GtkBox` with two labels: `.clock-time` (48pt) on
//      top, `.clock-date` (10pt, accent color) below.
//   3. `glib::source::timeout_add_local` fires every 1s on the GTK main
//      loop, refreshing both labels with `chrono::Local::now()`. The
//      closure returns `ControlFlow::Continue` so the timeout persists
//      for the lifetime of the process.
//   4. `attach_drag` (from `widgets::mod`) makes the window draggable;
//      the cumulative drag delta is saved to
//      `~/.config/astra/widgets.toml` (real Wayland window movement
//      requires a layer-shell surface, scheduled for post-ISO 4).

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};

use crate::widgets::attach_drag;

/// The clock widget — owns its `ApplicationWindow`.
pub struct ClockWidget {
    #[allow(dead_code)] // held for the lifetime of the app
    window: ApplicationWindow,
}

impl ClockWidget {
    /// Build the clock window. The widget is presented by the caller
    /// via [`ClockWidget::present`].
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Clock")
            .default_width(260)
            .default_height(120)
            .decorated(false)
            .build();
        window.add_css_class("astra-widget");
        window.add_css_class("clock");

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .margin_start(18)
            .margin_end(18)
            .margin_top(16)
            .margin_bottom(16)
            .build();
        window.set_child(Some(&container));

        // 48pt time label — the visual focus of the widget.
        let time_label = Label::builder()
            .label("--:--")
            .css_classes(["clock-time"])
            .build();
        container.append(&time_label);

        // Long-form date label below the time.
        let date_label = Label::builder()
            .label("—")
            .css_classes(["clock-date"])
            .build();
        container.append(&date_label);

        // Per-second tick. Clones of the labels travel into the closure;
        // GTK's ref-counting keeps the underlying GObjects alive.
        let tl = time_label.clone();
        let dl = date_label.clone();
        glib::source::timeout_add_local(std::time::Duration::from_secs(1), move || {
            let now = chrono::Local::now();
            tl.set_label(&now.format("%H:%M").to_string());
            dl.set_label(&now.format("%A %-d %B").to_string());
            glib::ControlFlow::Continue
        });

        // Draggable; drag delta is accumulated and persisted on
        // drag-end. Initial position restore is delegated to the WM —
        // gtk4-rs 0.9 dropped `Window::move_()` (GTK 4.10 deprecation).
        attach_drag(&window, "clock");

        Self { window }
    }

    /// Present the widget on screen.
    pub fn present(&self) {
        self.window.present();
    }
}
