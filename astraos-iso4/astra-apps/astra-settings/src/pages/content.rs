// content.rs - Astra Settings content area (right pane).
//
// A `gtk4::Stack` with 4 children — one per page (Account /
// Personalization / System / About). Calling `switch_to("account")`
// etc. flips the visible child, optionally with a slide transition.
//
// `Content` implements `Clone` so the parent window can hand a copy to
// the sidebar's selection callback — GTK ref-counts the underlying
// `Stack`, so dropping the Rust wrapper is always safe.

use gtk4::{Stack, StackTransitionType};
use std::rc::Rc;

/// The right-hand content area. Holds a `gtk4::Stack` with all 4 pages
/// pre-built at startup — switching pages is just a `set_visible_child_name`
/// call (no widget reconstruction).
#[derive(Clone)]
pub struct Content {
    stack: Rc<Stack>,
}

impl Content {
    /// Build the content stack with all 4 pages pre-attached.
    pub fn new() -> Self {
        let stack = Stack::builder()
            .transition_type(StackTransitionType::Crossfade)
            .transition_duration(180)
            .css_classes(["astra-content"])
            .hexpand(true)
            .vexpand(true)
            .build();

        // Build each page once and attach it under its page key. The
        // pages are self-contained (they don't reference each other),
        // so the order of insertion doesn't matter.
        stack.add_named(&crate::pages::account::build(), Some("account"));
        stack.add_named(
            &crate::pages::personalization::build(),
            Some("personalization"),
        );
        stack.add_named(&crate::pages::system::build(), Some("system"));
        stack.add_named(&crate::pages::about::build(), Some("about"));

        Self {
            stack: Rc::new(stack),
        }
    }

    /// Switch the visible page by key.
    ///
    /// Unknown keys are silently ignored — sidebar callers always pass
    /// one of the 4 valid keys, so this is a defensive no-op.
    pub fn switch_to(&self, page: &str) {
        self.stack.set_visible_child_name(page);
    }

    /// Borrow the underlying GTK widget for layout parenting.
    pub fn widget(&self) -> &Stack {
        &self.stack
    }
}
