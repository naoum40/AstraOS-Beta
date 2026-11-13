// ==============================================================================
// AstraOS Dock — Intellihide (traduit de MacOSDock/intellihide.ts → Rust/GTK4)
// ==============================================================================
// Source : https://github.com/vinnytherobot/MacOSDock/blob/main/src/lib/intellihide.ts
// 79 lignes TS → traduction directe (logique simple).
//
// Logique : si une fenêtre maximisée/fullscreen chevauche le dock → hide.
// Sinon → show. Callback appelé seulement quand le status change.
//
// GNOME → Wayfire :
//   global.get_window_actors()           → Wayfire IPC (wlr-foreign-toplevel)
//   global.workspace_manager             → Wayfire IPC (active workspace)
//   metaWin.maximized_vertically         → Wayfire window state
//   metaWin.get_frame_rect()             → Wayfire window geometry
//   global.display "restacked"           → Wayfire IPC event
//   Shell.WindowTracker "focus-app"      → Wayfire IPC focus event
// ==============================================================================
use std::cell::RefCell;
use std::rc::Rc;

/// Status d'overlap (traduit de OverlapStatus = boolean).
/// true = une fenêtre chevauche le dock (hide), false = pas de chevauchement (show).
pub type OverlapStatus = bool;

/// Rectangle du dock (traduit de _dockRect).
#[derive(Clone, Copy, Default)]
pub struct DockRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Représentation d'une fenêtre pour le check overlap (traduit de MetaWindow).
pub struct WindowInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub maximized_vertically: bool,
    pub fullscreen: bool,
    pub on_active_workspace: bool,
}

/// Intellihide — cache le dock quand une fenêtre maximisée/fullscreen le chevauche.
///
/// Traduit de la classe Intellihide (intellihide.ts).
/// Logique identique :
///   1. Récupère les fenêtres du workspace actif
///   2. Pour chaque fenêtre maximisée/fullscreen : check overlap avec dock rect
///   3. Si overlap → status = true (hide). Sinon → false (show).
///   4. Callback appelé seulement si le status change.
pub struct Intellihide {
    status: Rc<RefCell<OverlapStatus>>,
    dock_rect: Rc<RefCell<Option<DockRect>>>,
    on_status_changed: Rc<RefCell<Box<dyn Fn(OverlapStatus)>>>,
}

impl Intellihide {
    pub fn new() -> Self {
        Self {
            status: Rc::new(RefCell::new(false)),
            dock_rect: Rc::new(RefCell::new(None)),
            on_status_changed: Rc::new(RefCell::new(Box::new(|_| {}))),
        }
    }

    /// Démarre l'intellihide. Traduit de start().
    /// En production : connecter aux events Wayfire IPC :
    ///   - window created / destroyed / restacked
    ///   - focus changed
    ///   - workspace switched
    /// Chaque event → check_overlap()
    pub fn start<F: Fn(OverlapStatus) + 'static>(&self, callback: F) {
        *self.on_status_changed.borrow_mut() = Box::new(callback);

        // FIX Wayfire : au lieu de GNOME signals (global.display restacked, etc.),
        // subscribe aux events Wayfire IPC.
        // TODO: quand Wayfire IPC est branché :
        //   wayfire.on_window_created(|| self.check_overlap());
        //   wayfire.on_window_destroyed(|| self.check_overlap());
        //   wayfire.on_focus_changed(|| self.check_overlap());
        //   wayfire.on_workspace_switched(|| self.check_overlap());

        self.check_overlap();
    }

    /// Définit le rectangle du dock. Traduit de setDockRect().
    pub fn set_dock_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        *self.dock_rect.borrow_mut() = Some(DockRect { x, y, width, height });
        self.check_overlap();
    }

    /// Arrête. Traduit de stop().
    pub fn stop(&self) {
        // TODO: déconnecter les events Wayfire IPC
        *self.on_status_changed.borrow_mut() = Box::new(|_| {});
    }

    /// Check si une fenêtre maximisée/fullscreen chevauche le dock.
    /// Traduit de _checkOverlap(). Math identique (rectangle intersection).
    pub fn check_overlap(&self) {
        let dock_rect = match *self.dock_rect.borrow() {
            Some(r) => r,
            None => return,
        };

        let dock_x1 = dock_rect.x;
        let dock_y1 = dock_rect.y;
        let dock_x2 = dock_rect.x + dock_rect.width;
        let dock_y2 = dock_rect.y + dock_rect.height;

        // FIX Wayfire : récupérer les fenêtres via IPC.
        // Pour l'instant : stub (pas de fenêtres → pas d'overlap → dock toujours visible).
        // TODO: let windows = wayfire.get_windows_on_active_workspace();
        let windows: Vec<WindowInfo> = vec![]; // STUB — à remplir via Wayfire IPC

        let mut overlaps = false;
        for win in &windows {
            // Skip : pas sur le workspace actif
            if !win.on_active_workspace { continue; }

            // Skip : pas maximisé ET pas fullscreen
            if !win.maximized_vertically && !win.fullscreen { continue; }

            // Check overlap (rectangle intersection — math pure, identique à la source TS)
            let win_x1 = win.x;
            let win_y1 = win.y;
            let win_x2 = win.x + win.width;
            let win_y2 = win.y + win.height;

            if win_x1 < dock_x2 && win_x2 > dock_x1 && win_y1 < dock_y2 && win_y2 > dock_y1 {
                overlaps = true;
                break;
            }
        }

        // Callback seulement si le status change (traduit de la fin de _checkOverlap)
        let mut status = self.status.borrow_mut();
        if *status != overlaps {
            *status = overlaps;
            drop(status); // release borrow avant d'appeler le callback
            (self.on_status_changed.borrow())(overlaps);
        }
    }
}
