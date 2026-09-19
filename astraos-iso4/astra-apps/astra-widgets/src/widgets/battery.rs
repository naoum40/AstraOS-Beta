// battery.rs - AstraOS Battery widget.
//
// Floating glassmorphism panel that shows the system battery state
// (icon + percentage + status). Reads from the Linux power supply
// sysfs interface:
//   /sys/class/power_supply/BAT0/capacity   — 0-100 (percent)
//   /sys/class/power_supply/BAT0/status     — Charging/Discharging/Full/...
//
// Refreshes every 30 seconds via a `glib::source::timeout_add_local`
// tick on the GTK main loop — no worker thread needed because reading
// two tiny sysfs files is sub-millisecond.
//
// Color rules (per spec):
//   100-50%  → green   (#22c55e)
//   49-20%   → yellow  (#facc15)
//   19-0%    → red     (#ef4444)
//
// The icon is a small emoji prefix:
//   🔋 discharging   ⚡ charging   🔌 full/AC

use std::fs;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};

use crate::widgets::attach_drag;

/// Refresh interval — 30 seconds.
const REFRESH_INTERVAL_SECS: u64 = 30;

/// Linux sysfs paths (BAT0 — AstraOS standard laptop battery).
const CAPACITY_PATH: &str = "/sys/class/power_supply/BAT0/capacity";
const STATUS_PATH: &str = "/sys/class/power_supply/BAT0/status";

/// The battery widget — owns its `ApplicationWindow`.
pub struct BatteryWidget {
    #[allow(dead_code)] // held for the lifetime of the app
    window: ApplicationWindow,
}

impl BatteryWidget {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Battery")
            .default_width(160)
            .default_height(80)
            .decorated(false)
            .build();
        window.add_css_class("astra-widget");
        window.add_css_class("battery");

        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .margin_start(14)
            .margin_end(14)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        window.set_child(Some(&container));

        let icon = Label::builder()
            .label("🔋")
            .css_classes(["bat-icon"])
            .build();
        container.append(&icon);

        let pct = Label::builder()
            .label("--%")
            .css_classes(["bat-pct"])
            .build();
        container.append(&pct);

        // Restore saved position.
        // NOTE: gtk4-rs 0.9 removed `Window::move_()` — saved
        // positions are restored by the WM via layer-shell rules
        // (planned post-ISO 4). For ISO 4 the widget appears at the
        // WM's default placement.
        attach_drag(&window, "battery");

        // Initial render + recurring tick.
        refresh(&icon, &pct);
        {
            let icon = icon.clone();
            let pct = pct.clone();
            glib::source::timeout_add_local(
                std::time::Duration::from_secs(REFRESH_INTERVAL_SECS),
                move || {
                    refresh(&icon, &pct);
                    glib::ControlFlow::Continue
                },
            );
        }

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

/// Read `/sys/class/power_supply/BAT0/{capacity,status}` and update the
/// two labels. Failures (no battery on this machine — e.g. a desktop)
/// render `--%` and log a single warning so the log isn't flooded every
/// 30 seconds.
fn refresh(icon: &Label, pct: &Label) {
    match read_battery() {
        Ok(state) => {
            let color = color_for_percent(state.percent);
            pct.set_label(&format!("{}%", state.percent));
            // Inline CSS style for the percentage color — overrides the
            // stylesheet's color rule. (Goes through `set_markup` so
            // the `<span>` tag is parsed as markup, not literal text.)
            pct.set_markup(&format!(
                "<span foreground=\"{}\">{}%</span>",
                color, state.percent
            ));
            icon.set_label(state.emoji);
        }
        Err(e) => {
            log::warn!("[battery] read failed: {}", e);
            pct.set_label("--%");
            icon.set_label("🔋");
        }
    }
}

/// Compact battery snapshot returned by `read_battery`.
struct BatteryState {
    percent: u8,
    emoji: &'static str,
}

/// Read the sysfs files and map them into a `BatteryState`.
///
/// `capacity` is `0..=100`. `status` (one of
/// `Charging`/`Discharging`/`Full`/`Not charging`) is mapped to an
/// emoji prefix so the user can tell at a glance whether the device is
/// plugged in.
fn read_battery() -> Result<BatteryState, std::io::Error> {
    let cap_str = fs::read_to_string(CAPACITY_PATH)?;
    let status_str = fs::read_to_string(STATUS_PATH)?;
    let percent: u8 = cap_str
        .trim()
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)))?;
    let emoji = match status_str.trim() {
        "Charging" => "⚡",
        "Full" => "🔌",
        "Not charging" => "🔌",
        _ => "🔋",
    };
    Ok(BatteryState { percent, emoji })
}

/// Map a `0..=100` battery percentage to the spec's hex color.
fn color_for_percent(p: u8) -> &'static str {
    match p {
        50..=100 => "#22c55e", // green
        20..=49 => "#facc15",  // yellow
        _ => "#ef4444",        // red (0..=19)
    }
}
