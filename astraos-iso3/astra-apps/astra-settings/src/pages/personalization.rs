// personalization.rs - Personalization page.
//
// Surfaces every visual knob the user can flip:
//
//   - Theme toggle (Dark/Light)              - GtkSwitch
//   - Accent color picker (8 colors, 4x2)     - GtkGrid of GtkButtons
//   - Wallpaper gallery (3 thumbnails)        - GtkBox horizontal of
//                                               GtkButtons
//   - Taskbar mode (Win/Mac)                  - GtkComboBoxText
//   - Glassmorphism toggle                    - GtkSwitch
//
// Each control writes back to the shared `SettingsConfig` cell (the same
// one owned by the content area) and immediately persists to
// `~/.config/astra/desktop.toml` via `SettingsConfig::save()`.

use crate::config::SettingsConfig;
use glib::Propagation;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, ComboBoxText, Grid, Label, Orientation, Switch};
use std::cell::RefCell;
use std::rc::Rc;

/// Accent colors offered in the picker. Each is `(hex, css_safe_class,
/// display_name)`. The hex (minus the leading `#`) is reused as the
/// CSS class so `settings.css` can color each button without inline
/// styles.
const ACCENT_COLORS: [(&str, &str, &str); 8] = [
    ("#0078d7", "0078d7", "Azure"),
    ("#107C10", "107C10", "Verdure"),
    ("#D83B01", "D83B01", "Sunset"),
    ("#8E8E93", "8E8E93", "Slate"),
    ("#FF2D55", "FF2D55", "Magenta"),
    ("#5856D6", "5856D6", "Violet"),
    ("#00B7C3", "00B7C3", "Teal"),
    ("#FFB900", "FFB900", "Gold"),
];

/// Wallpaper thumbnails. `(id, display_name)`. The id is used both as
/// the button name (so CSS `#default-violet` matches it) and as the
/// basename written to the config (with a `.png` suffix appended).
const WALLPAPERS: [(&str, &str); 3] = [
    ("default-violet", "Default Violet"),
    ("default-bleu", "Default Blue"),
    ("classic", "Classic"),
];

type SharedCfg = Rc<RefCell<SettingsConfig>>;

/// Personalization page - holds the 5 visual controls + their helpers.
pub struct PersonalizationPage {
    container: GtkBox,
    cfg: SharedCfg,
}

impl PersonalizationPage {
    /// Build the page. `cfg` is the shared config cell owned by the
    /// content area.
    pub fn new(cfg: &SharedCfg) -> Self {
        let cfg_cell = cfg.clone();

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(18)
            .margin_top(36)
            .margin_bottom(36)
            .margin_start(48)
            .margin_end(48)
            .css_classes(["astra-page", "astra-personalization"])
            .build();

        // Page header.
        let header = Label::builder()
            .label("Personalization")
            .css_classes(["astra-page-title"])
            .halign(gtk4::Align::Start)
            .build();
        container.append(&header);

        // ---- 1. Theme toggle -------------------------------------
        let theme_row = build_row("Theme", "Switch between Dark and Light");
        let theme_active = cfg_cell.borrow().theme == "light";
        let theme_switch = Switch::builder()
            .active(theme_active)
            .css_classes(["astra-switch"])
            .valign(gtk4::Align::Center)
            .build();
        {
            let cfg_clone = cfg_cell.clone();
            theme_switch.connect_state_set(move |_, state| {
                let theme = if state { "light" } else { "dark" };
                cfg_clone.borrow_mut().theme = theme.to_string();
                if let Err(e) = cfg_clone.borrow().save() {
                    log::warn!("Failed to save theme: {}", e);
                }
                log::info!("Theme set to {}", theme);
                Propagation::Proceed
            });
        }
        theme_row.append(&theme_switch);
        container.append(&theme_row);

        // ---- 2. Accent color picker ------------------------------
        let accent_label = Label::builder()
            .label("Accent color")
            .css_classes(["astra-section-title"])
            .halign(gtk4::Align::Start)
            .margin_top(6)
            .build();
        container.append(&accent_label);

        let accent_grid = Grid::builder()
            .css_classes(["astra-accent-grid"])
            .column_spacing(12)
            .row_spacing(12)
            .halign(gtk4::Align::Start)
            .build();

        for (i, (hex, css_class, display)) in ACCENT_COLORS.iter().enumerate() {
            let btn = Button::builder()
                .css_classes(["astra-color-button", &format!("accent-{}", css_class)])
                .width_request(48)
                .height_request(48)
                .tooltip_text(*display)
                .build();

            let cfg_clone = cfg_cell.clone();
            let hex_owned = hex.to_string();
            btn.connect_clicked(move |_| {
                cfg_clone.borrow_mut().accent_color = hex_owned.clone();
                if let Err(e) = cfg_clone.borrow().save() {
                    log::warn!("Failed to save accent color: {}", e);
                }
                log::info!("Accent color set to {}", hex_owned);
            });

            let row = (i / 4) as i32;
            let col = (i % 4) as i32;
            accent_grid.attach(&btn, col, row, 1, 1);
        }
        container.append(&accent_grid);

        // ---- 3. Wallpaper gallery ---------------------------------
        let wallpaper_label = Label::builder()
            .label("Wallpaper")
            .css_classes(["astra-section-title"])
            .halign(gtk4::Align::Start)
            .margin_top(6)
            .build();
        container.append(&wallpaper_label);

        let wallpaper_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .halign(gtk4::Align::Start)
            .build();
        for (id, display) in WALLPAPERS.iter() {
            let thumb = Button::builder()
                .css_classes(["astra-wallpaper-thumb"])
                .name(*id)
                .tooltip_text(*display)
                .width_request(140)
                .height_request(80)
                .build();

            let cfg_clone = cfg_cell.clone();
            let id_owned = id.to_string();
            thumb.connect_clicked(move |_| {
                let filename = format!("{}.png", id_owned);
                cfg_clone.borrow_mut().wallpaper = filename.clone();
                if let Err(e) = cfg_clone.borrow().save() {
                    log::warn!("Failed to save wallpaper: {}", e);
                }
                log::info!("Wallpaper set to {}", filename);
            });

            wallpaper_box.append(&thumb);
        }
        container.append(&wallpaper_box);

        // ---- 4. Taskbar mode --------------------------------------
        let taskbar_row = build_row("Taskbar mode", "Windows or Mac style");
        let taskbar_combo = ComboBoxText::builder()
            .css_classes(["astra-combo"])
            .valign(gtk4::Align::Center)
            .build();
        taskbar_combo.append(Some("win"), "Windows");
        taskbar_combo.append(Some("mac"), "Mac");
        taskbar_combo.set_active_id(Some(&cfg_cell.borrow().taskbar_mode));
        {
            let cfg_clone = cfg_cell.clone();
            taskbar_combo.connect_changed(move |combo| {
                if let Some(id) = combo.active_id() {
                    let id_str = id.to_string();
                    cfg_clone.borrow_mut().taskbar_mode = id_str.clone();
                    if let Err(e) = cfg_clone.borrow().save() {
                        log::warn!("Failed to save taskbar mode: {}", e);
                    }
                    log::info!("Taskbar mode set to {}", id_str);
                }
            });
        }
        taskbar_row.append(&taskbar_combo);
        container.append(&taskbar_row);

        // ---- 5. Glassmorphism toggle ------------------------------
        let glass_row = build_row("Glassmorphism", "Translucent panels with blur");
        let glass_switch = Switch::builder()
            .active(cfg_cell.borrow().glassmorphism)
            .css_classes(["astra-switch"])
            .valign(gtk4::Align::Center)
            .build();
        {
            let cfg_clone = cfg_cell.clone();
            glass_switch.connect_state_set(move |_, state| {
                cfg_clone.borrow_mut().glassmorphism = state;
                if let Err(e) = cfg_clone.borrow().save() {
                    log::warn!("Failed to save glassmorphism: {}", e);
                }
                log::info!("Glassmorphism set to {}", state);
                Propagation::Proceed
            });
        }
        glass_row.append(&glass_switch);
        container.append(&glass_row);

        Self {
            container,
            cfg: cfg_cell,
        }
    }

    /// Borrow the root `GtkBox`.
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Borrow the shared config cell (kept for future pages that may
    /// want to read the live config).
    #[allow(dead_code)]
    pub fn config(&self) -> &SharedCfg {
        &self.cfg
    }
}

/// Build a horizontal "row" widget: a titled subtitle on the left
/// (hexpand) + space on the right for a switch / combo / button.
///
/// The caller appends the actual control to the returned `GtkBox`.
fn build_row(title: &str, subtitle: &str) -> GtkBox {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .css_classes(["astra-row"])
        .build();

    let labels = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();

    let t = Label::builder()
        .label(title)
        .css_classes(["astra-row-title"])
        .halign(gtk4::Align::Start)
        .build();
    let s = Label::builder()
        .label(subtitle)
        .css_classes(["astra-row-subtitle"])
        .halign(gtk4::Align::Start)
        .build();

    labels.append(&t);
    labels.append(&s);
    row.append(&labels);
    row
}
