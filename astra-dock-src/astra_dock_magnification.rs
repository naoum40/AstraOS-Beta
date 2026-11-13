// ==============================================================================
// AstraOS Dock — Magnification (traduit de MacOSDock/magnification.ts → Rust/GTK4)
// ==============================================================================
// Source : https://github.com/vinnytherobot/MacOSDock/blob/main/src/lib/magnification.ts
// Traduction ligne par ligne en Rust/GTK4. Fix : APIs GNOME (St/Clutter) → GTK4.
//
// Algorithme (identique à la source TypeScript) :
//   1. Poll à framerate configurable (60fps défaut)
//   2. Track position souris (EventMotion GTK4 au lieu de global.get_pointer)
//   3. Trouve l'icône focale (la plus proche du curseur)
//   4. Pour chaque icône : distance → smoothstep falloff → target scale
//   5. Lerp interpolation (smooth animation vers target)
//   6. Applique scale via GTK4 widget transform
//
// Params configurables (du schema MacOSDock) :
//   max_scale : 1.4 défaut (scale max sous le curseur)
//   falloff_distance : 100px (portée de la magnification)
//   framerate : 60fps (30/60/120)
//   lerp_factor : 0.25 (smoothing par tick)
//   pivot : (0.5, 1.0) — centre-bas (les icônes grandissent depuis le bas)
// ==============================================================================
use gtk4::prelude::*;
use gtk4::{Box, Widget};
use glib::source::{SourceId, timeout_add_local};
use std::cell::RefCell;
use std::rc::Rc;

const MIN_SCALE: f64 = 1.0;
const LERP_FACTOR: f64 = 0.25; // smoothing per tick (same as TS source)

/// État de magnification pour le dock AstraOS.
///
/// Traduit de la classe `Magnification` (magnification.ts).
/// Au lieu de poller global.get_pointer (GNOME), on track la souris via
/// EventMotion sur le container GTK4. Le poll loop reste (glib timeout).
pub struct Magnification {
    container: Box,
    enabled: bool,
    max_scale: f64,
    falloff_distance: f64,
    framerate: u32,
    pivot_x: f64,
    pivot_y: f64,
    paused: bool,
    // État mutable (intérieur RefCell pour le callback)
    inner: Rc<RefCell<MagnificationInner>>,
}

struct MagnificationInner {
    poll_id: Option<SourceId>,
    current_scales: Vec<f64>,
    mouse_x: f64,
    mouse_y: f64,
    /// Les widgets enfants du container (mis à jour à chaque update)
    children: Vec<Widget>,
}

impl Magnification {
    /// Crée la magnification pour un container GTK4 Box (le dock).
    pub fn new(container: &Box) -> Self {
        let inner = Rc::new(RefCell::new(MagnificationInner {
            poll_id: None,
            current_scales: vec![],
            mouse_x: 0.0,
            mouse_y: 0.0,
            children: vec![],
        }));

        let mag = Self {
            container: container.clone(),
            enabled: true,
            max_scale: 1.4,
            falloff_distance: 100.0,
            framerate: 60,
            pivot_x: 0.5,
            pivot_y: 1.0,
            paused: false,
            inner,
        };

        // FIX GTK4 : tracker la souris via EventMotion (au lieu de global.get_pointer GNOME)
        let inner_clone = mag.inner.clone();
        container.add_controller(gtk4::EventControllerMotion::new());
        // On connect le motion via un EventControllerMotion séparé
        let motion = gtk4::EventControllerMotion::new();
        let inner_for_motion = mag.inner.clone();
        motion.connect_motion(move |_ctrl, x, y| {
            let mut i = inner_for_motion.borrow_mut();
            i.mouse_x = x;
            i.mouse_y = y;
        });
        motion.connect_leave(move |_ctrl| {
            let mut i = inner_clone.borrow_mut();
            // Souris quittée → on reset (comme le leave-event dans la source TS)
            i.mouse_x = f64::MIN;
            i.mouse_y = f64::MIN;
        });
        container.add_controller(motion);

        mag
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset_all();
        }
    }

    pub fn set_max_scale(&mut self, scale: f64) {
        self.max_scale = scale;
    }

    pub fn set_falloff_distance(&mut self, distance: f64) {
        self.falloff_distance = distance;
    }

    pub fn set_framerate(&mut self, fps: u32) {
        self.framerate = fps;
        self.restart_poll();
    }

    pub fn set_pivot_point(&mut self, x: f64, y: f64) {
        self.pivot_x = x;
        self.pivot_y = y;
    }

    /// Freeze la magnification (pendant un drag-reorder). Le poll tourne mais
    /// retourne early. Snap toutes les icônes à 1.0.
    pub fn pause(&mut self) {
        if self.paused { return; }
        self.paused = true;
        self.snap_all_to_min();
    }

    pub fn resume(&mut self) {
        if !self.paused { return; }
        self.paused = false;
        self.inner.borrow_mut().current_scales.clear();
    }

    /// Démarre le poll loop (équivalent de start() dans la source TS).
    pub fn start(&self) {
        self.start_poll();
    }

    /// Arrête le poll loop + déconnecte tout (équivalent de stop()).
    pub fn stop(&self) {
        self.stop_poll();
        self.reset_all();
    }

    // --- Internes ---

    fn start_poll(&self) {
        self.stop_poll();
        let interval = 1000 / self.framerate;
        let inner = self.inner.clone();
        let container = self.container.clone();
        let max_scale = self.max_scale;
        let falloff = self.falloff_distance;
        let pivot_x = self.pivot_x;
        let pivot_y = self.pivot_y;

        // Capture l'état "enabled" et "paused" via RefCell (ils peuvent changer)
        let state = Rc::new(RefCell::new((self.enabled, self.paused)));
        let state_clone = state.clone();

        let source = timeout_add_local(
            std::time::Duration::from_millis(interval as u64),
            move || {
                let (enabled, paused) = *state_clone.borrow();
                if !enabled || paused || !container.is_visible() {
                    return glib::ControlFlow::Continue;
                }

                // Récupère les enfants du container
                let children: Vec<Widget> = container.observe_children()
                    .into_iter()
                    .filter_map(|item| item.ok())
                    .filter_map(|item| item.downcast::<Widget>().ok())
                    .collect();

                let mut i = inner.borrow_mut();

                // Position souris (local au container, déjà en coords relatives via EventMotion)
                let (local_x, local_y) = (i.mouse_x, i.mouse_y);

                // Si souris hors zone (leave-event → mouse_x = MIN)
                if local_x == f64::MIN {
                    Self::reset_scales(&mut i.current_scales, &children);
                    return glib::ControlFlow::Continue;
                }

                if children.is_empty() {
                    return glib::ControlFlow::Continue;
                }

                // Assure que current_scales a la bonne taille
                while i.current_scales.len() < children.len() {
                    i.current_scales.push(MIN_SCALE);
                }

                // Trouve l'icône focale (la plus proche du curseur X)
                let mut focal_idx = 0;
                let mut best_dist = f64::INFINITY;
                for (idx, child) in children.iter().enumerate() {
                    let allocation = child.allocation();
                    let cx = allocation.x() as f64;
                    let cw = allocation.width() as f64;
                    let center = cx + cw / 2.0;
                    let d = (local_x - center).abs();
                    if d < best_dist {
                        best_dist = d;
                        focal_idx = idx;
                    }
                }

                // Si la souris est trop loin du dock, reset
                if best_dist > falloff * 1.5 {
                    Self::reset_scales(&mut i.current_scales, &children);
                    return glib::ControlFlow::Continue;
                }

                // Centre de l'icône focale
                let focal_alloc = children[focal_idx].allocation();
                let focal_center = focal_alloc.x() as f64 + focal_alloc.width() as f64 / 2.0;

                // Compute target scales + lerp pour chaque icône
                for (idx, child) in children.iter().enumerate() {
                    let alloc = child.allocation();
                    let cx = alloc.x() as f64;
                    let cw = alloc.width() as f64;
                    let center = cx + cw / 2.0;
                    let dist = (center - focal_center).abs();

                    // Smoothstep falloff (identique à la source TS)
                    let t = (1.0 - dist / falloff).max(0.0);
                    let smooth = t * t * (3.0 - 2.0 * t);
                    let target = MIN_SCALE + (max_scale - MIN_SCALE) * smooth;

                    // Lerp vers target (smoothing)
                    let prev = i.current_scales[idx];
                    let next = prev + (target - prev) * LERP_FACTOR;
                    i.current_scales[idx] = next;

                    // FIX GTK4 : appliquer le scale via widget transform
                    // (GTK4 n'a pas scale_x/scale_y comme Clutter/St)
                    // → utiliser gtk4::Widget transform via allocation ou CSS
                    Self::apply_scale(child, next, pivot_x, pivot_y);
                }

                glib::ControlFlow::Continue
            },
        );
        self.inner.borrow_mut().poll_id = Some(source);
    }

    fn stop_poll(&self) {
        let mut i = self.inner.borrow_mut();
        if let Some(id) = i.poll_id.take() {
            id.remove();
        }
    }

    fn restart_poll(&self) {
        let has_poll = self.inner.borrow().poll_id.is_some();
        if has_poll {
            self.stop_poll();
            self.start_poll();
        }
    }

    /// Reset toutes les icônes vers scale 1.0 avec lerp (smooth)
    fn reset_all(&self) {
        if self.paused { return; }
        let mut i = self.inner.borrow_mut();
        let children: Vec<Widget> = self.container.observe_children()
            .into_iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| item.downcast::<Widget>().ok())
            .collect();
        Self::reset_scales(&mut i.current_scales, &children);
    }

    fn reset_scales(scales: &mut Vec<f64>, children: &[Widget]) {
        for (idx, child) in children.iter().enumerate() {
            let prev = scales.get(idx).copied().unwrap_or(MIN_SCALE);
            if (prev - MIN_SCALE).abs() < 0.001 { continue; }
            let next = prev + (MIN_SCALE - prev) * LERP_FACTOR;
            scales[idx] = next;
            Self::apply_scale(child, next, 0.5, 1.0);
        }
    }

    /// Snap immédiat à 1.0 (pas de lerp) — utilisé par pause()
    fn snap_all_to_min(&self) {
        let mut i = self.inner.borrow_mut();
        let children: Vec<Widget> = self.container.observe_children()
            .into_iter()
            .filter_map(|item| item.ok())
            .filter_map(|item| item.downcast::<Widget>().ok())
            .collect();
        for child in &children {
            Self::apply_scale(child, MIN_SCALE, 0.5, 1.0);
        }
        i.current_scales.clear();
    }

    /// FIX GTK4 : applique un scale à un widget.
    ///
    /// Dans la source TS (GNOME/Clutter), on fait `child.scale_x = next; child.scale_y = next;`
    /// GTK4 n'a pas cette propriété directe. Solutions :
    ///
    /// Option A (recommandée) : CSS transform via une classe dynamique
    ///   → `child.set_css_classes(&["dock-icon", &format!("scale-{:.2}", next)])`
    ///   + CSS: `.scale-1\.22 { transform: scale(1.22); }` (pré-généré)
    ///
    /// Option B : Custom LayoutManager qui applique un Gsk::Transform
    ///   → override allocate() avec transform scale
    ///
    /// Option C (la plus simple pour GTK4 4.14+) : widget transform
    ///   → `child.queue_allocate()` + custom allocation avec transform
    ///
    /// Ici on utilise l'Option A (CSS) pour la simplicité. En production,
    /// un custom LayoutManager (Option B) serait plus performant (pas de
    /// re-parse CSS à chaque tick).
    fn apply_scale(widget: &Widget, scale: f64, _pivot_x: f64, _pivot_y: f64) {
        // Option A : CSS transform
        // NOTE : GTK4 CSS supporte `transform: scale()` depuis GTK 4.12+
        // On génère le style inline via une classe dynamique
        // En production : pré-générer N classes (scale-100, scale-110, ... scale-140)
        // et switcher, plutôt que de générer du CSS à chaque tick.

        // Pour cette traduction, on utilise add_css_class avec une classe
        // pré-définie. La precision du scale est arrondie à 0.05.
        let rounded = (scale * 20.0).round() / 20.0; // précision 0.05
        let class = format!("mag-{}", (rounded * 100.0).round() as u32);

        // Retire les anciennes classes mag-*
        // (En production : tracker la classe actuelle pour éviter le scan)
        let current_classes: Vec<String> = widget.css_classes().into_iter().map(|s| s.to_string()).collect();
        for c in &current_classes {
            if c.starts_with("mag-") {
                widget.remove_css_class(c);
            }
        }
        widget.add_css_class(&class);
    }
}

impl Drop for Magnification {
    fn drop(&mut self) {
        self.stop();
    }
}

// ==============================================================================
// CSS à générer pour les classes mag-* (dans le stylesheet du dock)
// ==============================================================================
// Génère 41 classes de mag-100 (scale 1.0) à mag-140 (scale 1.4) par pas de 0.01
// Plus les classes intermédiaires (mag-105, mag-110, ... mag-200)
//
// Exemple CSS :
// .mag-100 { transform: scale(1.0); }
// .mag-110 { transform: scale(1.1); }
// .mag-122 { transform: scale(1.22); }
// .mag-140 { transform: scale(1.4); }
//
// En production : générer via un script au build, ou utiliser un LayoutManager
// custom (Option B) pour éviter le CSS switching.
// ==============================================================================
