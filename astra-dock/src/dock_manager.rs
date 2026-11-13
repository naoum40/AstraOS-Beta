// ==============================================================================
// AstraOS Dock — DockManager (traduit de MacOSDock/dockManager.ts → Rust/GTK4)
// ==============================================================================
// Source : https://github.com/vinnytherobot/MacOSDock/blob/main/src/lib/dockManager.ts
//
// GNOME → GTK4/Wayfire API replacements :
//   Main.layoutManager.addTopChrome()  → gtk4_layer_shell (position au bord écran)
//   Main.layoutManager.primaryMonitor  → gdk4::Display::monitors()
//   Shell.WindowTracker                → Wayfire IPC (window tracking)
//   Shell.AppSystem                    → gio::AppInfo (freedesktop.org standard)
//   Main.wm.addKeybinding              → Wayfire config keybinds
//   Shell.BlurEffect                   → Wayfire blur plugin (natif, pas de code)
//   St.BoxLayout                       → gtk4::Box
//   global.workspace_manager           → Wayfire IPC (workspaces)
//   Main.overview.dash                 → N/A (AstraOS n'a pas de dash GNOME)
// ==============================================================================
use gtk4::prelude::*;
use gtk4::{Box, Orientation, Widget};
use gtk4_layer_shell::{Layer, LayerShell, Anchor};
use gio::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Position du dock (traduit de POSITIONS const).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DockPosition {
    Bottom = 0,
    Left = 1,
    Right = 2,
    Top = 3,
}

/// Constantes (traduites de DockManager static readonly).
const MARGIN_EDGE: i32 = 12;       // MARGIN_BOTTOM dans la source
const MIN_DOCK_WIDTH: i32 = 300;
const LAUNCH_DEBOUNCE_MS: u64 = 400;

/// Configuration du dock (traduit du schema GSettings).
pub struct DockConfig {
    pub icon_size: i32,            // 48 défaut
    pub dock_opacity: f64,         // 0.0-1.0 (60% défaut = 0.6)
    pub background_color: (u8, u8, u8), // RGB (#1e1e1e défaut)
    pub border_radius: i32,        // 16px défaut
    pub blur_enabled: bool,        // true (Wayfire blur natif)
    pub position: DockPosition,
    pub auto_hide: bool,
    pub magnification_enabled: bool,
    pub magnification_scale: f64,  // 1.4 défaut
    pub magnification_falloff: i32,// 100px
    pub magnification_framerate: u32, // 60
    pub bounce_on_launch: bool,
    pub animation_duration_ms: u32,// 200
    pub show_threshold: i32,       // 25px (auto-hide show distance)
    pub running_indicators: bool,
    pub show_running_apps: bool,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            icon_size: 48,
            dock_opacity: 0.6,
            background_color: (0x1e, 0x1e, 0x1e),
            border_radius: 16,
            blur_enabled: true, // Wayfire blur natif
            position: DockPosition::Bottom,
            auto_hide: true,
            magnification_enabled: true,
            magnification_scale: 1.4,
            magnification_falloff: 100,
            magnification_framerate: 60,
            bounce_on_launch: true,
            animation_duration_ms: 200,
            show_threshold: 25,
            running_indicators: true,
            show_running_apps: true,
        }
    }
}

/// DockManager — orchestrateur du dock AstraOS.
///
/// Traduit de la classe DockManager (dockManager.ts).
/// Crée le container GTK4, le positionne via layer-shell, gère le style CSS,
/// coordonne magnification + icons + visibility + intellihide.
pub struct DockManager {
    container: Box,
    config: Rc<RefCell<DockConfig>>,
    // Sous-modules (seront implémentés dans leurs propres fichiers)
    // magnification: Magnification,      // déjà traduit (astra_dock_magnification.rs)
    // icon_manager: IconManager,         // à traduire (iconManager.ts)
    // visibility: DockVisibility,        // à traduire (dockVisibility.ts)
    // intellihide: Intellihide,          // à traduire (intellihide.ts)
    icon_count: Rc<RefCell<usize>>,
}

impl DockManager {
    /// Crée le dock. Équivalent de enable() dans la source TS.
    pub fn new(config: DockConfig) -> Self {
        let config = Rc::new(RefCell::new(config));

        // FIX GTK4 : St.BoxLayout → gtk4::Box (horizontal par défaut pour dock bottom)
        let container = Box::new(Orientation::Horizontal, 6);
        container.add_css_class("astra-dock-container");

        // FIX GTK4 : Main.layoutManager.addTopChrome() → gtk4_layer_shell
        // Le dock est positionné au-dessus des fenêtres normales (overlay layer)
        LayerShell::init(&container);
        LayerShell::set_layer(&container, Layer::Overlay);
        LayerShell::set_keyboard_mode(&container, gtk4_layer_shell::KeyboardMode::OnDemand);

        let mgr = Self {
            container: container.clone(),
            config: config.clone(),
            icon_count: Rc::new(RefCell::new(0)),
        };

        // Positionner le dock (anchor + margin)
        mgr.apply_dock_position();
        // Appliquer le style (CSS inline)
        mgr.apply_style();
        // Calculer la taille initiale
        mgr.update_position();

        mgr
    }

    /// Équivalent de disable() — cleanup.
    pub fn destroy(&self) {
        // Les sous-modules (magnification, icon_manager, etc.) se cleanup
        // via leur propre Drop. Ici on détruit juste le container.
        // self.container sera détruit par GTK quand retiré du parent.
    }

    /// Référence au container GTK4 (pour icon_manager, magnification, etc.)
    pub fn container(&self) -> &Box {
        &self.container
    }

    // --- POSITIONING (traduit de _updatePosition) ---

    /// Calcule et applique la position + taille du dock.
    /// Traduit de _updatePosition() — la logique math est identique.
    pub fn update_position(&self) {
        let cfg = self.config.borrow();
        let icon_count = *self.icon_count.borrow();

        // Calcul de la taille du dock (traduit direct de la source TS)
        let icon_padded = cfg.icon_size + 12; // icon + padding
        let spacing = 6;
        let padding = 20;
        // separator + app button omis pour simplifier (à ajouter quand icon_manager traduit)
        let content_size = if icon_count > 0 {
            icon_count * icon_padded + (icon_count - 1) * spacing + padding
        } else {
            padding
        };
        let dock_axis_size = std::cmp::max(content_size, MIN_DOCK_WIDTH);

        let dock_height = cfg.icon_size + 16 + 8; // iconActorHeight + verticalPadding

        // FIX GTK4 : Main.layoutManager.primaryMonitor → gdk4 monitor
        // Pour la démo, on utilise l'écran principal. En prod : gdk4::Display::monitors()
        let display = gtk4::gdk::Display::default();
        let monitors = display.map(|d| d.monitors());
        let primary_monitor = monitors
            .as_ref()
            .and_then(|m| m.item(0))
            .and_then(|item| item.ok())
            .and_then(|item| item.downcast::<gtk4::gdk::Monitor>().ok());

        if primary_monitor.is_none() {
            return; // pas de monitor (headless ?)
        }
        let monitor = primary_monitor.unwrap();
        let monitor_geom = monitor.geometry();

        // Position selon l'edge (traduit du switch POSITIONS)
        match cfg.position {
            DockPosition::Bottom => {
                // Dock horizontal, centré, collé au bas
                LayerShell::set_anchor(&self.container, Anchor::Bottom);
                LayerShell::set_margin(&self.container, gtk4_layer_shell::Edge::Bottom, MARGIN_EDGE);
                self.container.set_size_request(dock_axis_size, dock_height);
            }
            DockPosition::Top => {
                LayerShell::set_anchor(&self.container, Anchor::Top);
                LayerShell::set_margin(&self.container, gtk4_layer_shell::Edge::Top, MARGIN_EDGE);
                self.container.set_size_request(dock_axis_size, dock_height);
            }
            DockPosition::Left => {
                // Dock vertical
                self.container.set_orientation(Orientation::Vertical);
                LayerShell::set_anchor(&self.container, Anchor::Left);
                LayerShell::set_margin(&self.container, gtk4_layer_shell::Edge::Left, MARGIN_EDGE);
                self.container.set_size_request(dock_height, dock_axis_size);
            }
            DockPosition::Right => {
                self.container.set_orientation(Orientation::Vertical);
                LayerShell::set_anchor(&self.container, Anchor::Right);
                LayerShell::set_margin(&self.container, gtk4_layer_shell::Edge::Right, MARGIN_EDGE);
                self.container.set_size_request(dock_height, dock_axis_size);
            }
        }

        // Le layer-shell centre automatiquement sur l'edge (Anchor::Bottom = centré horizontal)
        // Pas besoin de calculer x/y manuellement comme dans la source TS (GNOME addTopChrome)
        // Le layer-shell gère le positioning selon les anchors.
    }

    // --- STYLE (traduit de _applyDockStyle) ---

    /// Applique le style CSS du dock (opacité, couleur, coins arrondis).
    /// Traduit de _applyDockStyle() — le blur est géré par Wayfire natif.
    pub fn apply_style(&self) {
        let cfg = self.config.borrow();
        let (r, g, b) = cfg.background_color;
        let alpha = cfg.dock_opacity;
        let radius = cfg.border_radius;

        // FIX GTK4 : container.style = "..." → add_css_class + CssProvider
        // En GTK4 on applique le style via CSS. Soit inline (set_css_classes)
        // soit via un CssProvider global. Ici on génère du CSS inline.
        let css = format!(
            ".astra-dock-container {{
                background-color: rgba({}, {}, {}, {});
                border-radius: {}px;
                padding: 4px 10px;
                spacing: 6px;
            }}",
            r, g, b, alpha, radius
        );

        // Charger le CSS via GtkCssProvider
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(css.as_bytes());

        // Appliquer au container (style priority = USER pour override)
        gtk4::style_context_add_provider_for_display(
            &self.container.display(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_USER as u32,
        );

        // NOTE : blur est géré par Wayfire blur plugin (natif, pas de code Rust).
        // Dans la source TS, Shell.BlurEffect était ajouté au container.
        // En AstraOS : Wayfire blur s'applique aux surfaces layer-shell automatiquement
        // si le plugin blur est activé dans la config Wayfire.
        // Aucun code Rust nécessaire pour le blur. ✅
    }

    // --- POSITION (traduit de _applyDockPosition) ---

    /// Change la position du dock (bottom/top/left/right).
    /// Traduit de _applyDockPosition().
    pub fn apply_dock_position(&self) {
        let cfg = self.config.borrow();
        let is_vertical = cfg.position == DockPosition::Left || cfg.position == DockPosition::Right;
        self.container.set_orientation(if is_vertical {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        });

        // Clear tous les anchors puis set le bon
        LayerShell::set_anchor(&self.container, Anchor::Bottom);
        LayerShell::set_anchor(&self.container, match cfg.position {
            DockPosition::Bottom => Anchor::Bottom,
            DockPosition::Top => Anchor::Top,
            DockPosition::Left => Anchor::Left,
            DockPosition::Right => Anchor::Right,
        });

        // Le pivot de magnification dépend de la position (traduit de la source TS)
        // Bottom → pivot (0.5, 1.0) centre-bas
        // Top → pivot (0.5, 0.0) centre-haut
        // Left → pivot (0.0, 0.5) gauche-centre
        // Right → pivot (1.0, 0.5) droite-centre
        // → à passer au Magnification::set_pivot_point() quand magnification est connectée
    }

    // --- APP CLICK (traduit de _onAppClicked) ---

    /// Comportement au clic sur une app du dock.
    /// Traduit de _onAppClicked() — la logique window management change
    /// (Shell.WindowTracker → Wayfire IPC).
    ///
    /// Logique (identique à la source) :
    ///   - fenêtre focalisée + visible → minimize
    ///   - fenêtre minimized → restore + activate
    ///   - fenêtre non-focalisée → activate
    ///   - pas de fenêtre → open new window
    pub fn on_app_clicked(&self, app_id: &str) {
        // FIX GTK4/Wayfire : Shell.WindowTracker → Wayfire IPC
        // 1. Demander à Wayfire la liste des fenêtres (wlr-foreign-toplevel)
        // 2. Filtrer par app_id
        // 3. Si fenêtre trouvée :
        //    - si focused + visible → minimize (Wayfire IPC: focus another window)
        //    - si minimized → unminimize + activate
        //    - sinon → activate
        // 4. Si pas de fenêtre → lancer l'app via gio::AppInfo::launch()

        // Version simplifiée (sans Wayfire IPC complet) :
        let app_info = gio::AppInfo::default_for_type(app_id, false);
        if let Some(app) = app_info {
            // Pour l'instant : lance l'app (activate/new window)
            // En prod : vérifier si fenêtre existe via Wayfire IPC → toggle minimize/activate
            let _ = app.launch(&[], None::<&gio::AppLaunchContext>);
        }
    }

    // --- BOUNCE ON LAUNCH (traduit de _onFocusAppChanged) ---

    /// Déclenche le bounce d'une icône quand l'app est lancée.
    /// Traduit de _onFocusAppChanged() — le tracking de focus change
    /// (Shell.WindowTracker::focus-app → Wayfire IPC focus event).
    ///
    /// Logique (identique) :
    ///   - Track focus app changes
    ///   - Si nouvelle app focus (pas déjà focused récemment) → bounce
    ///   - Debounce 400ms (évite bounce répété)
    pub fn on_focus_changed(&self, app_id: &str) {
        let cfg = self.config.borrow();
        if !cfg.bounce_on_launch {
            return;
        }
        // FIX Wayfire : tracker focus via Wayfire IPC (event "focus-window")
        // → callback avec l'app_id de la fenêtre focus
        // → appeler icon_manager.bounce_for_app(app_id)
        // (debounce 400ms via glib::timeout_add_local — même logique que la source)
    }

    // --- UPDATE ICON COUNT (pour update_position) ---

    /// Met à jour le nombre d'icônes (appelé par IconManager quand les icons changent).
    pub fn set_icon_count(&self, count: usize) {
        *self.icon_count.borrow_mut() = count;
        self.update_position();
    }

    // --- CONFIG UPDATE (traduit des settings changed handlers) ---

    /// Met à jour la config (quand les settings changent).
    /// Traduit des settings.connect("changed::...") handlers.
    pub fn update_config(&self, config: DockConfig) {
        *self.config.borrow_mut() = config;
        self.apply_style();
        self.apply_dock_position();
        self.update_position();
    }

    /// Toggle la visibilité du dock (Super+D).
    /// Traduit de _toggleDockVisibility().
    pub fn toggle_visibility(&self) {
        let visible = self.container.is_visible();
        self.container.set_visible(!visible);
    }
}

// ==============================================================================
// NOTE : les sous-modules suivants seront traduits séparément :
//   - IconManager (iconManager.ts) → app icons, running indicators, bounce
//   - DockVisibility (dockVisibility.ts) → auto-hide/show animation
//   - Intellihide (intellihide.ts) → hide quand fenêtre maximisée
//   - WindowPreview (windowPreview.ts) → thumbnails de fenêtres
//
// Le DockManager ci-dessus orchestre ces modules (comme la source TS).
// L'architecture est identique : DockManager → {IconManager, Magnification,
// DockVisibility, Intellihide, WindowPreview, MediaManager}
// ==============================================================================
