// calendar.rs - AstraOS Calendar widget.
//
// Floating glassmorphism panel built around `gtk4::Calendar`. Today is
// auto-selected on mount; two nav buttons (`‹` / `›`) above the grid
// switch months by adjusting `Calendar`'s `year`/`month` properties
// (gtk4-rs 0.9 has no `prev_month` / `next_month` methods — the C API
// dropped them in favor of direct property writes).
//
// Pipeline:
//   1. Build an undecorated `ApplicationWindow` (CSS classes
//      `astra-widget` + `calendar`).
//   2. Vertical `GtkBox`: header row (prev-arrow | month label |
//      next-arrow) + the `gtk4::Calendar` itself.
//   3. `Calendar::select_day(&glib::DateTime::now_local()?)` highlights
//      today; the GTK widget internally styles the current day with
//      its `:today` CSS pseudo-class.
//   4. Nav buttons compute the next/prev month (with year wrap-around)
//      and write it back via `set_month` / `set_year`, then refresh
//      the month label.
//   5. `attach_drag` makes the window draggable (delta persisted to
//      `~/.config/astra/widgets.toml`).

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, Calendar, Label, Orientation};

use crate::widgets::attach_drag;

/// The calendar widget — owns its `ApplicationWindow` + child widgets.
pub struct CalendarWidget {
    #[allow(dead_code)] // held for the lifetime of the app
    window: ApplicationWindow,
}

impl CalendarWidget {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Calendar")
            .default_width(280)
            .default_height(260)
            .decorated(false)
            .build();
        window.add_css_class("astra-widget");
        window.add_css_class("calendar");

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        window.set_child(Some(&container));

        // Header row: prev | month label | next.
        let header = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Fill)
            .build();
        container.append(&header);

        let prev = Button::with_label("‹");
        prev.add_css_class("cal-nav");
        let next = Button::with_label("›");
        next.add_css_class("cal-nav");

        let month_label = Label::builder()
            .label("")
            .css_classes(["cal-month"])
            .hexpand(true)
            .build();
        header.append(&prev);
        header.append(&month_label);
        header.append(&next);

        // The calendar grid itself.
        let cal = Calendar::new();
        cal.add_css_class("cal-grid");
        // Highlight today using the local timezone's current date.
        match glib::DateTime::now_local() {
            Ok(now) => cal.select_day(&now),
            Err(e) => log::warn!(
                "Could not build glib::DateTime::now_local() — today not selected: {}",
                e
            ),
        }
        container.append(&cal);

        // Initial month label.
        refresh_month_label(&cal, &month_label);

        // Prev/Next handlers — compute the new (year, month) and write
        // it back via `set_year` / `set_month` (gtk4-rs 0.9 has no
        // `prev_month` / `next_month` methods).
        {
            let cal = cal.clone();
            let label = month_label.clone();
            prev.connect_clicked(move |_| {
                shift_month(&cal, -1);
                refresh_month_label(&cal, &label);
            });
        }
        {
            let cal = cal.clone();
            let label = month_label.clone();
            next.connect_clicked(move |_| {
                shift_month(&cal, 1);
                refresh_month_label(&cal, &label);
            });
        }

        attach_drag(&window, "calendar");

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

/// Refresh `label` to reflect `cal`'s current month/year — formatted
/// as `October 2025` (full month name + 4-digit year).
fn refresh_month_label(cal: &Calendar, label: &Label) {
    // `Calendar::year()` / `month()` return the displayed (not current)
    // date's components; `month()` is 0-based (0 = January) and typed
    // `i32` in gtk4-rs 0.9 (under the v4_14 feature gate, which we
    // leave off by default).
    let year = cal.year();
    let month0 = cal.month();
    let month_name = month_name(month0);
    label.set_label(&format!("{} {}", month_name, year));
}

/// Shift `cal`'s displayed month by `delta` months (positive = forward,
/// negative = backward), wrapping the year on overflow / underflow.
fn shift_month(cal: &Calendar, delta: i32) {
    let mut m = cal.month() + delta;
    let mut y = cal.year();
    while m < 0 {
        m += 12;
        y -= 1;
    }
    while m > 11 {
        m -= 12;
        y += 1;
    }
    cal.set_year(y);
    cal.set_month(m);
}

/// Map a 0-based month index to its English long-form name.
fn month_name(m0: i32) -> &'static str {
    match m0 {
        0 => "January",
        1 => "February",
        2 => "March",
        3 => "April",
        4 => "May",
        5 => "June",
        6 => "July",
        7 => "August",
        8 => "September",
        9 => "October",
        10 => "November",
        11 => "December",
        _ => "Unknown",
    }
}
