// ==============================================================================
// AstraOS Dock — Preferences UI (traduit de MacOSDock/prefs.ts → Rust/GTK4)
// ==============================================================================
// Source : 458 lignes TS. Fenêtre de paramètres Adw (libadwaita).
//
// TRADUIT :
//   - Fenêtre de préférences avec groupes (Appearance, Position, Magnification, etc.)
//   - SpinRow (icon size), ComboRow (position), SwitchRow (toggles), ScaleRow (sliders)
//   - Bind settings → widgets
//
// GNOME → GTK4 :
//   Adw.PreferencesWindow → gtk4::Window (ou libadwaita PreferencesWindow si dispo)
//   Adw.SpinRow → gtk4::SpinButton
//   Adw.ComboRow → gtk4::DropDown
//   Adw.SwitchRow → gtk4::Switch
//   Adw.ScaleRow → gtk4::Scale
//   GSettings bind → GSettings bind (glib, marche en GTK4)
// ==============================================================================
use gtk4::prelude::*;
use gtk4::{Box, DropDown, Label, Orientation, Scale, SpinButton, Switch, Window};
use gio::prelude::*;

/// Crée la fenêtre de préférences du dock.
/// Traduit de fillPreferencesWindow().
pub fn create_prefs_window(settings: &gio::Settings) -> Window {
    let window = Window::new();
    window.set_title(Some("AstraOS Dock — Settings"));
    window.set_default_size(400, 600);

    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);

    // --- Appearance ---
    main_box.append(&section_label("Appearance"));

    // Icon size (SpinButton 16-96)
    let icon_spin = SpinButton::with_range(16.0, 96.0, 2.0);
    icon_spin.set_value(settings.int("icon-size") as f64);
    settings.bind("icon-size", &icon_spin, "value", gio::SettingsBindFlags::DEFAULT);
    main_box.append(&labeled_row("Icon size", &icon_spin));

    // Dock opacity (Scale 0-100)
    let opacity_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 5.0);
    opacity_scale.set_value(settings.int("dock-opacity") as f64);
    opacity_scale.set_draw_value(true);
    main_box.append(&labeled_row("Dock opacity (%)", &opacity_scale));

    // Border radius (SpinButton 0-30)
    let radius_spin = SpinButton::with_range(0.0, 30.0, 1.0);
    radius_spin.set_value(settings.int("dock-border-radius") as f64);
    main_box.append(&labeled_row("Border radius (px)", &radius_spin));

    // --- Position ---
    main_box.append(&section_label("Position"));

    let position_model = gtk4::StringList::new(&["Bottom", "Left", "Right", "Top"]);
    let position_dropdown = DropDown::new(Some(position_model), None);
    position_dropdown.set_selected(settings.int("dock-position") as u32);
    main_box.append(&labeled_row("Dock position", &position_dropdown));

    // --- Magnification ---
    main_box.append(&section_label("Magnification"));

    let mag_enabled = Switch::new();
    mag_enabled.set_active(settings.boolean("magnification-enabled"));
    main_box.append(&labeled_row("Enable magnification", &mag_enabled));

    let mag_scale = Scale::with_range(Orientation::Horizontal, 1.0, 2.0, 0.05);
    mag_scale.set_value(settings.double("magnification-scale"));
    mag_scale.set_draw_value(true);
    main_box.append(&labeled_row("Magnification scale", &mag_scale));

    let falloff_spin = SpinButton::with_range(20.0, 300.0, 10.0);
    falloff_spin.set_value(settings.int("magnification-falloff") as f64);
    main_box.append(&labeled_row("Falloff distance (px)", &falloff_spin));

    // --- Behavior ---
    main_box.append(&section_label("Behavior"));

    let auto_hide = Switch::new();
    auto_hide.set_active(settings.boolean("auto-hide"));
    main_box.append(&labeled_row("Auto-hide", &auto_hide));

    let bounce = Switch::new();
    bounce.set_active(settings.boolean("bounce-on-launch"));
    main_box.append(&labeled_row("Bounce on launch", &bounce));

    let running_ind = Switch::new();
    running_ind.set_active(settings.boolean("running-indicators"));
    main_box.append(&labeled_row("Running indicators", &running_ind));

    // --- Window Previews ---
    main_box.append(&section_label("Window Previews"));

    let previews = Switch::new();
    previews.set_active(settings.boolean("window-previews"));
    main_box.append(&labeled_row("Window previews", &previews));

    window.set_child(Some(&main_box));
    window
}

fn section_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("dock-prefs-section");
    label.set_halign(gtk4::Align::Start);
    label.set_margin_top(12);
    label
}

fn labeled_row(label_text: &str, widget: &impl IsA<gtk4::Widget>) -> Box {
    let row = Box::new(Orientation::Horizontal, 12);
    row.set_margin_top(6);
    row.set_margin_bottom(6);

    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);

    row.append(&label);
    row.append(widget);
    row
}
