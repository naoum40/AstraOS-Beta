// ==============================================================================
// AstraOS Dock — SignalManager (traduit de MacOSDock/signalManager.ts → Rust)
// ==============================================================================
// Source : 34 lignes TS. Simple tracker de connections GObject.
//
// En Rust/GTK4, les signaux sont auto-disconnectés quand l'objet est droppé
// (ownership system). Mais pour un contrôle explicite (disconnect all on
// disable), ce SignalManager stocke les SignalHandlerId et les disconnect.
// ==============================================================================
use glib::signal::SignalHandlerId;
use glib::prelude::*;
use std::cell::RefCell;

/// SignalManager — tracke les connections signaux pour cleanup propre.
/// Traduit de la classe SignalManager (signalManager.ts).
pub struct SignalManager {
    connections: RefCell<Vec<(glib::Object, SignalHandlerId)>>,
}

impl SignalManager {
    pub fn new() -> Self {
        Self { connections: RefCell::new(vec![]) }
    }

    /// Enregistre une connection (à appeler après connect).
    pub fn track(&self, source: &impl IsA<glib::Object>, handler_id: SignalHandlerId) {
        self.connections.borrow_mut().push((source.as_ref().clone(), handler_id));
    }

    /// Disconnect tous les signaux enregistrés. Traduit de disconnectAll().
    pub fn disconnect_all(&self) {
        let mut conns = self.connections.borrow_mut();
        for (source, id) in conns.drain(..) {
            source.disconnect(id);
        }
    }
}

impl Drop for SignalManager {
    fn drop(&mut self) {
        self.disconnect_all();
    }
}
