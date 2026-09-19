// account.rs - Account page.
//
// Avatar placeholder (a 96x96 `gtk4::Box` styled with a violet→magenta
// gradient CSS, contains the single letter "A" centered) + the username
// "astra" + a "Change password" stub button + "Member of AstraOS"
// caption.
//
// The "Change password" button is intentionally a placeholder: in
// production it would launch `gnome-keyring` / `passwd`-backed dialog;
// for ISO 3 it just logs the click.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation, Separator};

/// Build the Account page root widget.
pub fn build() -> GtkBox {
    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .valign(Align::Start)
        .margin_top(24)
        .build();

    // --- Page title ---------------------------------------------------
    let title = Label::builder()
        .label("Account")
        .css_classes(["astra-section-title"])
        .halign(Align::Start)
        .build();
    root.append(&title);

    // --- Avatar placeholder -------------------------------------------
    // A 96x96 box styled with a violet→magenta gradient (the AstraOS
    // brand) carrying the single letter "A" in 32pt white. The box
    // is the avatar placeholder; in production this would be replaced
    // by a real `libadwaita::Avatar` once user picture uploads land.
    let avatar = GtkBox::builder()
        .width_request(96)
        .height_request(96)
        .halign(Align::Start)
        .css_classes(["astra-avatar"])
        .build();
    let avatar_letter = Label::builder()
        .label("A")
        .css_classes(["astra-avatar-letter"])
        .build();
    avatar.append(&avatar_letter);
    root.append(&avatar);

    // --- Username -----------------------------------------------------
    let username = Label::builder()
        .label("<b>astra</b>")
        .use_markup(true)
        .halign(Align::Start)
        .css_classes(["astra-username"])
        .build();
    root.append(&username);

    // --- "Member of AstraOS" caption --------------------------------
    let caption = Label::builder()
        .label("Member of AstraOS")
        .halign(Align::Start)
        .css_classes(["astra-caption"])
        .build();
    root.append(&caption);

    // --- Thin separator ---------------------------------------------
    let sep = Separator::builder()
        .orientation(Orientation::Horizontal)
        .css_classes(["astra-sep"])
        .build();
    root.append(&sep);

    // --- "Change password" stub button ------------------------------
    // Placeholder: logs the click. Real implementation would invoke
    // `passwd` or a gnome-keyring prompt in a future task.
    let change_pwd = Button::builder()
        .label("Change password")
        .css_classes(["astra-button"])
        .halign(Align::Start)
        .build();
    change_pwd.connect_clicked(|_| {
        log::info!("Change password clicked — placeholder, no-op for ISO 3");
    });
    root.append(&change_pwd);

    root
}
