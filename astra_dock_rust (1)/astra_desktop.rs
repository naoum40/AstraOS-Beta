// ==============================================================================
// AstraOS Desktop — Bureau (wallpaper + desktop icons + dock integration)
// ==============================================================================
// Crée le bureau AstraOS :
//   1. Layer-shell Background → wallpaper (fond d'écran)
//   2. Desktop icons → apps système AstraOS (clic = lance l'app)
//   3. Le dock (déjà traduit) est géré séparément par DockManager
// ==============================================================================
use gtk4::prelude::*;
use gtk4::{Box, Image, Label, Orientation, Picture, Widget};
use gtk4_layer_shell::{Layer, LayerShell, Anchor};
use gio::prelude::*;
use std::path::Path;

/// App système AstraOS (pour les icônes du bureau).
struct DesktopApp {
    name: &'static str,
    icon: &'static str,      // nom icône freedesktop ou chemin
    exec: &'static str,      // commande pour lancer
    desktop_file: &'static str, // .desktop file ID
}

/// Liste des apps système AstraOS (du repo astra-apps/iso4).
const ASTRA_APPS: &[DesktopApp] = &[
    DesktopApp { name: "Settings",      icon: "preferences-system-symbolic",       exec: "astra-settings",      desktop_file: "astra-settings.desktop" },
    DesktopApp { name: "AstraPass",     icon: "system-lock-screen-symbolic",       exec: "astra-pass",          desktop_file: "astra-pass.desktop" },
    DesktopApp { name: "Photos",        icon: "image-x-generic-symbolic",          exec: "astra-photos",        desktop_file: "astra-photos.desktop" },
    DesktopApp { name: "Screenshot",    icon: "camera-photo-symbolic",             exec: "astra-screenshot",    desktop_file: "astra-screenshot.desktop" },
    DesktopApp { name: "Sticky Notes",  icon: "document-edit-symbolic",            exec: "astra-sticky-notes",  desktop_file: "astra-sticky-notes.desktop" },
    DesktopApp { name: "Widgets",       icon: "view-app-grid-symbolic",            exec: "astra-widgets",       desktop_file: "astra-widgets.desktop" },
    DesktopApp { name: "Minesweeper",   icon: "applications-games-symbolic",       exec: "astra-minesweeper",   desktop_file: "astra-minesweeper.desktop" },
    DesktopApp { name: "Snake",         icon: "applications-games-symbolic",       exec: "astra-snake",         desktop_file: "astra-snake.desktop" },
    DesktopApp { name: "Tic-Tac-Toe",   icon: "applications-games-symbolic",       exec: "astra-tictactoe",     desktop_file: "astra-tictactoe.desktop" },
];

/// Bureau AstraOS — wallpaper + desktop icons.
pub struct Desktop;

impl Desktop {
    /// Crée le bureau : wallpaper (Background layer) + desktop icons (Bottom layer above wallpaper).
    pub fn new(wallpaper_path: &str) {
        Self::create_wallpaper_layer(wallpaper_path);
        Self::create_desktop_icons();
    }

    /// Layer-shell Background → affiche le wallpaper plein écran.
    fn create_wallpaper_layer(path: &str) {
        let window = gtk4::Window::new();

        LayerShell::init(&window);
        LayerShell::set_layer(&window, Layer::Background);
        // Ancre sur tous les bords = full screen
        LayerShell::set_anchor(&window, Anchor::Top);
        LayerShell::set_anchor(&window, Anchor::Bottom);
        LayerShell::set_anchor(&window, Anchor::Left);
        LayerShell::set_anchor(&window, Anchor::Right);
        // Exclusive zone = -1 (background, ne prend pas d'espace)
        LayerShell::set_exclusive_zone(&window, -1);

        // Charger le wallpaper
        if Path::new(path).exists() {
            let picture = Picture::for_filename(path);
            picture.set_content_fit(gtk4::ContentFit::Cover);
            window.set_child(Some(&picture));
        } else {
            // Fallback : couleur de fond sombre
            let fallback = Box::new(Orientation::Vertical, 0);
            fallback.add_css_class("desktop-fallback");
            let label = Label::new(Some("AstraOS"));
            label.add_css_class("desktop-fallback-label");
            fallback.append(&label);
            window.set_child(Some(&fallback));
        }

        window.show();
    }

    /// Desktop icons — grid d'apps système cliquables.
    /// Layer-shell Bottom (au-dessus du wallpaper, en-dessous des fenêtres).
    fn create_desktop_icons() {
        let window = gtk4::Window::new();

        LayerShell::init(&window);
        LayerShell::set_layer(&window, Layer::Bottom);
        LayerShell::set_anchor(&window, Anchor::Top);
        LayerShell::set_anchor(&window, Anchor::Left);
        LayerShell::set_exclusive_zone(&window, -1);

        // Grid d'icônes (vertical, aligné en haut à gauche)
        let grid = Box::new(Orientation::Vertical, 16);
        grid.set_margin_start(32);
        grid.set_margin_top(32);
        grid.set_halign(gtk4::Align::Start);
        grid.set_valign(gtk4::Align::Start);

        // Créer 2 colonnes d'icônes
        let row1 = Box::new(Orientation::Horizontal, 24);
        let row2 = Box::new(Orientation::Horizontal, 24);

        for (i, app) in ASTRA_APPS.iter().enumerate() {
            let icon_btn = Self::create_desktop_icon(app);
            if i < 5 {
                row1.append(&icon_btn);
            } else {
                row2.append(&icon_btn);
            }
        }

        grid.append(&row1);
        grid.append(&row2);
        window.set_child(Some(&grid));
        window.show();
    }

    /// Crée une icône desktop cliquable (icon + label).
    fn create_desktop_icon(app: &DesktopApp) -> Box {
        let container = Box::new(Orientation::Vertical, 4);
        container.set_size_request(80, 80);
        container.add_css_class("desktop-icon");

        // Icône
        let icon = Image::from_icon_name(app.icon);
        icon.set_pixel_size(48);

        // Label
        let label = Label::new(Some(app.name));
        label.add_css_class("desktop-icon-label");
        label.set_max_width_chars(12);

        container.append(&icon);
        container.append(&label);

        // Double-clic → lance l'app
        let exec = app.exec.to_string();
        let click = gtk4::GestureClick::new();
        click.set_button(0); // all buttons
        click.connect_released(move |gesture, npress, _x, _y| {
            if npress >= 1 {
                // Lancer l'app via gio::AppInfo ou commande directe
                let exec_clone = exec.clone();
                glib::spawn_future_local(async move {
                    let _ = std::process::Command::new(&exec_clone).spawn();
                });
                gesture.set_state(gtk4::EventSequenceState::Claimed);
            }
        });
        container.add_controller(click);

        // Hover effect
        let motion = gtk4::EventControllerMotion::new();
        let container_clone = container.clone();
        motion.connect_enter(move |_ctrl, _x, _y| {
            container_clone.add_css_class("desktop-icon-hover");
        });
        let container_clone2 = container.clone();
        motion.connect_leave(move |_ctrl| {
            container_clone2.remove_css_class("desktop-icon-hover");
        });
        container.add_controller(motion);

        container
    }
}
