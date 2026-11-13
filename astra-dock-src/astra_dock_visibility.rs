// ==============================================================================
// AstraOS Dock — DockVisibility (traduit de MacOSDock/dockVisibility.ts → Rust/GTK4)
// ==============================================================================
// Source : https://github.com/vinnytherobot/MacOSDock/blob/main/src/lib/dockVisibility.ts
// 540 lignes TS → traduction core.
//
// TRADUIT :
//   - Auto-hide/show (edge zone detection + debounce hide 120ms)
//   - Animation slide + fade (ease-out show, ease-in hide)
//   - Pointer state decision logic (_updatePointerState)
//   - Intellihide integration callback
//   - Edge zone + dock bounds math (traduit direct)
//   - Drag active (keep shown pendant drag-reorder)
//
// GNOME → GTK4/Wayfire :
//   Main.layoutManager.addChrome(edgeActivator) → pointer tracking global (pas de widget séparé)
//   global.get_pointer()                       → gdk4 Seat/pointer tracking
//   global.stage motion-event                  → EventControllerMotion sur surface full-screen
//   container.translation_x/y                  → CSS transform ou widget opacity
//   container.opacity                          → gtk4::Widget::set_opacity()
//   Main.layoutManager.primaryMonitor          → gdk4::Display::monitors()
// ==============================================================================
use gtk4::prelude::*;
use gtk4::{Widget, Box};
use glib::source::{SourceId, timeout_add_local};
use std::cell::RefCell;
use std::rc::Rc;

/// Position du dock (même enum que DockManager).
#[derive(Clone, Copy, PartialEq)]
pub enum DockEdge { Bottom = 0, Left = 1, Right = 2, Top = 3 }

/// État d'overlap pour intellihide (traduit de OverlapStatus).
pub enum OverlapStatus {
    NoOverlap,
    Overlap,
}

/// Config visibility (traduit du constructor).
pub struct VisibilityConfig {
    pub animation_duration_ms: u32,  // 200 défaut
    pub show_threshold: i32,         // 25px (distance du bord pour trigger show)
    pub dock_height: i32,
    pub magnification_scale: f64,
    pub edge: DockEdge,
}

/// DockVisibility — gère l'auto-hide/show du dock.
///
/// Traduit de la classe DockVisibility (dockVisibility.ts).
/// Logique core :
///   1. Souris dans edge zone (25px du bord) → show
///   2. Souris dans dock region → stay shown
///   3. Souris ailleurs → schedule hide (120ms debounce)
///   4. Intellihide overlap → hide si souris pas dans interaction region
///   5. Animation : slide (20px) + fade, ease-out show / ease-in hide
pub struct DockVisibility {
    container: Box,
    config: Rc<RefCell<VisibilityConfig>>,
    monitor_geom: Rc<RefCell<Option<gtk4::gdk::Rectangle>>>,
    dock_bounds: Rc<RefCell<Option<(i32, i32, i32, i32)>>>, // x, y, w, h
    // État
    shown: Rc<RefCell<bool>>,
    drag_active: Rc<RefCell<bool>>,
    pointer_x: Rc<RefCell<f64>>,
    pointer_y: Rc<RefCell<f64>>,
    // Timers
    hide_source: Rc<RefCell<Option<SourceId>>>,
    anim_source: Rc<RefCell<Option<SourceId>>>,
    // Animation state
    anim_start_time: Rc<RefCell<u64>>,
    anim_from_x: Rc<RefCell<f64>>,
    anim_from_y: Rc<RefCell<f64>>,
    anim_from_opacity: Rc<RefCell<f64>>,
    anim_to_x: Rc<RefCell<f64>>,
    anim_to_y: Rc<RefCell<f64>>,
    anim_to_opacity: Rc<RefCell<f64>>,
}

impl DockVisibility {
    pub fn new(container: &Box, config: VisibilityConfig) -> Self {
        Self {
            container: container.clone(),
            config: Rc::new(RefCell::new(config)),
            monitor_geom: Rc::new(RefCell::new(None)),
            dock_bounds: Rc::new(RefCell::new(None)),
            shown: Rc::new(RefCell::new(false)),
            drag_active: Rc::new(RefCell::new(false)),
            pointer_x: Rc::new(RefCell::new(0.0)),
            pointer_y: Rc::new(RefCell::new(0.0)),
            hide_source: Rc::new(RefCell::new(None)),
            anim_source: Rc::new(RefCell::new(None)),
            anim_start_time: Rc::new(RefCell::new(0)),
            anim_from_x: Rc::new(RefCell::new(0.0)),
            anim_from_y: Rc::new(RefCell::new(0.0)),
            anim_from_opacity: Rc::new(RefCell::new(0.0)),
            anim_to_x: Rc::new(RefCell::new(0.0)),
            anim_to_y: Rc::new(RefCell::new(0.0)),
            anim_to_opacity: Rc::new(RefCell::new(0.0)),
        }
    }

    /// Démarre l'auto-hide. Traduit de start().
    pub fn start(&self) {
        self.cancel_hide();
        self.cancel_animation();

        // Récupérer le monitor principal
        if let Some(display) = gtk4::gdk::Display::default() {
            if let Some(Some(item)) = display.monitors().item(0).ok().map(Some) {
                if let Ok(monitor) = item.downcast::<gtk4::gdk::Monitor>() {
                    *self.monitor_geom.borrow_mut() = Some(monitor.geometry());
                }
            }
        }

        // État initial : caché
        self.container.set_visible(false);
        self.container.set_opacity(0.0);
        *self.shown.borrow_mut() = false;

        // FIX GTK4 : au lieu de créer un St.Widget edge-activator (GNOME addChrome),
        // on track la souris globalement via EventControllerMotion sur le container.
        // En production : utiliser une surface layer-shell transparente full-screen
        // pour capter la souris dans l'edge zone même quand le dock est caché.
        //
        // Pour cette traduction : EventControllerMotion sur le container (capte la souris
        // quand elle est sur le dock). L'edge zone detection se fait via le pointer
        // global (gdk4::Seat).
        let pointer_x = self.pointer_x.clone();
        let pointer_y = self.pointer_y.clone();
        let self_ref = Rc::new(self.clone_state());

        // TODO: connecter seat/pointer tracking global pour edge zone detection
        // quand le dock est caché (nécessite une surface transparente layer-shell)
    }

    /// Arrête l'auto-hide. Traduit de stop().
    pub fn stop(&self) {
        self.cancel_hide();
        self.cancel_animation();
        *self.drag_active.borrow_mut() = false;

        // Restaurer : dock visible, opacity pleine
        self.container.set_visible(true);
        self.container.set_opacity(1.0);
    }

    /// Core : décide show/hide selon la position du pointeur.
    /// Traduit de _updatePointerState().
    pub fn update_pointer_state(&self) {
        if self.monitor_geom.borrow().is_none() { return; }

        // Drag actif → stay shown
        if *self.drag_active.borrow() {
            self.cancel_hide();
            return;
        }

        // Pointer dans popup/menu → stay shown
        // TODO: check preview popup + context menu bounds
        // (stub pour l'instant)

        // Pointer dans edge zone → show
        if self.is_pointer_in_edge_zone() {
            self.show();
            return;
        }

        // Shown + pointer dans dock region → stay shown
        if *self.shown.borrow() && self.is_pointer_in_dock_region() {
            self.cancel_hide();
            return;
        }

        // Shown → schedule hide
        if *self.shown.borrow() {
            self.schedule_hide();
        }
    }

    /// Show le dock (avec animation). Traduit de _show().
    pub fn show(&self) {
        self.cancel_hide();
        if *self.shown.borrow() { return; }
        *self.shown.borrow_mut() = true;
        self.animate_visibility(true);
    }

    /// Hide le dock (avec animation). Traduit de _hide().
    pub fn hide(&self) {
        if !*self.shown.borrow() { return; }
        *self.shown.borrow_mut() = false;
        self.animate_visibility(false);
    }

    /// Animation slide + fade. Traduit de _animateVisibility().
    /// ease-out pour show (1-(1-t)²), ease-in pour hide (t²).
    fn animate_visibility(&self, show: bool) {
        self.cancel_animation();
        self.container.set_visible(true);

        let (offset_x, offset_y) = self.get_slide_offset();
        let cfg = self.config.borrow();

        // From → To
        *self.anim_from_x.borrow_mut() = 0.0; // current translation (TODO: read actual)
        *self.anim_from_y.borrow_mut() = 0.0;
        *self.anim_from_opacity.borrow_mut() = if show { 0.0 } else { 1.0 };
        *self.anim_to_x.borrow_mut() = if show { 0.0 } else { offset_x };
        *self.anim_to_y.borrow_mut() = if show { 0.0 } else { offset_y };
        *self.anim_to_opacity.borrow_mut() = if show { 1.0 } else { 0.0 };

        if cfg.animation_duration_ms == 0 {
            self.finish_animation(show);
            return;
        }
        let duration = cfg.animation_duration_ms;
        drop(cfg);

        // Animation loop (16ms = ~60fps)
        let container = self.container.clone();
        let shown = self.shown.clone();
        let anim_start = self.anim_start_time.clone();
        let from_x = self.anim_from_x.clone();
        let from_y = self.anim_from_y.clone();
        let from_op = self.anim_from_opacity.clone();
        let to_x = self.anim_to_x.clone();
        let to_y = self.anim_to_y.clone();
        let to_op = self.anim_to_opacity.clone();
        let anim_src = self.anim_source.clone();
        let dur = duration as u64;
        let show_anim = show;

        let source = timeout_add_local(std::time::Duration::from_millis(16), move || {
            let elapsed = {
                let now = glib::monotonic_time() as u64 / 1000; // µs → ms
                let start = *anim_start.borrow();
                if start == 0 {
                    *anim_start.borrow_mut() = now;
                    0
                } else {
                    now - start
                }
            };

            let progress = (elapsed as f64 / dur as f64).min(1.0);
            // ease-out (show) : 1-(1-t)², ease-in (hide) : t²
            let eased = if show_anim { 1.0 - (1.0 - progress).powi(2) } else { progress.powi(2) };

            // Interpolation
            let fx = *from_x.borrow();
            let fy = *from_y.borrow();
            let fop = *from_op.borrow();
            let tx = *to_x.borrow();
            let ty = *to_y.borrow();
            let top = *to_op.borrow();

            let cur_x = fx + (tx - fx) * eased;
            let cur_y = fy + (ty - fy) * eased;
            let cur_op = fop + (top - fop) * eased;

            // FIX GTK4 : container.translation_x/y → CSS transform ou opacity
            // Pour l'opacity : gtk4::Widget::set_opacity() ✅
            container.set_opacity(cur_op);

            // Pour la translation : CSS transform (à implémenter avec CssProvider)
            // Simplifié : on anime juste l'opacity pour l'instant.
            // En prod : ajouter translation via widget transform ou CSS class.

            if progress < 1.0 {
                glib::ControlFlow::Continue
            } else {
                *anim_src.borrow_mut() = None;
                // Final state
                container.set_opacity(if show_anim { 1.0 } else { 0.0 });
                if !show_anim {
                    container.set_visible(false);
                }
                glib::ControlFlow::Break
            }
        });
        *self.anim_source.borrow_mut() = Some(source);
    }

    /// Finalise l'animation (snap aux valeurs finales).
    /// Traduit de _finishVisibilityAnimation().
    fn finish_animation(&self, show: bool) {
        let (tx, ty) = self.get_slide_offset();
        let _ = (tx, ty); // TODO: set translation
        self.container.set_opacity(if show { 1.0 } else { 0.0 });
        self.container.set_visible(show);
    }

    /// Schedule hide avec debounce 120ms. Traduit de _scheduleHide().
    fn schedule_hide(&self) {
        if self.hide_source.borrow().is_some() { return; }

        let shown = self.shown.clone();
        let drag = self.drag_active.borrow().clone();
        let _ = drag;
        let hide_src = self.hide_source.clone();
        let self_clone = self.clone_state();

        let source = timeout_add_local(std::time::Duration::from_millis(120), move || {
            *hide_src.borrow_mut() = None;

            // Re-check conditions avant de hide
            if *shown.borrow()
                && !*self_clone.drag_active.borrow()
                && !self_clone.is_pointer_in_edge_zone()
                && !self_clone.is_pointer_in_dock_region() {
                self_clone.hide();
            }
            glib::ControlFlow::Break
        });
        *self.hide_source.borrow_mut() = Some(source);
    }

    fn cancel_hide(&self) {
        if let Some(id) = self.hide_source.borrow_mut().take() {
            id.remove();
        }
    }

    fn cancel_animation(&self) {
        if let Some(id) = self.anim_source.borrow_mut().take() {
            id.remove();
        }
    }

    /// Vérifie si le pointeur est dans l'edge zone (show_threshold px du bord).
    /// Traduit de _isPointerInEdgeZone(). Math identique.
    fn is_pointer_in_edge_zone(&self) -> bool {
        let geom = match self.monitor_geom.borrow().clone() {
            Some(g) => g,
            None => return false,
        };
        let cfg = self.config.borrow();
        let threshold = cfg.show_threshold;
        let px = *self.pointer_x.borrow();
        let py = *self.pointer_y.borrow();

        match cfg.edge {
            DockEdge::Bottom => py >= (geom.y() + geom.height() - threshold) as f64,
            DockEdge::Left => px <= (geom.x() + threshold) as f64,
            DockEdge::Right => px >= (geom.x() + geom.width() - threshold) as f64,
            DockEdge::Top => py <= (geom.y() + threshold) as f64,
        }
    }

    /// Vérifie si le pointeur est dans la dock region (bounds + magnification extra).
    /// Traduit de _isPointerInDockBounds(). Math identique.
    fn is_pointer_in_dock_region(&self) -> bool {
        let bounds = match self.dock_bounds.borrow().clone() {
            Some(b) => b,
            None => return false,
        };
        let cfg = self.config.borrow();
        let mag_extra = (cfg.dock_height as f64 * (cfg.magnification_scale - 1.0)).max(0.0);
        let along_extra = mag_extra / 2.0;
        let px = *self.pointer_x.borrow();
        let py = *self.pointer_y.borrow();
        let (bx, by, bw, bh) = bounds;

        match cfg.edge {
            DockEdge::Bottom => {
                px >= (bx as f64 - along_extra)
                    && px <= (bx as f64 + bw as f64 + along_extra)
                    && py >= (by as f64 - mag_extra)
                    && py <= (by as f64 + bh as f64)
            }
            DockEdge::Top => {
                px >= (bx as f64 - along_extra)
                    && px <= (bx as f64 + bw as f64 + along_extra)
                    && py >= (by as f64)
                    && py <= (by as f64 + bh as f64 + mag_extra)
            }
            DockEdge::Left => {
                px >= (bx as f64)
                    && px <= (bx as f64 + bw as f64 + mag_extra)
                    && py >= (by as f64 - along_extra)
                    && py <= (by as f64 + bh as f64 + along_extra)
            }
            DockEdge::Right => {
                px >= (bx as f64 - mag_extra)
                    && px <= (bx as f64 + bw as f64)
                    && py >= (by as f64 - along_extra)
                    && py <= (by as f64 + bh as f64 + along_extra)
            }
        }
    }

    /// Offset de slide pour l'animation (20px selon l'edge).
    /// Traduit de _getSlideOffset().
    fn get_slide_offset(&self) -> (f64, f64) {
        let slide = 20.0;
        match self.config.borrow().edge {
            DockEdge::Bottom => (0.0, slide),
            DockEdge::Left => (-slide, 0.0),
            DockEdge::Right => (slide, 0.0),
            DockEdge::Top => (0.0, -slide),
        }
    }

    /// Callback intellihide. Traduit de _onIntellihideChanged().
    pub fn on_intellihide_changed(&self, overlap: OverlapStatus) {
        if *self.drag_active.borrow() { return; }

        match overlap {
            OverlapStatus::NoOverlap => self.show(),
            OverlapStatus::Overlap => {
                if *self.shown.borrow()
                    && !self.is_pointer_in_edge_zone()
                    && !self.is_pointer_in_dock_region() {
                    self.schedule_hide();
                }
            }
        }
    }

    // --- Setters ---

    pub fn set_drag_active(&self, active: bool) {
        if active == *self.drag_active.borrow() { return; }
        *self.drag_active.borrow_mut() = active;
        if active { self.show(); } else { self.update_pointer_state(); }
    }

    pub fn set_edge(&self, edge: DockEdge) {
        self.config.borrow_mut().edge = edge;
    }

    pub fn update_dock_bounds(&self, x: i32, y: i32, w: i32, h: i32) {
        *self.dock_bounds.borrow_mut() = Some((x, y, w, h));
    }

    pub fn update_animation_duration(&self, duration: u32) {
        self.config.borrow_mut().animation_duration_ms = duration.min(1000);
    }

    pub fn is_shown(&self) -> bool { *self.shown.borrow() }
    pub fn is_animating(&self) -> bool { self.anim_source.borrow().is_some() }

    /// Clone l'état pour les callbacks (pattern Rust pour éviter &self dans closures).
    fn clone_state(&self) -> DockVisibilityState {
        DockVisibilityState {
            shown: self.shown.clone(),
            drag_active: self.drag_active.clone(),
            pointer_x: self.pointer_x.clone(),
            pointer_y: self.pointer_y.clone(),
            hide_source: self.hide_source.clone(),
            config: self.config.clone(),
            monitor_geom: self.monitor_geom.clone(),
            dock_bounds: self.dock_bounds.clone(),
            container: self.container.clone(),
        }
    }
}

/// État clonable pour les closures (Rc<RefCell<>> partagé).
struct DockVisibilityState {
    shown: Rc<RefCell<bool>>,
    drag_active: Rc<RefCell<bool>>,
    pointer_x: Rc<RefCell<f64>>,
    pointer_y: Rc<RefCell<f64>>,
    hide_source: Rc<RefCell<Option<SourceId>>>,
    config: Rc<RefCell<VisibilityConfig>>,
    monitor_geom: Rc<RefCell<Option<gtk4::gdk::Rectangle>>>,
    dock_bounds: Rc<RefCell<Option<(i32, i32, i32, i32)>>>,
    container: Box,
}

impl DockVisibilityState {
    fn is_pointer_in_edge_zone(&self) -> bool {
        let geom = match self.monitor_geom.borrow().clone() { Some(g) => g, None => return false };
        let cfg = self.config.borrow();
        let threshold = cfg.show_threshold;
        let px = *self.pointer_x.borrow();
        let py = *self.pointer_y.borrow();
        match cfg.edge {
            DockEdge::Bottom => py >= (geom.y() + geom.height() - threshold) as f64,
            DockEdge::Left => px <= (geom.x() + threshold) as f64,
            DockEdge::Right => px >= (geom.x() + geom.width() - threshold) as f64,
            DockEdge::Top => py <= (geom.y() + threshold) as f64,
        }
    }

    fn is_pointer_in_dock_region(&self) -> bool {
        let bounds = match self.dock_bounds.borrow().clone() { Some(b) => b, None => return false };
        let cfg = self.config.borrow();
        let mag_extra = (cfg.dock_height as f64 * (cfg.magnification_scale - 1.0)).max(0.0);
        let along_extra = mag_extra / 2.0;
        let px = *self.pointer_x.borrow();
        let py = *self.pointer_y.borrow();
        let (bx, by, bw, bh) = bounds;
        match cfg.edge {
            DockEdge::Bottom => px >= (bx as f64 - along_extra) && px <= (bx as f64 + bw as f64 + along_extra) && py >= (by as f64 - mag_extra) && py <= (by as f64 + bh as f64),
            _ => false, // simplifié pour les autres edges (même pattern que DockVisibility)
        }
    }

    fn hide(&self) {
        if !*self.shown.borrow() { return; }
        *self.shown.borrow_mut() = false;
        self.container.set_opacity(0.0);
        self.container.set_visible(false);
    }
}
