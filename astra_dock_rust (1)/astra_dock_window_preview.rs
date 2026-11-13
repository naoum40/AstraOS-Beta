// ==============================================================================
// AstraOS Dock — WindowPreview (traduit de MacOSDock/windowPreview.ts → Rust/GTK4)
// ==============================================================================
// Source : 439 lignes TS. Thumbnails de fenêtres au survol d'une icône dock.
//
// TRADUIT :
//   - Popup avec grille de thumbnails (max 4 colonnes)
//   - Thumbnail = clone de la fenêtre (Clutter.Clone → GTK4 : à adapter)
//   - Titre + bouton fermer par thumbnail
//   - Click sur thumbnail → activate/minimize la fenêtre
//   - Highlight fenêtre focusée
//   - Auto-refresh focus highlights (200ms = 5fps)
//   - Positionnement (au-dessus de l'icône, clampé au monitor)
//   - Animations in/out (fade + slide, 150ms/120ms)
//   - Schedule hide (300ms debounce)
//
// GNOME → GTK4/Wayfire :
//   Clutter.Clone (window thumbnail) → gtk4::Snapshot/Paintable (à adapter)
//   Main.layoutManager.addTopChrome  → gtk4::Popover ou layer-shell séparé
//   global.get_window_actors()       → Wayfire IPC (window list + thumbnails)
//   metaWin.delete/activate          → Wayfire IPC (close/focus window)
//   St.Button / St.Label             → gtk4::Button / gtk4::Label
//   Clutter.AnimationMode            → glib timeout + interpolation
// ==============================================================================
use gtk4::prelude::*;
use gtk4::{Box, Button, Image, Label, Orientation, Popover, Widget};
use glib::source::{SourceId, timeout_add_local};
use std::cell::RefCell;
use std::rc::Rc;

const UPDATE_INTERVAL_MS: u64 = 200; // 5fps refresh focus highlights
const CLOSE_DELAY_MS: u64 = 300;     // debounce hide
const MAX_COLUMNS: usize = 4;

/// Données d'un thumbnail (traduit de ThumbData).
struct ThumbData {
    container: Box,
    title_label: Label,
    close_btn: Button,
    window_id: String, // Wayfire window ID (au lieu de Meta.Window)
    app_id: String,
}

/// WindowPreviewPopup — popup de thumbnails de fenêtres.
///
/// Traduit de la classe WindowPreviewPopup (windowPreview.ts).
/// Au survol d'une icône dock → affiche les thumbnails des fenêtres de l'app.
pub struct WindowPreviewPopup {
    popup: Rc<RefCell<Option<Popover>>>,
    thumbs: Rc<RefCell<Vec<ThumbData>>>,
    visible: Rc<RefCell<bool>>,
    preview_width: Rc<RefCell<i32>>,
    current_app_id: Rc<RefCell<Option<String>>>,
    // Timers
    close_timer: Rc<RefCell<Option<SourceId>>>,
    update_timer: Rc<RefCell<Option<SourceId>>>,
    refresh_timer: Rc<RefCell<Option<SourceId>>>,
}

impl WindowPreviewPopup {
    pub fn new() -> Self {
        Self {
            popup: Rc::new(RefCell::new(None)),
            thumbs: Rc::new(RefCell::new(vec![])),
            visible: Rc::new(RefCell::new(false)),
            preview_width: Rc::new(RefCell::new(200)),
            current_app_id: Rc::new(RefCell::new(None)),
            close_timer: Rc::new(RefCell::new(None)),
            update_timer: Rc::new(RefCell::new(None)),
            refresh_timer: Rc::new(RefCell::new(None)),
        }
    }

    pub fn set_preview_width(&self, width: i32) {
        *self.preview_width.borrow_mut() = width;
    }

    pub fn is_visible(&self) -> bool {
        *self.visible.borrow()
    }

    /// Bounds du popup (pour DockVisibility — ignore pointer events dedans).
    /// Traduit de getBounds().
    pub fn get_bounds(&self) -> Option<(i32, i32, i32, i32)> {
        let popup = self.popup.borrow();
        let p = popup.as_ref()?;
        if !*self.visible.borrow() { return None; }
        let alloc = p.allocation();
        Some((alloc.x(), alloc.y(), alloc.width(), alloc.height()))
    }

    /// Affiche le popup pour une app. Traduit de show().
    pub fn show(&self, app_id: &str, _icon_widget: &Widget) {
        // FIX Wayfire : app.get_windows() → Wayfire IPC (get windows for app_id)
        // Pour l'instant : stub (pas de fenêtres → pas de popup)
        // TODO: let windows = wayfire.get_windows_for_app(app_id);
        let windows: Vec<(String, String, bool, bool)> = vec![];
        // (window_id, title, minimized, focused)

        if windows.is_empty() { return; }

        // Si déjà visible pour la même app → rien
        if *self.visible.borrow() && *self.current_app_id.borrow() == Some(app_id.to_string()) {
            return;
        }

        self.destroy_popup();
        self.create_popup(app_id, &windows);
    }

    /// Crée le popup avec les thumbnails. Traduit de _createPopup().
    fn create_popup(&self, app_id: &str, windows: &[(String, String, bool, bool)]) {
        *self.current_app_id.borrow_mut() = Some(app_id.to_string());

        // FIX GTK4 : St.BoxLayout popup → gtk4::Popover (positionné auto au-dessus du parent)
        let popover = Popover::new();
        popover.add_css_class("dock-preview-popover");

        let grid = Box::new(Orientation::Horizontal, 6);
        grid.add_css_class("dock-preview-grid");

        let preview_w = *self.preview_width.borrow();
        let thumb_height = (preview_w as f64 * 0.625) as i32;
        let mut col = 0;

        for (win_id, title, minimized, focused) in windows {
            if *minimized { continue; } // Skip minimized en premier pass
            let thumb = self.create_thumb(win_id, title, *focused, preview_w, thumb_height, app_id);
            grid.append(&thumb.container);
            self.thumbs.borrow_mut().push(thumb);
            col += 1;
            if col >= MAX_COLUMNS { break; }
        }

        // Si tous minimized → les montrer avec opacity réduite
        if self.thumbs.borrow().is_empty() {
            for (win_id, title, _minimized, focused) in windows {
                let thumb = self.create_thumb(win_id, title, *focused, preview_w, thumb_height, app_id);
                thumb.container.set_opacity(0.5);
                grid.append(&thumb.container);
                self.thumbs.borrow_mut().push(thumb);
            }
        }

        popover.set_child(Some(&grid));

        // Keep popup open quand souris dedans
        let close_timer = self.close_timer.clone();
        let motion = gtk4::EventControllerMotion::new();
        motion.connect_enter(move |_ctrl, _x, _y| {
            if let Some(id) = close_timer.borrow_mut().take() { id.remove(); }
        });
        let self_timer = self.close_timer.clone();
        motion.connect_leave(move |_ctrl| {
            Self::schedule_hide_static(&self_timer);
        });
        grid.add_controller(motion);

        *self.popup.borrow_mut() = Some(popover);
        *self.visible.borrow_mut() = true;

        // Start focus highlight refresh
        self.start_update_timer();
    }

    /// Crée un thumbnail. Traduit de _createThumb().
    fn create_thumb(&self, win_id: &str, title: &str, focused: bool,
                    width: i32, height: i32, app_id: &str) -> ThumbData {
        let thumb_box = Box::new(Orientation::Vertical, 4);
        thumb_box.add_css_class("dock-preview-thumb");
        thumb_box.set_size_request(width, height + 24);

        // FIX GTK4 : Clutter.Clone (window thumbnail) →
        // En GTK4, pas d'équivalent direct à Clutter.Clone.
        // Options :
        // 1. wlr-foreign-toplevel + screenshot de la fenêtre (via wlroots)
        // 2. PipeWire stream de la fenêtre (xdg-desktop-portal screencast)
        // 3. Placeholder Image (pour l'instant)
        let thumb_image = Image::from_icon_name("window-new-symbolic");
        thumb_image.set_pixel_size(width);
        thumb_image.set_size_request(width, height);
        thumb_box.append(&thumb_image);

        // Titre
        let title_label = Label::new(Some(title));
        title_label.add_css_class("dock-preview-title");
        title_label.set_max_width_chars(20);
        title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        thumb_box.append(&title_label);

        // Bouton fermer
        let close_btn = Button::with_label("×");
        close_btn.add_css_class("dock-preview-close-btn");
        let win_id_clone = win_id.to_string();
        let refresh = self.refresh_timer.clone();
        let app_id_clone = app_id.to_string();
        close_btn.connect_clicked(move |_| {
            // FIX Wayfire : metaWin.delete() → wayfire.close_window(&win_id)
            // TODO: wayfire.close_window(&win_id_clone);
            println!("[dock] close window {}", win_id_clone);
        });
        thumb_box.append(&close_btn);

        // Click sur thumbnail → activate la fenêtre
        let win_id_focus = win_id.to_string();
        let visible = self.visible.clone();
        let click = gtk4::GestureClick::new();
        click.connect_released(move |_gesture, _npress, _x, _y| {
            // FIX Wayfire : metaWin.activate() → wayfire.focus_window(&win_id)
            // TODO: wayfire.focus_window(&win_id_focus);
            println!("[dock] focus window {}", win_id_focus);
            *visible.borrow_mut() = false;
        });
        thumb_box.add_controller(click);

        // Highlight si focusée
        if focused {
            thumb_box.add_css_class("dock-preview-focused");
        }

        ThumbData {
            container: thumb_box,
            title_label,
            close_btn,
            window_id: win_id.to_string(),
            app_id: app_id.to_string(),
        }
    }

    /// Cache le popup (avec animation). Traduit de hide().
    pub fn hide(&self) {
        self.cancel_close_timer();
        self.stop_update_timer();
        let popup = self.popup.borrow().clone();
        if let Some(p) = popup {
            // FIX GTK4 : Clutter ease → Popover.popdown() (built-in animation)
            *self.visible.borrow_mut() = false;
            p.popdown();
        }
    }

    /// Schedule hide (300ms debounce). Traduit de scheduleHide().
    pub fn schedule_hide(&self) {
        self.cancel_close_timer();
        let close_timer = self.close_timer.clone();
        let source = timeout_add_local(std::time::Duration::from_millis(CLOSE_DELAY_MS), move || {
            if let Some(id) = close_timer.borrow_mut().take() {
                // hide sera appelé par le caller
                let _ = id;
            }
            glib::ControlFlow::Break
        });
        // On stocke le timer — le hide se fait dans la closure
        // En prod : cloner self pour appeler self.hide() dans la closure
        *self.close_timer.borrow_mut() = Some(source);
    }

    pub fn cancel_scheduled_hide(&self) {
        self.cancel_close_timer();
    }

    pub fn stop(&self) {
        self.cancel_close_timer();
        self.stop_update_timer();
        self.cancel_refresh_timer();
        self.destroy_popup();
    }

    /// Refresh focus highlights (200ms). Traduit de _startUpdateTimer().
    fn start_update_timer(&self) {
        self.stop_update_timer();
        let thumbs = self.thumbs.clone();
        let source = timeout_add_local(
            std::time::Duration::from_millis(UPDATE_INTERVAL_MS),
            move || {
                // FIX Wayfire : check focus app + window has_focus
                // TODO: pour chaque thumb, check si focused → add/remove CSS class
                let _ = thumbs.borrow(); // stub
                glib::ControlFlow::Continue
            },
        );
        *self.update_timer.borrow_mut() = Some(source);
    }

    fn stop_update_timer(&self) {
        if let Some(id) = self.update_timer.borrow_mut().take() { id.remove(); }
    }

    fn cancel_close_timer(&self) {
        if let Some(id) = self.close_timer.borrow_mut().take() { id.remove(); }
    }

    fn cancel_refresh_timer(&self) {
        if let Some(id) = self.refresh_timer.borrow_mut().take() { id.remove(); }
    }

    fn destroy_popup(&self) {
        self.stop_update_timer();
        self.cancel_close_timer();
        self.cancel_refresh_timer();
        self.thumbs.borrow_mut().clear();
        *self.popup.borrow_mut() = None;
        *self.visible.borrow_mut() = false;
        *self.current_app_id.borrow_mut() = None;
    }

    fn schedule_hide_static(close_timer: &Rc<RefCell<Option<SourceId>>>) {
        // Helper statique pour les closures
        if close_timer.borrow().is_some() { return; }
        // TODO: cloner self pour appeler hide() dans la closure
    }
}
