// system.rs - System page.
//
// Four sections, each rendered as a row (`astra-row` CSS class):
//   1. Performance mode    — `GtkComboBoxText` with "Éco" / "Équilibré" /
//                            "Performance".
//   2. Battery saver       — `GtkSwitch` (default OFF).
//   3. Storage             — `GtkLabel` "Storage: 12.5 GB used / 64 GB total".
//   4. Astra Defender      — `GtkLabel` "🟢 Protected" (green dot).
//
// Combobox + switch changes log their new value; persistence to
// `desktop.toml` lands in a follow-up task.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, ComboBoxText, Label, Orientation, Switch};

/// Build the System page root widget.
pub fn build() -> GtkBox {
    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .valign(Align::Start)
        .margin_top(24)
        .build();

    // --- Page title ---------------------------------------------------
    let title = Label::builder()
        .label("System")
        .css_classes(["astra-section-title"])
        .halign(Align::Start)
        .build();
    root.append(&title);

    // --- 1. Performance mode -----------------------------------------
    root.append(&section_performance());

    // --- 2. Battery saver --------------------------------------------
    root.append(&section_battery());

    // --- 3. Storage --------------------------------------------------
    root.append(&section_storage());

    // --- 4. Astra Defender status -----------------------------------
    root.append(&section_defender());

    root
}

/// Section 1 — Performance mode combo ("Éco" / "Équilibré" / "Performance").
fn section_performance() -> GtkBox {
    let row = make_row();

    let label = Label::builder()
        .label("Performance mode")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let combo = ComboBoxText::builder().build();
    combo.append_text("Éco");
    combo.append_text("Équilibré");
    combo.append_text("Performance");
    combo.set_active(Some(1)); // Équilibré by default
    combo.connect_changed(move |c| {
        if let Some(idx) = c.active() {
            let mode = match idx {
                0 => "eco",
                1 => "balanced",
                _ => "performance",
            };
            log::info!("Performance mode: {} (idx {})", mode, idx);
        }
    });
    row.append(&combo);

    row
}

/// Section 2 — Battery saver switch (default OFF).
fn section_battery() -> GtkBox {
    let row = make_row();

    let label = Label::builder()
        .label("Battery saver")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let switch = Switch::builder().active(false).build();
    switch.connect_notify(Some("active"), move |sw, _ps| {
        log::info!("Battery saver switched: {}", sw.is_active());
    });
    row.append(&switch);

    row
}

/// Section 3 — Storage readout (static label for ISO 3).
fn section_storage() -> GtkBox {
    let row = make_row();

    let label = Label::builder()
        .label("Storage")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let val = Label::builder()
        .label("Storage: 12.5 GB used / 64 GB total")
        .halign(Align::End)
        .css_classes(["astra-storage-val"])
        .build();
    row.append(&val);

    row
}

/// Section 4 — Astra Defender status (green dot + "Protected").
fn section_defender() -> GtkBox {
    let row = make_row();

    let label = Label::builder()
        .label("Astra Defender")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let status = Label::builder()
        .label("\u{1F7E2} Protected") // 🟢 Protected
        .halign(Align::End)
        .css_classes(["astra-defender-status"])
        .build();
    row.append(&status);

    row
}

/// Helper — a styled row with `astra-row` CSS class + horizontal layout.
fn make_row() -> GtkBox {
    GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["astra-row"])
        .build()
}
