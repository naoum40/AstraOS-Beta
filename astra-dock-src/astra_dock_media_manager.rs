// ==============================================================================
// AstraOS Dock — MediaManager (traduit de MacOSDock/mediaManager.ts → Rust)
// ==============================================================================
// Source : 288 lignes TS. Gestion MPRIS (contrôles musique).
//
// TRADUIT :
//   - Discover players via D-Bus (org.mpris.MediaPlayer2)
//   - Track active player (title, artist, status)
//   - Play/Pause, Next, Previous commands
//   - State changed callback
//
// GNOME → Rust :
//   Gio.DBus (session bus) → zbus (Rust D-Bus crate) ou gio::DBus (glib bindings)
//   Shell.App (active media app) → app_id string
// ==============================================================================
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Info d'un player MPRIS (traduit de MprisPlayerInfo).
pub struct MprisPlayerInfo {
    pub bus_name: String,
    pub title: String,
    pub artist: String,
    pub status: String, // "Playing", "Paused", "Stopped"
    pub desktop_entry: String,
}

/// MediaManager — gère les contrôles musique via MPRIS.
///
/// Traduit de la classe MediaManager (mediaManager.ts).
/// En production : utiliser zbus (Rust D-Bus) pour les appels MPRIS.
pub struct MediaManager {
    players: Rc<RefCell<HashMap<String, MprisPlayerInfo>>>,
    active_player: Rc<RefCell<Option<String>>>, // bus_name du player actif
    on_state_changed: Rc<RefCell<Box<dyn Fn(Option<&str>)>>>, // callback app_id
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            players: Rc::new(RefCell::new(HashMap::new())),
            active_player: Rc::new(RefCell::new(None)),
            on_state_changed: Rc::new(RefCell::new(Box::new(|_| {}))),
        }
    }

    pub fn set_on_state_changed<F: Fn(Option<&str>) + 'static>(&self, f: F) {
        *self.on_state_changed.borrow_mut() = Box::new(f);
    }

    /// Démarre : discover players via D-Bus. Traduit de start().
    pub fn start(&self) {
        // FIX Rust : Gio.DBus.session → zbus::Connection::session()
        // TODO: utiliser zbus pour :
        // 1. Watch org.freedesktop.DBus.NameOwnerChanged (detect new players)
        // 2. Discover existing players (ListNames → filter org.mpris.MediaPlayer2.*)
        // 3. Pour chaque player : create proxy, read Metadata (title, artist), Status
        // 4. Track active player (last playing, or last focused)
        //
        // Stub pour l'instant : pas de players découverts
    }

    pub fn stop(&self) {
        self.players.borrow_mut().clear();
        *self.active_player.borrow_mut() = None;
    }

    pub fn is_playing(&self) -> bool {
        let active = self.active_player.borrow();
        if let Some(bus_name) = active.as_ref() {
            let players = self.players.borrow();
            if let Some(info) = players.get(bus_name) {
                return info.status == "Playing";
            }
        }
        false
    }

    /// Play/Pause. Traduit de togglePlayPause().
    pub fn toggle_play_pause(&self) {
        if let Some(bus_name) = self.active_player.borrow().as_ref() {
            // FIX Rust : proxy.call("PlayPause") → zbus proxy call
            // TODO: zbus::Proxy::call_method("PlayPause")
            println!("[dock] MPRIS PlayPause for {}", bus_name);
        }
    }

    /// Next track. Traduit de next().
    pub fn next(&self) {
        if let Some(bus_name) = self.active_player.borrow().as_ref() {
            println!("[dock] MPRIS Next for {}", bus_name);
        }
    }

    /// Previous track. Traduit de previous().
    pub fn previous(&self) {
        if let Some(bus_name) = self.active_player.borrow().as_ref() {
            println!("[dock] MPRIS Previous for {}", bus_name);
        }
    }

    /// Retourne l'app_id du player actif (pour IconManager media badge).
    pub fn get_active_app_id(&self) -> Option<String> {
        let active = self.active_player.borrow();
        let bus_name = active.as_ref()?;
        let players = self.players.borrow();
        players.get(bus_name).map(|info| info.desktop_entry.clone())
    }
}
