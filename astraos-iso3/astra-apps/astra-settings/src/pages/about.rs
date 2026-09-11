// about.rs - About page.
//
// Shows:
//   - AstraOS logo placeholder ("\u{2726}" four-pointed star, large,
//     styled by `.astra-logo-large` in settings.css with a magenta
//     glow).
//   - "AstraOS" title.
//   - "Version 0.3.0".
//   - "Edition: Core".
//   - "License: MIT".
//   - "Developer: AstraOS Project".
//   - LinkButton to https://github.com/naoum40/AstraOS-Beta.
//
// Everything is centered vertically + horizontally (the page is the
// only one in the settings app that doesn't left-align its content).

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, LinkButton, Orientation};

/// About page - static AstraOS branding + GitHub link.
pub struct AboutPage {
    container: GtkBox,
}

impl AboutPage {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(48)
            .margin_bottom(48)
            .margin_start(48)
            .margin_end(48)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .css_classes(["astra-page", "astra-about"])
            .build();

        // Logo glyph ("\u{2726}" = ✦ four-pointed star). The magenta
        // glow + size come from `.astra-logo-large` in settings.css.
        let logo = Label::builder()
            .label("\u{2726}")
            .css_classes(["astra-logo-large"])
            .build();
        container.append(&logo);

        // Title.
        let title = Label::builder()
            .label("AstraOS")
            .css_classes(["astra-about-title"])
            .build();
        container.append(&title);

        // Version line.
        let version = Label::builder()
            .label("Version 0.3.0")
            .css_classes(["astra-about-info"])
            .build();
        container.append(&version);

        // Edition line.
        let edition = Label::builder()
            .label("Edition: Core")
            .css_classes(["astra-about-info"])
            .build();
        container.append(&edition);

        // License line.
        let license = Label::builder()
            .label("License: MIT")
            .css_classes(["astra-about-info"])
            .build();
        container.append(&license);

        // Developer line.
        let developer = Label::builder()
            .label("Developer: AstraOS Project (mmtstudio)")
            .css_classes(["astra-about-info"])
            .build();
        container.append(&developer);

        // GitHub link.
        let link = LinkButton::builder()
            .label("github.com/naoum40/AstraOS-Beta")
            .uri("https://github.com/naoum40/AstraOS-Beta")
            .css_classes(["astra-about-link"])
            .margin_top(12)
            .build();
        container.append(&link);

        Self { container }
    }

    /// Borrow the root `GtkBox`.
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }
}

impl Default for AboutPage {
    fn default() -> Self {
        Self::new()
    }
}
