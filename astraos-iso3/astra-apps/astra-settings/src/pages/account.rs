// account.rs - Account page.
//
// Shows:
//   - Avatar placeholder (gradient circle with the user initial "A",
//     styled by `.astra-avatar` in settings.css).
//   - Username display ("astra").
//   - User info text ("Local account - Administrator").
//   - "Change password" button (placeholder - logs the click and does
//     nothing else for now; will be wired to a real auth dialog later).
//
// All children are stacked vertically with generous margins so the
// layout breathes.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};

/// Account page - holds the avatar + username + change-password button.
pub struct AccountPage {
    container: GtkBox,
}

impl AccountPage {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(18)
            .margin_top(36)
            .margin_bottom(36)
            .margin_start(48)
            .margin_end(48)
            .css_classes(["astra-page", "astra-account"])
            .build();

        // Page header.
        let header = Label::builder()
            .label("Account")
            .css_classes(["astra-page-title"])
            .halign(gtk4::Align::Start)
            .build();
        container.append(&header);

        // Avatar (gradient circle with "A" - the rest is done in CSS).
        let avatar = Label::builder()
            .label("A")
            .css_classes(["astra-avatar"])
            .width_request(96)
            .height_request(96)
            .halign(gtk4::Align::Start)
            .margin_top(6)
            .build();
        container.append(&avatar);

        // Username.
        let username = Label::builder()
            .label("astra")
            .css_classes(["astra-username"])
            .halign(gtk4::Align::Start)
            .build();
        container.append(&username);

        // User info (small subtitle under the username).
        let info = Label::builder()
            .label("Member of AstraOS")
            .css_classes(["astra-user-info"])
            .halign(gtk4::Align::Start)
            .build();
        container.append(&info);

        // "Change password" button (placeholder action).
        let change_pw = Button::builder()
            .label("Change password")
            .css_classes(["astra-button", "suggested-action"])
            .halign(gtk4::Align::Start)
            .margin_top(12)
            .build();
        change_pw.connect_clicked(|_| {
            log::info!("Change password clicked (placeholder - not yet implemented)");
        });
        container.append(&change_pw);

        Self { container }
    }

    /// Borrow the root `GtkBox`.
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }
}

impl Default for AccountPage {
    fn default() -> Self {
        Self::new()
    }
}
