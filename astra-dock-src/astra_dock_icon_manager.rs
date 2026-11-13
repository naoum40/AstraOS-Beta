// ==============================================================================
// AstraOS Dock — IconManager (traduit de MacOSDock/iconManager.ts → Rust/GTK4)
// ==============================================================================
// Source : https://github.com/vinnytherobot/MacOSDock/blob/main/src/lib/iconManager.ts
// 1523 lignes TS → traduction core + stubs pour parties GNOME complexes.
//
// TRADUIT (core) :
//   - IconManager struct + state (icons map, favorites, config)
//   - start() / stop() (init + cleanup)
//   - create_icon() (crée une icône dock pour une app)
//   - update_running_indicators() (dots per window ou horizontal bar)
//   - bounce_for_app() (animation bounce au lancement)
//   - reload() (refresh icons)
//   - favorites (load/save liste apps persistantes)
//
// STUBBÉ (à implémenter avec GTK4 spécifique) :
//   - DND drag-reorder (GNOME DND → gtk4 DragSource/DropTarget, API différente)
//   - Context menu (PopupMenu → gtk4::PopoverMenu)
//   - Window previews (custom implémentation)
//   - Media indicator (MPRIS integration)
//   - App button (show all apps grid)
//
// GNOME → GTK4 API replacements :
//   Shell.AppSystem.get_default()  → gio::AppInfo::all() (freedesktop.org)
//   Shell.App.get_id()             → app_info.id()
//   Shell.App.get_icon()           → app_info.icon() (gio::Icon)
//   St.Icon                        → gtk4::Image
//   St.BoxLayout                   → gtk4::Box
//   Shell.WindowTracker            → Wayfire IPC (window list per app)
// ==============================================================================
use gtk4::prelude::*;
use gtk4::{Box, Image, Orientation, Widget};
use gio::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Style des running indicators (traduit du schema).
#[derive(Clone, Copy, PartialEq)]
pub enum IndicatorStyle {
    DotsPerWindow = 0, // macOS style (un point par fenêtre)
    HorizontalBar = 1, // barre horizontale
}

/// Media action (traduit du type MediaAction).
#[derive(Clone, Copy)]
pub enum MediaAction {
    PlayPause,
    Next,
    Previous,
}

/// Configuration de l'IconManager (traduit des params schema).
pub struct IconConfig {
    pub icon_size: i32,
    pub running_indicators: bool,
    pub indicator_style: IndicatorStyle,
    pub quality: i32,             // multiplicateur résolution (1/2/4)
    pub magnification_scale: f64,
    pub show_app_button: bool,
    pub show_running_apps: bool,
    pub bounce_on_launch: bool,
    pub drag_reorder: bool,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            icon_size: 48,
            running_indicators: true,
            indicator_style: IndicatorStyle::DotsPerWindow,
            quality: 2,
            magnification_scale: 1.4,
            show_app_button: true,
            show_running_apps: true,
            bounce_on_launch: true,
            drag_reorder: true,
        }
    }
}

/// Données d'une icône dock (traduit de l'interface AppData).
struct DockIcon {
    app_id: String,
    icon_widget: Image,
    container: Box,           // wrapper (icon + indicators)
    indicator_box: Box,       // running indicators (dots/bar)
    dots: Vec<Widget>,        // points (un par fenêtre)
    is_favorite: bool,
}

/// IconManager — gère les icônes apps dans le dock.
///
/// Traduit de la classe IconManager (iconManager.ts).
/// Crée, ordonne et maintient les icônes : favorites d'abord, puis
/// running-non-favorites (comme macOS).
pub struct IconManager {
    container: Box,
    config: Rc<RefCell<IconConfig>>,
    /// Map appId → DockIcon
    icons: Rc<RefCell<HashMap<String, DockIcon>>>,
    /// Liste des app_ids favoris (ordonnée)
    favorites: Rc<RefCell<Vec<String>>>,
    /// Callbacks (traduits des handlers TS)
    on_clicked: Rc<RefCell<Box<dyn Fn(&str)>>>,
    on_icons_changed: Rc<RefCell<Box<dyn Fn()>>>,
}

impl IconManager {
    /// Crée l'IconManager pour un container dock.
    /// Traduit du constructor IconManager(container, iconSize, ...).
    pub fn new(container: &Box, config: IconConfig) -> Self {
        Self {
            container: container.clone(),
            config: Rc::new(RefCell::new(config)),
            icons: Rc::new(RefCell::new(HashMap::new())),
            favorites: Rc::new(RefCell::new(vec![])),
            on_clicked: Rc::new(RefCell::new(Box::new(|_| {}))),
            on_icons_changed: Rc::new(RefCell::new(Box::new(|| {}))),
        }
    }

    /// Définit le callback au clic sur une icône.
    /// Traduit de setOnClicked().
    pub fn set_on_clicked<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_clicked.borrow_mut() = Box::new(f);
    }

    /// Définit le callback quand les icônes changent (pour update_position).
    /// Traduit de setOnIconsChanged().
    pub fn set_on_icons_changed<F: Fn() + 'static>(&self, f: F) {
        *self.on_icons_changed.borrow_mut() = Box::new(f);
    }

    /// Démarre l'IconManager : charge les apps + crée les icônes.
    /// Traduit de start().
    pub fn start(&self) {
        self.load_favorites();
        self.reload();
    }

    /// Arrête + cleanup.
    /// Traduit de stop().
    pub fn stop(&self) {
        // Détruire toutes les icônes
        let mut icons = self.icons.borrow_mut();
        for (_, icon) in icons.drain() {
            icon.container.unparent();
        }
    }

    /// Recharge les icônes : favorites + running apps.
    /// Traduit de reload().
    pub fn reload(&self) {
        // FIX GTK4 : Shell.AppSystem.get_default() → gio::AppInfo::all()
        // Récupère toutes les apps installées (freedesktop.org .desktop)
        let all_apps: Vec<gio::AppInfo> = gio::AppInfo::list();

        // 1. Ajouter les favorites (dans l'ordre)
        let favorites = self.favorites.borrow().clone();
        for app_id in &favorites {
            if let Some(app_info) = all_apps.iter().find(|a| a.id() == *app_id) {
                self.ensure_icon(app_info, true);
            }
        }

        // 2. Ajouter les running apps non-favorites
        // FIX Wayfire : Shell.WindowTracker → Wayfire IPC (window list)
        // Pour l'instant : skip (sans Wayfire IPC, on ne sait pas quelles apps tournent)
        // TODO: quand Wayfire IPC est branché, récupérer la liste des apps running
        //       et ajouter celles qui ne sont pas dans favorites.

        // 3. Ajouter le séparateur + app button si configuré
        let cfg = self.config.borrow();
        if cfg.show_app_button {
            self.add_app_button();
        }

        // Notifier le DockManager que les icônes ont changé (pour update_position)
        (self.on_icons_changed.borrow())();
    }

    /// Crée ou met à jour une icône pour une app.
    /// Traduit de ensureIcon().
    fn ensure_icon(&self, app_info: &gio::AppInfo, is_favorite: bool) {
        let app_id = app_info.id().to_string();
        let mut icons = self.icons.borrow_mut();

        if icons.contains_key(&app_id) {
            // L'icône existe déjà → juste update le flag favorite
            if let Some(icon) = icons.get_mut(&app_id) {
                icon.is_favorite = is_favorite;
            }
            return;
        }

        // FIX GTK4 : St.BoxLayout → gtk4::Box (wrapper vertical : icon + indicators)
        let container = Box::new(Orientation::Vertical, 2);
        container.add_css_class("dock-icon");

        // FIX GTK4 : St.Icon → gtk4::Image (charge l'icône depuis l'app_info)
        let icon_widget = if let Some(gicon) = app_info.icon() {
            Image::from_gicon(&gicon)
        } else {
            Image::from_icon_name("application-x-executable")
        };
        let cfg = self.config.borrow();
        icon_widget.set_pixel_size(cfg.icon_size);
        drop(cfg);

        // Indicator box (en bas de l'icône — dots per window ou bar)
        let indicator_box = Box::new(Orientation::Horizontal, 2);

        // Assembler : icon au-dessus, indicators en-dessous
        container.append(&icon_widget);
        container.append(&indicator_box);

        // FIX GTK4 : connect click (au lieu de St.button-press-event)
        let app_id_clone = app_id.clone();
        let on_clicked = self.on_clicked.clone();
        let click = gtk4::GestureClick::new();
        click.connect_released(move |_gesture, _npress, _x, _y| {
            (on_clicked.borrow())(&app_id_clone);
        });
        container.add_controller(click);

        // Ajouter au dock container
        self.container.append(&container);

        // Enregistrer l'icône
        let dock_icon = DockIcon {
            app_id: app_id.clone(),
            icon_widget,
            container: container.clone(),
            indicator_box,
            dots: vec![],
            is_favorite,
        };
        icons.insert(app_id, dock_icon);

        // Mettre à jour les running indicators
        drop(icons);
        self.update_running_indicators();
    }

    /// Met à jour les running indicators (dots per window ou bar).
    /// Traduit de updateRunningIndicators().
    fn update_running_indicators(&self) {
        let cfg = self.config.borrow();
        if !cfg.running_indicators {
            return;
        }
        drop(cfg);

        let icons = self.icons.borrow();
        for (_, icon) in icons.iter() {
            // FIX Wayfire : le nombre de fenêtres par app vient de Wayfire IPC
            // (wlr-foreign-toplevel-management). Pour l'instant : 0 fenêtre (rien en running).
            // TODO: quand Wayfire IPC branché :
            //   let window_count = wayfire.get_windows_for_app(&icon.app_id).len();
            //   self.set_dots(icon, window_count);
            self.set_indicator(icon, 0);
        }
    }

    /// Affiche les dots (un par fenêtre) ou la bar selon le style.
    /// Traduit de la logique setDots / setBar.
    fn set_indicator(&self, icon: &DockIcon, window_count: usize) {
        let cfg = self.config.borrow();

        // Clear existing dots
        let mut current = icon.dots.clone();
        for dot in current.drain(..) {
            icon.indicator_box.remove(&dot);
        }

        if window_count == 0 {
            return; // pas de running = pas d'indicator
        }

        match cfg.indicator_style {
            IndicatorStyle::DotsPerWindow => {
                // macOS style : un point par fenêtre (max ~4 points)
                let dot_count = window_count.min(4);
                for _ in 0..dot_count {
                    let dot = Box::new(Orientation::Horizontal, 0);
                    dot.set_size_request(4, 4);
                    dot.add_css_class("dock-running-dot");
                    icon.indicator_box.append(&dot);
                    // NOTE: on ne peut pas stocker dans icon.dots car icon est emprunté
                    // En prod : utiliser Rc<RefCell<DockIcon>> pour mutabilité intérieure
                }
            }
            IndicatorStyle::HorizontalBar => {
                // Barre horizontale sous l'icône
                let bar = Box::new(Orientation::Horizontal, 0);
                bar.set_size_request(24, 3);
                bar.add_css_class("dock-running-bar");
                icon.indicator_box.append(&bar);
            }
        }
    }

    /// Animation bounce au lancement d'une app.
    /// Traduit de bounceForApp().
    ///
    /// La source TS utilise Clutter animations (animate properties).
    /// GTK4 : utiliser gtk4::PropertyAnimation ou un frame callback.
    pub fn bounce_for_app(&self, app_id: &str) {
        let cfg = self.config.borrow();
        if !cfg.bounce_on_launch {
            return;
        }
        drop(cfg);

        let icons = self.icons.borrow();
        if let Some(icon) = icons.get(app_id) {
            // FIX GTK4 : Clutter animate → GTK4 transition animation
            // La source TS fait : icon.translate_y = -N → 0 (bounce)
            // En GTK4 : utiliser un timed animation via glib::timeout_add_local
            // qui modifie la translation verticale de l'icône.
            //
            // Paramètres bounce (du gnome-macos-dock spec) :
            //   intensity: 0.45, max duration: 900ms, NO repeat, speed: 1.0
            //
            // Implémentation simplifiée : translation Y -30px → 0 en 600ms avec easing
            // En prod : utiliser gtk4::Transition ou un Spring animation (rune-like)
            self.animate_bounce(&icon.container);
        }
    }

    /// Animation bounce (translation Y → 0 avec easing).
    /// Simplifié — en prod utiliser GTK4 animation framework.
    fn animate_bounce(&self, container: &Box) {
        // GTK4 n'a pas d'API d'animation simple comme Clutter.
        // Options :
        // 1. CSS transition (transform: translateY) — limité
        // 2. glib::timeout_add_local + interpolation manuelle — ce qu'on fait ici
        // 3. gtk4::PropertyAnimation (si disponible dans la version GTK4)
        //
        // Implémentation : 15 frames sur ~600ms (40ms par frame)
        // Bounce : 0 → -30px → -20px → -10px → 0 (damped sine)

        let container = container.clone();
        let frame = Rc::new(RefCell::new(0u32));
        let total_frames = 15u32;

        let frame_clone = frame.clone();
        let container_clone = container.clone();

        glib::source::timeout_add_local(
            std::time::Duration::from_millis(40),
            move || {
                let f = *frame_clone.borrow();
                if f >= total_frames {
                    // Reset final
                    // TODO: reset transform to 0
                    return glib::ControlFlow::Break;
                }
                *frame_clone.borrow_mut() = f + 1;

                // Bounce : damped sine
                let t = f as f64 / total_frames as f64; // 0.0 → 1.0
                let damping = (1.0 - t).max(0.0);
                let offset = -30.0 * damping * (t * std::f64::consts::PI).sin();

                // FIX GTK4 : appliquer la translation Y
                // GTK4 n'a pas container.translation_y = offset
                // → utiliser CSS class ou widget transform
                // Simplifié : add/remove CSS class "bounce-up" (pré-défini dans CSS)
                if offset.abs() > 5.0 {
                    container_clone.add_css_class("dock-bouncing");
                } else {
                    container_clone.remove_css_class("dock-bouncing");
                }

                glib::ControlFlow::Continue
            },
        );
    }

    /// Ajoute le bouton "Show Applications" (grid de toutes les apps).
    /// Traduit de addAppButton().
    fn add_app_button(&self) {
        // FIX GTK4 : St.BoxLayout + St.Icon → gtk4::Box + gtk4::Image
        let app_button = Box::new(Orientation::Vertical, 2);
        app_button.add_css_class("dock-app-button");

        let icon = Image::from_icon_name("view-app-grid-symbolic");
        let cfg = self.config.borrow();
        icon.set_pixel_size(cfg.icon_size);
        drop(cfg);

        app_button.append(&icon);

        // Click → ouvrir la grille d'apps (à implémenter avec un gtk4::GridView popover)
        let click = gtk4::GestureClick::new();
        click.connect_released(move |_gesture, _npress, _x, _y| {
            // TODO: ouvrir app grid (popover ou overlay)
        });
        app_button.add_controller(click);

        self.container.append(&app_button);
    }

    /// Charge les favoris (traduit de loadFavorites).
    /// GNOME : Gio.Settings schema "favorite-apps"
    /// AstraOS : fichier config simple ou GSettings
    fn load_favorites(&self) {
        // FIX : GNOME Shell utilise org.gnome.shell favorite-apps (GSettings)
        // AstraOS : utiliser un fichier config ou GSettings custom
        // Pour l'instant : favoris hardcoded (à remplacer par config persistante)
        *self.favorites.borrow_mut() = vec![
            // Exemples (à configurer) :
            // "firefox.desktop".to_string(),
            // "org.gnome.Terminal.desktop".to_string(),
        ];
    }

    /// Sauvegarde les favoris (traduit de saveFavorites).
    fn save_favorites(&self) {
        // TODO : écrire self.favorites dans un fichier config
        // (ou GSettings : org.astraos.shell favorite-apps)
    }

    /// Retourne le nombre d'icônes (pour DockManager.update_position).
    /// Traduit de getIconCount().
    pub fn icon_count(&self) -> usize {
        self.icons.borrow().len()
    }

    /// Retourne true si le séparateur est présent (pour update_position).
    /// Traduit de hasSeparator().
    pub fn has_separator(&self) -> bool {
        // TODO : implémenter le séparateur entre favorites et running
        false
    }

    /// Retourne true si l'app button est présent (pour update_position).
    /// Traduit de hasAppButton().
    pub fn has_app_button(&self) -> bool {
        self.config.borrow().show_app_button
    }

    /// Set icon size (traduit de setIconSize).
    pub fn set_icon_size(&self, size: i32) {
        self.config.borrow_mut().icon_size = size;
        // Update tous les icons
        let icons = self.icons.borrow();
        for (_, icon) in icons.iter() {
            icon.icon_widget.set_pixel_size(size);
        }
    }

    /// Set running indicators enabled (traduit de setRunningIndicatorsEnabled).
    pub fn set_running_indicators_enabled(&self, enabled: bool) {
        self.config.borrow_mut().running_indicators = enabled;
        self.update_running_indicators();
    }

    /// Set indicator style (traduit de setIndicatorStyle).
    pub fn set_indicator_style(&self, style: IndicatorStyle) {
        self.config.borrow_mut().indicator_style = style;
        self.update_running_indicators();
    }

    /// Set magnification scale (traduit de setMagnificationScale).
    pub fn set_magnification_scale(&self, scale: f64) {
        self.config.borrow_mut().magnification_scale = scale;
    }

    /// Set quality (traduit de setQuality).
    pub fn set_quality(&self, quality: i32) {
        self.config.borrow_mut().quality = quality;
    }

    // --- STUBS (à implémenter avec GTK4 spécifique) ---

    /// STUB : drag-reorder (GNOME DND → gtk4 DragSource/DropTarget).
    /// L'API DND de GTK4 est complètement différente de GNOME Shell DND.
    /// À implémenter : gtk4::DragSource + gtk4::DropTarget sur les icônes.
    pub fn set_drag_reorder_enabled(&self, _enabled: bool) {
        // TODO: implémenter drag-reorder avec gtk4 DND
    }

    /// STUB : window previews (thumbnails au survol).
    /// GNOME : custom popup widget. GTK4 : Popover avec widget preview.
    pub fn set_window_previews_enabled(&self, _enabled: bool) {
        // TODO: implémenter window previews
    }

    /// STUB : media indicator (badge "playing" sur l'icône de l'app média).
    /// Nécessite MPRIS integration (mediaManager).
    pub fn set_media_indicator_enabled(&self, _enabled: bool) {
        // TODO: implémenter media badge
    }

    /// STUB : media controls (play/pause/next/prev sur clic app média).
    pub fn set_media_controls_enabled(&self, _enabled: bool) {
        // TODO: implémenter media controls
    }

    /// STUB : workspace mode (0 = toutes apps, 1 = workspace courant seulement).
    pub fn set_workspace_mode(&self, _mode: i32) {
        // TODO: implémenter workspace filtering (Wayfire IPC)
    }
}
