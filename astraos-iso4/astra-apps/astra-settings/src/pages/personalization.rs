// personalization.rs - Personalization page.
//
// Five sections, each rendered as a row (`astra-row` CSS class):
//   1. Theme          — `GtkSwitch`, dark ON by default.
//   2. Accent color   — 8 `GtkButton`s in a 2x4 `GtkGrid`, color swatches.
//   3. Wallpaper       — 3 `GtkImage` thumbnails in an `GtkBox`.
//   4. Taskbar mode    — `GtkComboBoxText` with "Windows" / "Mac" options.
//   5. Glassmorphism   — `GtkSwitch`, ON by default.
//
// All toggles log their changes; persistence to `desktop.toml` lands in
// a follow-up task (the in-memory `SettingsConfig` is already wired up
// in `main.rs`, this module just renders the UI).

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, ComboBoxText, Grid, Image, Label, Orientation, Switch};

/// The 8 accent swatches offered in the color picker, in display order.
/// Matches the Windows-11 / macOS accent palette mixed with AstraOS brand.
const ACCENT_COLORS: &[(&str, &str)] = &[
    ("#0078d7", "blue"),
    ("#107C10", "green"),
    ("#D83B01", "orange"),
    ("#8E8E93", "gray"),
    ("#FF2D55", "pink"),
    ("#5856D6", "purple"),
    ("#00B7C3", "teal"),
    ("#FFB900", "yellow"),
];

/// The 3 wallpaper thumbnails. Files are at `/usr/share/backgrounds/astraos/`
/// on a live AstraOS install; outside that, the `GtkImage` falls back to
/// its missing-icon rendering (no crash).
const WALLPAPERS: &[(&str, &str)] = &[
    (
        "/usr/share/backgrounds/astraos/default-violet.png",
        "Violet",
    ),
    ("/usr/share/backgrounds/astraos/default-bleu.png", "Blue"),
    ("/usr/share/backgrounds/astraos/classic.png", "Classic"),
];

/// Build the Personalization page root widget.
pub fn build() -> GtkBox {
    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .valign(Align::Start)
        .margin_top(24)
        .build();

    // --- Page title ---------------------------------------------------
    let title = Label::builder()
        .label("Personalization")
        .css_classes(["astra-section-title"])
        .halign(Align::Start)
        .build();
    root.append(&title);

    // --- 1. Theme (dark switch) --------------------------------------
    root.append(&section_theme());

    // --- 2. Accent color (2x4 grid of swatches) ----------------------
    root.append(&section_accent());

    // --- 3. Wallpaper (3 thumbnails) ---------------------------------
    root.append(&section_wallpaper());

    // --- 4. Taskbar mode (combo) -------------------------------------
    root.append(&section_taskbar());

    // --- 5. Glassmorphism (switch) ----------------------------------
    root.append(&section_glassmorphism());

    root
}

/// Section 1 — Theme toggle (Dark ON by default).
fn section_theme() -> GtkBox {
    let row = make_row();

    let label = Label::builder()
        .label("Dark theme")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let switch = Switch::builder().active(true).build();
    switch.connect_notify(Some("active"), move |sw, _ps| {
        log::info!("Dark theme switched: {}", sw.is_active());
    });
    row.append(&switch);

    row
}

/// Section 2 — Accent color picker (2 rows x 4 cols of swatches).
fn section_accent() -> GtkBox {
    let row = make_row();
    row.set_orientation(Orientation::Vertical);
    row.set_spacing(8);

    let label = Label::builder()
        .label("Accent color")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let grid = Grid::builder().column_spacing(12).row_spacing(12).build();
    for (idx, (hex, name)) in ACCENT_COLORS.iter().enumerate() {
        let col = (idx % 4) as i32;
        let row_idx = (idx / 4) as i32;
        let swatch = Button::builder()
            .css_classes(["astra-color"])
            .tooltip_text(*name)
            .build();
        // Inline background color via a per-button CSS class so the
        // swatch shows the actual accent color (a static class rule
        // in `settings.css` can't know the hex ahead of time).
        let css = format!("button {{ background: {}; }}", hex);
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(&css);
        let style = swatch.style_context();
        style.add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);
        let hex_owned = hex.to_string();
        swatch.connect_clicked(move |_| {
            log::info!("Accent color picked: {}", hex_owned);
        });
        grid.attach(&swatch, col, row_idx, 1, 1);
    }
    row.append(&grid);

    row
}

/// Section 3 — Wallpaper thumbnails (3 `GtkImage`s side-by-side).
fn section_wallpaper() -> GtkBox {
    let row = make_row();
    row.set_orientation(Orientation::Vertical);
    row.set_spacing(8);

    let label = Label::builder()
        .label("Wallpaper")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let thumbs = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    for (path, name) in WALLPAPERS {
        // `GtkImage` has no `clicked` signal in GTK4 — wrap it in a
        // `GtkButton` so we get a click handler for free (an
        // `EventControllerClick` would be the other option).
        //
        // `ImageBuilder::file` accepts a path string (`impl
        // Into<glib::GString>`). A missing file renders the GTK
        // missing-image icon — no crash.
        let img = Image::builder()
            .file(*path)
            .width_request(160)
            .height_request(90)
            .css_classes(["astra-wallpaper-thumb"])
            .tooltip_text(*name)
            .build();
        let path_owned = path.to_string();
        let thumb_btn = Button::builder()
            .css_classes(["astra-wallpaper-btn"])
            .build();
        thumb_btn.set_child(Some(&img));
        thumb_btn.connect_clicked(move |_| {
            log::info!("Wallpaper picked: {}", path_owned);
        });
        thumbs.append(&thumb_btn);
    }
    row.append(&thumbs);

    row
}

/// Section 4 — Taskbar mode combo ("Windows" / "Mac").
fn section_taskbar() -> GtkBox {
    let row = make_row();

    let label = Label::builder()
        .label("Taskbar mode")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let combo = ComboBoxText::builder().build();
    combo.append_text("Windows");
    combo.append_text("Mac");
    combo.set_active(Some(0)); // Windows by default
    combo.connect_changed(move |c| {
        if let Some(idx) = c.active() {
            let mode = if idx == 0 { "win" } else { "mac" };
            log::info!("Taskbar mode: {} (idx {})", mode, idx);
        }
    });
    row.append(&combo);

    row
}

/// Section 5 — Glassmorphism switch (ON by default).
fn section_glassmorphism() -> GtkBox {
    let row = make_row();

    let label = Label::builder()
        .label("Glassmorphism")
        .halign(Align::Start)
        .hexpand(true)
        .build();
    row.append(&label);

    let switch = Switch::builder().active(true).build();
    switch.connect_notify(Some("active"), move |sw, _ps| {
        log::info!("Glassmorphism switched: {}", sw.is_active());
    });
    row.append(&switch);

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
