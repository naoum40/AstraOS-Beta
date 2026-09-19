// welcome.rs — AstraOS ISO 2 Welcome screen (typing animation)
//
// Full-screen black/glassmorphism background with AstraOS logo,
// title typed character-by-character (60 ms / char), subtitle typed
// afterwards (40 ms / char), blinking caret `|` while typing,
// then fade transition to the next screen after a short pause.
//
// Inspired by proto AstraOS (HTML) lines 10081+, adapted to Rust + GTK4-rs.
//
// Implementation note: the typing animation is a CHAIN of one-shot
// glib timeouts (each tick renders one character and schedules the next).
// Because each inner closure moves the `on_complete` callback one level
// deeper, the closures are `FnOnce` (not `FnMut`), so we use
// `timeout_add_local_once` (accepts `FnOnce`) and type-erase the callback
// as `Box<dyn FnOnce() + 'static>` to avoid infinite monomorphization
// across the recursive `type_chars_recursive` calls.

use glib::source::timeout_add_local_once;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Label, Orientation};
use std::time::Duration;

/// Welcome title — typed at 60 ms / char.
const WELCOME_TITLE: &str = "Bienvenue sur AstraOS";
/// Welcome subtitle — typed at 40 ms / char after the title completes.
const WELCOME_SUBTITLE: &str = "Configuration de votre système...";
/// Per-character delay for the title (proto AstraOS ~60 ms).
const TITLE_DELAY_MS: u64 = 60;
/// Per-character delay for the subtitle (slightly faster).
const SUBTITLE_DELAY_MS: u64 = 40;
/// Initial pause before typing starts (lets the logo fade in).
const INITIAL_DELAY_MS: u64 = 800;
/// Final pause before fading out and handing off to the next screen.
const FINAL_PAUSE_MS: u64 = 1500;

/// Welcome screen controller. Owns the GTK window and launches the
/// typing animation on `present()`. The `on_complete` callback is
/// invoked exactly once after the animation + final pause complete.
pub struct WelcomeScreen {
    window: ApplicationWindow,
}

impl WelcomeScreen {
    pub fn new(app: &gtk4::Application, on_complete: impl FnOnce() + 'static) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("AstraOS Welcome")
            .decorated(false)
            .build();
        // `fullscreen()` is a method on `Window` (via `WindowExt`),
        // not a method on the builder.
        window.fullscreen();
        window.add_css_class("astra-welcome");

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .spacing(20)
            .build();
        window.set_child(Some(&container));

        // Logo placeholder (magenta star glyph; styled by welcome.css).
        let logo = Label::builder()
            .label("\u{2726}") // ✦ four-pointed star
            .css_classes(["astra-welcome-logo"])
            .build();
        container.append(&logo);

        // Title (will be typed character-by-character).
        let title = Label::builder()
            .label("")
            .css_classes(["astra-welcome-title"])
            .build();
        container.append(&title);

        // Subtitle (typed after the title completes).
        let subtitle = Label::builder()
            .label("")
            .css_classes(["astra-welcome-subtitle"])
            .build();
        subtitle.set_opacity(0.0);
        container.append(&subtitle);

        // Type-erase the on_complete callback so the recursive
        // `type_chars_recursive` chain has a single concrete type
        // (avoids infinite monomorphization).
        let on_complete_boxed: Box<dyn FnOnce() + 'static> = Box::new(on_complete);

        // Kick off the animation after the initial delay.
        let title_for_anim = title.clone();
        let subtitle_for_anim = subtitle.clone();
        let window_for_fade = window.clone();
        timeout_add_local_once(Duration::from_millis(INITIAL_DELAY_MS), move || {
            // Type the title, then reveal + type the subtitle, then
            // wait FINAL_PAUSE_MS, then hide the window and call on_complete.
            let subtitle_inner = subtitle_for_anim.clone();
            let window_inner = window_for_fade.clone();
            let after_title: Box<dyn FnOnce() + 'static> = Box::new(move || {
                // Reveal subtitle, then type it.
                subtitle_inner.set_opacity(1.0);
                let window_after = window_inner.clone();
                let after_subtitle: Box<dyn FnOnce() + 'static> = Box::new(move || {
                    // Final pause, then fade + handoff.
                    let window_final = window_after.clone();
                    let final_cb: Box<dyn FnOnce() + 'static> = Box::new(move || {
                        window_final.set_visible(false);
                        on_complete_boxed();
                    });
                    timeout_add_local_once(Duration::from_millis(FINAL_PAUSE_MS), final_cb);
                });
                type_text(
                    subtitle_inner,
                    WELCOME_SUBTITLE,
                    SUBTITLE_DELAY_MS,
                    after_subtitle,
                );
            });
            type_text(title_for_anim, WELCOME_TITLE, TITLE_DELAY_MS, after_title);
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

/// Type `full_text` into `label` one character at a time, with a
/// blinking caret `|` appended while typing. Calls `on_complete`
/// once after the last character is rendered (and the caret is removed).
///
/// This is implemented as a recursive `timeout_add_local_once` chain so
/// we never block the GTK main loop. Each tick renders the next character
/// and schedules the next tick.
fn type_text(
    label: Label,
    full_text: &str,
    delay_ms: u64,
    on_complete: Box<dyn FnOnce() + 'static>,
) {
    type_chars_recursive(label, full_text, 0, delay_ms, on_complete);
}

fn type_chars_recursive(
    label: Label,
    full_text: &str,
    char_index: usize,
    delay_ms: u64,
    on_complete: Box<dyn FnOnce() + 'static>,
) {
    // UTF-8 safe iteration: collect into a Vec<char> so multi-byte
    // glyphs (accents, emoji) are counted as one character each.
    let chars: Vec<char> = full_text.chars().collect();

    if char_index >= chars.len() {
        // Typing complete: remove the caret, render the final text.
        label.set_label(full_text);
        on_complete();
        return;
    }

    // Render the current prefix + blinking caret.
    let current: String = chars.iter().take(char_index + 1).collect();
    let mut with_caret = current;
    with_caret.push('|');
    label.set_label(&with_caret);

    // Schedule the next character (one-shot).
    let label_next = label.clone();
    let full_text_owned = full_text.to_string();
    timeout_add_local_once(Duration::from_millis(delay_ms), move || {
        type_chars_recursive(
            label_next,
            &full_text_owned,
            char_index + 1,
            delay_ms,
            on_complete,
        );
    });
}
