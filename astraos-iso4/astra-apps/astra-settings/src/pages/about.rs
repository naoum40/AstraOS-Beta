// about.rs - About page.
//
// Centered layout: large "✦" symbol (64pt) + "AstraOS" title (24pt bold) +
// version / edition / license / developer lines + a clickable repo link.
//
// This is purely informational — no state, no callbacks beyond the link
// click (which just logs the URL; production would invoke `xdg-open`).

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation};

/// Repository URL surfaced on the About page.
const REPO_URL: &str = "https://github.com/naoum40/AstraOS-Beta";

/// Build the About page root widget.
pub fn build() -> GtkBox {
    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .valign(Align::Center)
        .halign(Align::Center)
        .margin_top(24)
        .build();

    // --- Large ✦ brand symbol ----------------------------------------
    let symbol = Label::builder()
        .label("\u{2726}") // ✦ four-pointed star
        .css_classes(["astra-brand-symbol"])
        .build();
    root.append(&symbol);

    // --- "AstraOS" title ---------------------------------------------
    let title = Label::builder()
        .label("<b>AstraOS</b>")
        .use_markup(true)
        .css_classes(["astra-brand-title"])
        .build();
    root.append(&title);

    // --- Spacer -----------------------------------------------------
    root.append(&gtk4::Box::builder().height_request(8).build());

    // --- Meta info lines --------------------------------------------
    let version = Label::builder()
        .label("Version 0.3.0")
        .css_classes(["astra-meta"])
        .build();
    root.append(&version);

    let edition = Label::builder()
        .label("Edition: Core")
        .css_classes(["astra-meta"])
        .build();
    root.append(&edition);

    let license = Label::builder()
        .label("License: MIT")
        .css_classes(["astra-meta"])
        .build();
    root.append(&license);

    let dev = Label::builder()
        .label("Developer: AstraOS Project (mmtstudio)")
        .css_classes(["astra-meta"])
        .build();
    root.append(&dev);

    // --- Spacer -----------------------------------------------------
    root.append(&gtk4::Box::builder().height_request(12).build());

    // --- Repo link (clickable) --------------------------------------
    let link = Label::builder()
        .label(&format!("<a href=\"{}\">{}</a>", REPO_URL, REPO_URL))
        .use_markup(true)
        .css_classes(["astra-link"])
        .build();
    root.append(&link);

    root
}
