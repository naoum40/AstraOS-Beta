// onboarding.rs — AstraOS ISO 2 Onboarding wizard (6 steps)
//
// Steps:
//   1. Language           — FR / EN / ES / DE
//   2. Timezone           — Europe / Americas / Asia (representative cities)
//   3. Identity           — username + admin code + keyboard layout
//   4. Security           — password + confirm (SHA-256 hash hint)
//   5. Privacy            — diagnostics opt-in + autologin toggle
//   6. Customization      — Win/Mac mode + dark/light + glassmorphism
//
// UI: 6 progress dots at top (active = magenta accent), step content
// fills the middle, Back/Next (or Finish) buttons at bottom-right.
//
// State is held in `Rc<RefCell<OnboardingState>>` so that button-click
// closures (which must be `'static`) can mutate the current step and
// re-render by calling the free `render_step` function.

use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box as GtkBox, Button, ComboBoxText, Entry, Label, Orientation, Switch,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Total number of steps in the wizard.
const TOTAL_STEPS: u32 = 6;

/// Mutable onboarding state shared between the controller and the
/// per-step button callbacks.
struct OnboardingState {
    current_step: u32,
    /// Taken (consumed) exactly once on Finish.
    on_complete: Option<Box<dyn Fn()>>,
}

/// Onboarding wizard controller. Owns the GTK window; `on_complete`
/// is invoked exactly once when the user clicks Finish on step 6.
pub struct Onboarding {
    window: ApplicationWindow,
    state: Rc<RefCell<OnboardingState>>,
}

impl Onboarding {
    pub fn new(app: &gtk4::Application, on_complete: impl Fn() + 'static) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("AstraOS Setup")
            .default_width(800)
            .default_height(600)
            .decorated(false)
            .build();
        window.add_css_class("astra-onboarding");

        let state = Rc::new(RefCell::new(OnboardingState {
            current_step: 1,
            on_complete: Some(Box::new(on_complete)),
        }));

        Self { window, state }
    }

    pub fn present(&self) {
        render_step(&self.window, &self.state);
        self.window.present();
    }
}

/// Build / rebuild the full window contents for the current step.
/// Called on first present and on every Back / Next click.
///
/// This is a free function (not a method on `Onboarding`) so that
/// button-click closures can call it without borrowing `&self`.
fn render_step(window: &ApplicationWindow, state: &Rc<RefCell<OnboardingState>>) {
    let step = state.borrow().current_step;

    // Root container for this step.
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(20)
        .margin_top(40)
        .margin_bottom(40)
        .margin_start(40)
        .margin_end(40)
        .build();
    window.set_child(Some(&container));

    // Progress dots.
    let dots_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .halign(gtk4::Align::Center)
        .spacing(8)
        .build();
    for i in 1..=TOTAL_STEPS {
        let css = if i == step {
            ["astra-dot-active"]
        } else {
            ["astra-dot-inactive"]
        };
        let dot = Label::builder().label("\u{25CF}").css_classes(css).build();
        dots_box.append(&dot);
    }
    container.append(&dots_box);

    // Step content (free functions below).
    let content = match step {
        1 => step_language(),
        2 => step_timezone(),
        3 => step_identity(),
        4 => step_security(),
        5 => step_privacy(),
        6 => step_customization(),
        // Defensive: should never happen (TotalSteps = 6), but if it
        // does, jump straight to the completion screen.
        _ => step_complete(),
    };
    content.set_vexpand(true);
    container.append(&content);

    // Navigation row.
    let nav_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .halign(gtk4::Align::End)
        .spacing(12)
        .build();

    if step > 1 {
        let back_btn = Button::builder()
            .label("\u{2190} Pr\u{00E9}c\u{00E9}dent") // ← Précédent
            .css_classes(["astra-button", "astra-back"])
            .build();
        let state_clone = state.clone();
        let window_clone = window.clone();
        back_btn.connect_clicked(move |_| {
            let mut s = state_clone.borrow_mut();
            if s.current_step > 1 {
                s.current_step -= 1;
            }
            drop(s);
            render_step(&window_clone, &state_clone);
        });
        nav_box.append(&back_btn);
    }

    if step < TOTAL_STEPS {
        let next_btn = Button::builder()
            .label("Suivant \u{2192}") // Suivant →
            .css_classes(["astra-button", "astra-next"])
            .build();
        let state_clone = state.clone();
        let window_clone = window.clone();
        next_btn.connect_clicked(move |_| {
            let mut s = state_clone.borrow_mut();
            if s.current_step < TOTAL_STEPS {
                s.current_step += 1;
            }
            drop(s);
            render_step(&window_clone, &state_clone);
        });
        nav_box.append(&next_btn);
    } else {
        let finish_btn = Button::builder()
            .label("\u{2726} Terminer") // ✦ Terminer
            .css_classes(["astra-button", "astra-finish"])
            .build();
        let state_clone = state.clone();
        let window_clone = window.clone();
        finish_btn.connect_clicked(move |_| {
            log::info!("Onboarding complete");
            // TODO (Task 2-c): persist the collected fields to
            // `~/.config/astra/shell.toml` via the `toml` + `ShellConfig`
            // module. For now we just hand off to the next screen.
            window_clone.set_visible(false);
            let cb_opt = state_clone.borrow_mut().on_complete.take();
            if let Some(cb) = cb_opt {
                cb();
            }
        });
        nav_box.append(&finish_btn);
    }

    container.append(&nav_box);
}

// ---------------------------------------------------------------------------
// Per-step content builders. Each returns a `gtk4::Box` ready to be
// appended to the root container. These are stateless (they only build
// the UI; values are read out at Finish time in a later iteration).
// ---------------------------------------------------------------------------

fn step_language() -> GtkBox {
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Choisissez votre langue")
        .css_classes(["astra-step-title"])
        .build();
    box_.append(&title);

    let combo = ComboBoxText::builder().build();
    combo.append_text("Fran\u{00E7}ais"); // Français
    combo.append_text("English");
    combo.append_text("Espa\u{00F1}ol"); // Español
    combo.append_text("Deutsch");
    combo.set_active(Some(0));
    box_.append(&combo);

    box_
}

fn step_timezone() -> GtkBox {
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("S\u{00E9}lectionnez votre fuseau horaire") // Sélectionnez ...
        .css_classes(["astra-step-title"])
        .build();
    box_.append(&title);

    let combo = ComboBoxText::builder().build();
    combo.append_text("Europe/Paris (France)");
    combo.append_text("Europe/London (UK)");
    combo.append_text("America/New_York (EST)");
    combo.append_text("America/Los_Angeles (PST)");
    combo.append_text("Asia/Tokyo (Japan)");
    combo.append_text("Asia/Shanghai (China)");
    combo.set_active(Some(0));
    box_.append(&combo);

    box_
}

fn step_identity() -> GtkBox {
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Identit\u{00E9}") // Identité
        .css_classes(["astra-step-title"])
        .build();
    box_.append(&title);

    let username_entry = Entry::builder()
        .placeholder_text("Nom d'utilisateur")
        .text("astra")
        .build();
    box_.append(&username_entry);

    let admin_code = Entry::builder()
        .placeholder_text("Code admin (optionnel)")
        .visibility(false)
        .build();
    box_.append(&admin_code);

    let keyboard_label = Label::builder().label("Disposition clavier:").build();
    box_.append(&keyboard_label);

    let keyboard_combo = ComboBoxText::builder().build();
    keyboard_combo.append_text("AZERTY (Fran\u{00E7}ais)"); // AZERTY (Français)
    keyboard_combo.append_text("QWERTY (English US)");
    keyboard_combo.append_text("QWERTZ (Deutsch)");
    keyboard_combo.append_text("QWERTY (English UK)");
    keyboard_combo.set_active(Some(0));
    box_.append(&keyboard_combo);

    box_
}

fn step_security() -> GtkBox {
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("S\u{00E9}curit\u{00E9}") // Sécurité
        .css_classes(["astra-step-title"])
        .build();
    box_.append(&title);

    let password_entry = Entry::builder()
        .placeholder_text("Mot de passe")
        .visibility(false)
        .build();
    box_.append(&password_entry);

    let confirm_entry = Entry::builder()
        .placeholder_text("Confirmer le mot de passe")
        .visibility(false)
        .build();
    box_.append(&confirm_entry);

    let hint = Label::builder()
        .label("Le mot de passe sera hash\u{00E9} en SHA-256") // hashé
        .css_classes(["astra-hint"])
        .build();
    box_.append(&hint);

    box_
}

fn step_privacy() -> GtkBox {
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Confidentialit\u{00E9}") // Confidentialité
        .css_classes(["astra-step-title"])
        .build();
    box_.append(&title);

    // Diagnostics toggle.
    let diag_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    let diag_label = Label::builder()
        .label("Envoyer des donn\u{00E9}es de diagnostic anonymes") // données
        .build();
    let diag_switch = Switch::builder().active(false).build();
    diag_box.append(&diag_label);
    diag_box.append(&diag_switch);
    box_.append(&diag_box);

    // Autologin toggle.
    let auto_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    let auto_label = Label::builder().label("Connexion automatique").build();
    let auto_switch = Switch::builder().active(true).build();
    auto_box.append(&auto_label);
    auto_box.append(&auto_switch);
    box_.append(&auto_box);

    box_
}

fn step_customization() -> GtkBox {
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    let title = Label::builder()
        .label("Personnalisation")
        .css_classes(["astra-step-title"])
        .build();
    box_.append(&title);

    // Win/Mac mode.
    let mode_label = Label::builder().label("Style d'interface:").build();
    box_.append(&mode_label);
    let mode_combo = ComboBoxText::builder().build();
    mode_combo.append_text("Windows (taskbar)");
    mode_combo.append_text("Mac (dock)");
    mode_combo.set_active(Some(0));
    box_.append(&mode_combo);

    // Theme mode.
    let theme_label = Label::builder().label("Th\u{00E8}me:").build(); // Thème
    box_.append(&theme_label);
    let theme_combo = ComboBoxText::builder().build();
    theme_combo.append_text("Sombre");
    theme_combo.append_text("Clair");
    theme_combo.set_active(Some(0));
    box_.append(&theme_combo);

    // Glassmorphism toggle.
    let glass_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    let glass_label = Label::builder()
        .label("Effets de transparence (glassmorphism)")
        .build();
    let glass_switch = Switch::builder().active(true).build();
    glass_box.append(&glass_label);
    glass_box.append(&glass_switch);
    box_.append(&glass_box);

    box_
}

/// Completion screen shown after the user clicks Finish.
/// Not reachable via normal navigation (step 6 is the last), but kept
/// as a defensive fallback and for potential future "summary" views.
fn step_complete() -> GtkBox {
    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .valign(gtk4::Align::Center)
        .build();

    let checkmark = Label::builder()
        .label("\u{2713}") // ✓
        .css_classes(["astra-checkmark"])
        .build();
    box_.append(&checkmark);

    let title = Label::builder()
        .label("Configuration termin\u{00E9}e !") // terminée
        .css_classes(["astra-step-title"])
        .build();
    box_.append(&title);

    let subtitle = Label::builder()
        .label("Votre AstraOS est pr\u{00EA}t \u{00E0} \u{00EA}tre utilis\u{00E9}.") // prêt à être utilisé
        .css_classes(["astra-step-subtitle"])
        .build();
    box_.append(&subtitle);

    box_
}
