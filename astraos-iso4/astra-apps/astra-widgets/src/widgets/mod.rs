// widgets/mod.rs - Astra widget registry + shared drag helper.
//
// Each widget (clock / calendar / weather / battery) lives in its own
// module under `widgets/`. The `spawn` function below is the central
// dispatch: given a widget name from the user config, it constructs the
// matching widget window and presents it on screen.
//
// The `attach_drag` helper wires up a `gtk4::GestureDrag` on any
// `ApplicationWindow` so the user can reposition the widget with the
// mouse. On drag-end the cumulative delta is added to the last known
// position and persisted to `~/.config/astra/widgets.toml`.
//
// NOTE on gtk4-rs 0.9 API: `Window::move_()` / `Window::position()`
// were removed from the C API in GTK 4.10 and the bindings no longer
// expose them. Real Wayland window movement therefore requires a
// layer-shell surface (planned post-ISO 4); for ISO 4 we still wire up
// the gesture + persist the position so the next session on a
// layer-shell-aware host restores the layout faithfully.

pub mod battery;
pub mod calendar;
pub mod clock;
pub mod weather;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, GestureDrag};

use crate::config::WidgetsConfig;

/// Spawn a widget by name. Returns `true` if the widget was found and
/// presented; `false` (with a logged warning) for unknown names so the
/// main loop can continue booting the remaining widgets.
pub fn spawn(name: &str, app: &Application) -> bool {
    match name {
        "clock" => {
            clock::ClockWidget::new(app).present();
            true
        }
        "calendar" => {
            calendar::CalendarWidget::new(app).present();
            true
        }
        "weather" => {
            weather::WeatherWidget::new(app).present();
            true
        }
        "battery" => {
            battery::BatteryWidget::new(app).present();
            true
        }
        other => {
            log::warn!("Unknown widget name in config: {}", other);
            false
        }
    }
}

/// Wire up a `GestureDrag` on `window` so the user can drag it around
/// the desktop.
///
/// Because GTK4-rs 0.9 no longer exposes `Window::move_()` / `position()`
/// (deprecated C API removed in GTK 4.10), we delegate the real visual
/// move to the compositor via `gdk::Toplevel::begin_move` (called from
/// `drag-begin`) and independently accumulate the drag delta so we can
/// persist the user's positional intent to `~/.config/astra/widgets.toml`.
pub fn attach_drag(window: &ApplicationWindow, key: &str) {
    let drag = GestureDrag::new();
    drag.set_button(gtk4::gdk::BUTTON_PRIMARY);
    window.add_controller(drag.clone());

    // Cumulative drag delta captured on each `drag-update`, so we can
    // log + persist the user's intent on `drag-end`. Wrapped in
    // `Rc<RefCell<_>>` because each closure needs its own borrow of the
    // same mutable slot.
    let delta: Rc<RefCell<(f64, f64)>> = Rc::new(RefCell::new((0.0, 0.0)));

    {
        let delta = delta.clone();
        let window = window.clone();
        drag.connect_drag_begin(move |gesture, _start_x, _start_y| {
            // Reset the delta tracker at drag start.
            *delta.borrow_mut() = (0.0, 0.0);
            // Hand interactive-move control to the compositor. On
            // Wayland this is the only sanctioned path; on X11 it
            // also produces a smooth WM-driven move.
            begin_compositor_move(&window, gesture);
        });
    }

    {
        let delta = delta.clone();
        drag.connect_drag_update(move |_, dx, dy| {
            *delta.borrow_mut() = (dx, dy);
        });
    }

    {
        let delta = delta.clone();
        let key = key.to_string();
        drag.connect_drag_end(move |_, dx, dy| {
            log::info!("[{}] dragged by ({:.0},{:.0})", key, dx, dy);
            let mut cfg = WidgetsConfig::load();
            // Accumulate the delta onto the last known position so the
            // config tracks the user's drag intent across sessions.
            let (lx, ly) = cfg.positions.get(&key).copied().unwrap_or((0, 0));
            let new_pos = (lx + dx as i32, ly + dy as i32);
            cfg.positions.insert(key.clone(), new_pos);
            if let Err(e) = cfg.save() {
                log::error!("[{}] failed to persist position: {}", key, e);
            }
            let _ = delta; // keep slot alive past this closure
        });
    }
}

/// Trigger an interactive compositor-driven move on `window`'s
/// toplevel surface.
///
/// `gdk::Toplevel::begin_move` is called from the `drag-begin` handler
/// so the WM takes over the actual pointer-driven move. The gesture
/// device + last-event timestamp are passed so the compositor can
/// match the move to the originating pointer event.
///
/// Failure to acquire the surface / toplevel / device is logged at
/// debug level — the widget remains interactive even if the compositor
/// refuses the interactive-move request (rare in practice).
fn begin_compositor_move(window: &ApplicationWindow, gesture: &GestureDrag) {
    let Some(surface) = window.surface() else {
        log::debug!("[drag] window has no surface yet — not realized");
        return;
    };
    // `Surface` doesn't statically declare `@implements Toplevel` in
    // the GIR, so `dynamic_cast_ref` is needed to perform the runtime
    // interface check (any toplevel surface satisfies it after
    // `present()`).
    let Some(toplevel) = surface.dynamic_cast_ref::<gtk4::gdk::Toplevel>() else {
        log::debug!("[drag] surface is not a Toplevel — cannot begin_move");
        return;
    };
    let Some(device) = gesture.device() else {
        log::debug!("[drag] gesture has no associated device");
        return;
    };
    let seq = gesture.current_sequence();
    let last_event = gesture.last_event(seq.as_ref());
    let timestamp = last_event.as_ref().map(|e| e.time()).unwrap_or(0);
    let (x, y) = last_event
        .as_ref()
        .and_then(|e| e.position())
        .unwrap_or((0.0, 0.0));
    let button = gesture.button() as i32;
    toplevel.begin_move(&device, button, x, y, timestamp);
}
