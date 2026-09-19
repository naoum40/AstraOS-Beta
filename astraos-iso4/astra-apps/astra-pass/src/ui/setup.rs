// setup.rs - First-time setup screen.
//
// Shown when no vault file exists yet on disk. Walks the user through
// creating their first master password:
//
//   1. Master password entry (visibility=false).
//   2. Confirm password entry (visibility=false).
//   3. Live strength indicator (weak / medium / strong).
//   4. "Create vault" button (default, activated on Enter).
//   5. Error label for mismatch / too-short / etc.
//
// State:
//   - `on_create`: callback fired with the validated master password.
//     The parent creates the vault + saves it to disk.
//
// Password strength is a simple heuristic (not a substitute for
// zxcvbn): length + character-class diversity. Good enough for ISO 4
// UX feedback.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Entry, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Minimum master password length.
const MIN_PASSWORD_LEN: usize = 8;

/// The setup screen widget.
#[derive(Clone)]
pub struct SetupScreen {
    widget: GtkBox,
    password: Entry,
    confirm: Entry,
    strength_label: Label,
    error_label: Label,
    on_create: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl SetupScreen {
    /// Build the setup screen.
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .halign(Align::Center)
            .valign(Align::Center)
            .css_classes(["astra-setup"])
            .build();

        let brand = Label::builder()
            .label("<b>\u{2726} AstraPass</b>") // ✦ AstraPass
            .use_markup(true)
            .css_classes(["astra-brand-large"])
            .build();
        widget.append(&brand);

        let subtitle = Label::builder()
            .label("Create a master password to secure your vault")
            .css_classes(["astra-subtitle"])
            .build();
        widget.append(&subtitle);

        // Master password.
        let password = Entry::builder()
            .placeholder_text("Master password")
            .visibility(false)
            .width_chars(28)
            .css_classes(["astra-field", "astra-password"])
            .build();
        widget.append(&password);

        // Confirm.
        let confirm = Entry::builder()
            .placeholder_text("Confirm master password")
            .visibility(false)
            .width_chars(28)
            .css_classes(["astra-field", "astra-password"])
            .build();
        widget.append(&confirm);

        // Strength indicator (updates on every keystroke in `password`).
        let strength_label = Label::builder()
            .label("")
            .halign(Align::Start)
            .css_classes(["astra-strength"])
            .build();
        widget.append(&strength_label);

        // Error label — hidden until validation fails.
        let error_label = Label::builder()
            .label("")
            .css_classes(["astra-error"])
            .visible(false)
            .build();
        widget.append(&error_label);

        // Create button.
        let create_btn = Button::builder()
            .label("Create vault")
            .css_classes(["astra-btn", "astra-btn-primary"])
            .build();
        widget.append(&create_btn);

        let on_create: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let screen = Self {
            widget: widget.clone(),
            password: password.clone(),
            confirm: confirm.clone(),
            strength_label: strength_label.clone(),
            error_label: error_label.clone(),
            on_create: on_create.clone(),
        };

        // Update the strength indicator on every text change in the
        // master password field. `connect_changed` fires for every
        // edit including paste.
        let screen_for_strength = screen.clone();
        password.connect_changed(move |_| {
            let text = screen_for_strength.password.text().to_string();
            let (label, css) = strength_of(&text);
            screen_for_strength.strength_label.set_label(&label);
            // Reset CSS classes — only one of weak/medium/strong is
            // ever applied at a time.
            for c in ["weak", "medium", "strong"] {
                screen_for_strength.strength_label.remove_css_class(c);
            }
            screen_for_strength.strength_label.add_css_class(css);
        });

        // Wire the Create button: validate + fire the callback.
        let screen_for_create = screen.clone();
        create_btn.connect_clicked(move |_| {
            let pw = screen_for_create.password.text().to_string();
            let cf = screen_for_create.confirm.text().to_string();

            if pw.len() < MIN_PASSWORD_LEN {
                screen_for_create.show_error(&format!(
                    "Master password must be at least {} characters",
                    MIN_PASSWORD_LEN
                ));
                return;
            }
            if pw != cf {
                screen_for_create.show_error("Passwords do not match");
                return;
            }

            // Hide any prior error + fire the callback.
            screen_for_create.error_label.set_visible(false);
            if let Some(cb) = screen_for_create.on_create.borrow().as_ref() {
                cb(&pw);
            }
        });

        // Enter on the confirm field → click Create.
        let create_btn_clone = create_btn.clone();
        confirm.connect_activate(move |_| {
            create_btn_clone.emit_clicked();
        });

        screen
    }

    /// Install the create callback. The closure receives the validated
    /// master password (clear-text).
    pub fn set_on_create<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_create.borrow_mut() = Some(Box::new(f));
    }

    /// Show an error message on the setup screen.
    pub fn show_error(&self, message: &str) {
        self.error_label.set_label(message);
        self.error_label.set_visible(true);
        self.password.set_text("");
        self.confirm.set_text("");
    }

    /// Grab keyboard focus on the master password entry.
    pub fn grab_focus(&self) {
        self.password.grab_focus();
    }

    /// Borrow the underlying GTK widget for layout parenting.
    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
}

/// Compute a (label, css-class) strength rating for `password`.
///
/// Heuristic:
///   - < 8 chars  → weak (regardless of diversity)
///   - else: count character classes (lower, upper, digit, symbol)
///     - 1-2 classes → weak
///     - 3 classes   → medium
///     - 4 classes   → strong
fn strength_of(password: &str) -> (String, &'static str) {
    if password.is_empty() {
        return (String::new(), "");
    }
    let len = password.chars().count();
    let mut classes = 0;
    if password.chars().any(|c| c.is_ascii_lowercase()) {
        classes += 1;
    }
    if password.chars().any(|c| c.is_ascii_uppercase()) {
        classes += 1;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        classes += 1;
    }
    if password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        classes += 1;
    }

    if len < 8 || classes <= 2 {
        ("Weak".to_string(), "weak")
    } else if classes == 3 {
        ("Medium".to_string(), "medium")
    } else {
        ("Strong".to_string(), "strong")
    }
}

#[cfg(test)]
mod tests {
    use super::strength_of;

    #[test]
    fn short_is_weak() {
        assert_eq!(strength_of("abc").1, "weak");
        assert_eq!(strength_of("Ab1").1, "weak");
    }

    #[test]
    fn long_low_diversity_is_weak() {
        // 8 chars but only lowercase
        assert_eq!(strength_of("abcdefgh").1, "weak");
    }

    #[test]
    fn three_classes_is_medium() {
        assert_eq!(strength_of("Abcdefg1").1, "medium");
    }

    #[test]
    fn four_classes_is_strong() {
        assert_eq!(strength_of("Abcdefg1!").1, "strong");
    }

    #[test]
    fn empty_is_no_class() {
        let (label, css) = strength_of("");
        assert_eq!(label, "");
        assert_eq!(css, "");
    }
}
