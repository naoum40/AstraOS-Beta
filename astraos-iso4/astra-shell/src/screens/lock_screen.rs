// lock_screen.rs — AstraOS ISO 2 Lock screen
//
// Full-screen blurred wallpaper background, large clock (Outfit font),
// date below, avatar placeholder (gradient circle), username, pill-shaped
// password input + arrow submit button. SHA-256 (+ static salt) password
// verification against `$HOME/.config/astra/password.hash`. Wrong password
// triggers a shake animation (CSS class toggle).
//
// If no password hash file exists yet (fresh ISO 2 boot), the literal
// `astraos` password is accepted as the default fallback so the ISO is
// testable in VirtualBox without a prior onboarding pass.

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

/// Static per-version salt appended to the password before SHA-256.
/// Bumping the suffix invalidates previously-stored hashes on upgrade.
const PASSWORD_SALT: &[u8] = b"::astra_salt_v1";

/// Duration of the shake animation on wrong password (matches CSS).
const SHAKE_DURATION_MS: u64 = 500;

/// Lock screen controller. Owns the GTK window; `on_unlock` is invoked
/// exactly once after a successful password verification.
pub struct LockScreen {
    window: ApplicationWindow,
}

impl LockScreen {
    pub fn new(app: &gtk4::Application, on_unlock: impl Fn() + 'static) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("AstraOS Lock")
            .decorated(false)
            .build();
        // `fullscreen()` is a method on `Window`, not on the builder.
        window.fullscreen();
        window.add_css_class("astra-lock-screen");

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .spacing(16)
            .build();
        window.set_child(Some(&container));

        // Clock (HH:MM, 24h, Outfit font).
        let clock = Label::builder()
            .label("00:00")
            .css_classes(["astra-lock-clock"])
            .build();
        container.append(&clock);

        // Date (weekday + day + month, locale-aware via chrono).
        let date_label = Label::builder()
            .label("")
            .css_classes(["astra-lock-date"])
            .build();
        container.append(&date_label);

        // Spacer between the clock block and the avatar block.
        let spacer = GtkBox::builder().height_request(40).build();
        container.append(&spacer);

        // Avatar placeholder (gradient circle, styled by welcome.css).
        let avatar = Label::builder()
            .label("\u{1F464}") // 👤
            .css_classes(["astra-lock-avatar"])
            .build();
        container.append(&avatar);

        // Username (default AstraOS user).
        let username = Label::builder()
            .label("astra")
            .css_classes(["astra-lock-username"])
            .build();
        container.append(&username);

        // Password input (pill) + arrow submit button.
        let input_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let password_entry = Entry::builder()
            .placeholder_text("Mot de passe")
            .visibility(false)
            .css_classes(["astra-lock-input"])
            .width_request(260)
            .build();
        input_box.append(&password_entry);

        let submit_btn = Button::builder()
            .label("\u{2192}") // →
            .css_classes(["astra-lock-submit"])
            .build();
        input_box.append(&submit_btn);
        container.append(&input_box);

        // Update clock + date every second.
        let clock_clone = clock.clone();
        let date_clone = date_label.clone();
        glib::source::timeout_add_local(std::time::Duration::from_secs(1), move || {
            let now = chrono::Local::now();
            clock_clone.set_label(&now.format("%H:%M").to_string());
            date_clone.set_label(&now.format("%A %d %B").to_string());
            glib::ControlFlow::Continue
        });

        // Wrap the on_unlock callback so it can be taken exactly once
        // from a shared `Rc<RefCell<Option<Box<dyn Fn()>>>>`.
        let on_unlock_ref: Rc<RefCell<Option<Box<dyn Fn()>>>> =
            Rc::new(RefCell::new(Some(Box::new(on_unlock))));

        // Wire Enter (entry activate) and submit button click to the same
        // handler. We clone the per-handler inputs (entry + window + ref)
        // because `connect_*` closures require `'static` lifetimes.
        let entry_a = password_entry.clone();
        let window_a = window.clone();
        let ref_a = on_unlock_ref.clone();
        password_entry.connect_activate(move |_| {
            do_submit(&entry_a, &window_a, &ref_a);
        });

        let entry_b = password_entry.clone();
        let window_b = window.clone();
        let ref_b = on_unlock_ref.clone();
        submit_btn.connect_clicked(move |_| {
            do_submit(&entry_b, &window_b, &ref_b);
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

/// Shared submit handler: reads + clears the entry, verifies the password,
/// and either hides the window + invokes `on_unlock`, or triggers a shake
/// animation on the entry via the `error` CSS class.
fn do_submit(
    password_entry: &Entry,
    window: &ApplicationWindow,
    on_unlock_ref: &Rc<RefCell<Option<Box<dyn Fn()>>>>,
) {
    let password = password_entry.text().to_string();
    password_entry.set_text("");

    if verify_password(&password) {
        log::info!("Password correct, unlocking");
        window.set_visible(false);
        if let Some(cb) = on_unlock_ref.borrow_mut().take() {
            cb();
        }
    } else {
        log::warn!("Wrong password, shaking");
        // Toggle the `error` CSS class to trigger the CSS shake animation,
        // then remove it after SHAKE_DURATION_MS so the next attempt can
        // re-trigger the animation cleanly.
        password_entry.add_css_class("error");
        let entry_clone = password_entry.clone();
        glib::source::timeout_add_local(
            std::time::Duration::from_millis(SHAKE_DURATION_MS),
            move || {
                entry_clone.remove_css_class("error");
                glib::ControlFlow::Break
            },
        );
    }
}

/// Hash `input` with SHA-256 + static salt, hex-encode, and compare
/// against the stored hash at `$HOME/.config/astra/password.hash`.
///
/// If the hash file doesn't exist (fresh ISO 2 boot, no onboarding yet),
/// accept the literal `astraos` password as a default so the ISO is
/// testable in VirtualBox.
fn verify_password(input: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher.update(PASSWORD_SALT);
    let result = hasher.finalize();
    let input_hash = hex::encode(result);

    let hash_path =
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/home/astra".to_string()))
            .join(".config/astra/password.hash");

    match fs::read_to_string(&hash_path) {
        Ok(stored) => {
            let stored_trimmed = stored.trim();
            // Constant-time-ish comparison (short-circuits on length first,
            // then byte-wise). Not a true constant-time compare, but the
            // threat model here is shoulder-surfing, not timing attacks.
            input_hash == stored_trimmed
        }
        Err(_) => {
            // No password hash file: fresh ISO 2 boot. Accept the
            // documented default password `astraos` so the ISO is
            // usable in VirtualBox before onboarding has been run.
            log::debug!(
                "No password hash file at {:?}, accepting default password",
                hash_path
            );
            input == "astraos"
        }
    }
}
