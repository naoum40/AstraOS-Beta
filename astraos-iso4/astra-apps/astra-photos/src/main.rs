// main.rs - AstraOS Photos image viewer.
//
// A lightweight image viewer built on `GtkPicture` inside a
// `GtkScrolledWindow`. Supports:
//   * Open (file chooser filtered to PNG/JPEG/WebP/GIF)
//   * Zoom In / Zoom Out (×1.25 per step, clamped 0.5×..4×)
//   * Reset Zoom (back to 1×)
//   * Previous / Next (scan the parent folder for sibling images)
//
// Zoom is implemented by wrapping the underlying `gdk::Texture` in a
// custom `ScaledPaintable` GObject that implements the `gdk::Paintable`
// interface. The paintable reports scaled intrinsic dimensions and
// renders the texture at the requested (zoomed) size, so the
// `GtkPicture` automatically re-lays-out and scrolls correctly for
// both zoom-in and zoom-out.
//
// Viewer state (current path, folder image list) is held in
// `Rc<RefCell<AppState>>` and shared across the button callbacks.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gio::ApplicationFlags;
use gtk4::gdk::{self, Paintable, PaintableFlags, Snapshot};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, FileChooserAction, FileChooserDialog,
    FileFilter, Orientation, Picture, ResponseType, ScrolledWindow,
};

/// GApplication ID registered with the GNOME session manager.
const APP_ID: &str = "org.astraos.Photos";

/// Zoom bounds and step.
const MIN_ZOOM: f64 = 0.5;
const MAX_ZOOM: f64 = 4.0;
const ZOOM_STEP: f64 = 1.25;

/// File extensions recognized as images when scanning a folder.
const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff"];

// =============================================================================
// ScaledPaintable - a Paintable wrapping a Texture with a zoom factor.
// =============================================================================

mod imp {
    use super::*;

    /// Inner state for `ScaledPaintable`.
    pub struct ScaledPaintable {
        /// The wrapped image texture (None = empty paintable).
        pub texture: RefCell<Option<gdk::Texture>>,
        /// Current zoom factor (1.0 = native size).
        pub zoom: Cell<f64>,
    }

    impl Default for ScaledPaintable {
        fn default() -> Self {
            Self {
                texture: RefCell::new(None),
                zoom: Cell::new(1.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScaledPaintable {
        const NAME: &'static str = "AstraScaledPaintable";
        type Type = super::ScaledPaintable;
        type ParentType = glib::Object;
        type Interfaces = (Paintable,);
    }

    impl ObjectImpl for ScaledPaintable {
        fn dispose(&self) {
            // Drop the strong reference on the texture so the
            // underlying GPU memory can be released as soon as the
            // paintable is torn down.
            self.texture.borrow_mut().take();
        }
    }

    impl PaintableImpl for ScaledPaintable {
        /// Intrinsic width = texture width × zoom (clamped to ≥ 1).
        fn intrinsic_width(&self) -> i32 {
            let tex = self.texture.borrow();
            let zoom = self.zoom.get();
            tex.as_ref()
                .map(|t| ((t.width() as f64) * zoom).max(1.0) as i32)
                .unwrap_or(0)
        }

        /// Intrinsic height = texture height × zoom (clamped to ≥ 1).
        fn intrinsic_height(&self) -> i32 {
            let tex = self.texture.borrow();
            let zoom = self.zoom.get();
            tex.as_ref()
                .map(|t| ((t.height() as f64) * zoom).max(1.0) as i32)
                .unwrap_or(0)
        }

        /// Aspect ratio is preserved because both dimensions scale
        /// by the same factor.
        fn intrinsic_aspect_ratio(&self) -> f64 {
            let tex = self.texture.borrow();
            tex.as_ref()
                .map(|t| t.intrinsic_aspect_ratio())
                .unwrap_or(0.0)
        }

        fn flags(&self) -> PaintableFlags {
            PaintableFlags::empty()
        }

        /// Render the texture stretched into the (already-zoomed)
        /// allocation given by the `GtkPicture`.
        fn snapshot(&self, snapshot: &Snapshot, width: f64, height: f64) {
            let tex = self.texture.borrow();
            if let Some(t) = tex.as_ref() {
                t.snapshot(snapshot, width, height);
            }
        }
    }
}

glib::wrapper! {
    /// A `gdk::Paintable` that scales a `gdk::Texture` by a zoom factor
    /// without copying the underlying pixel data.
    pub struct ScaledPaintable(ObjectSubclass<imp::ScaledPaintable>)
        @implements Paintable;
}

impl ScaledPaintable {
    /// Construct an empty `ScaledPaintable` (zoom = 1.0).
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Replace the wrapped texture and emit `invalidate-size` /
    /// `invalidate-contents` so the host `GtkPicture` re-queries the
    /// intrinsic dimensions and re-renders.
    pub fn set_texture(&self, texture: Option<&gdk::Texture>) {
        self.imp().texture.replace(texture.cloned());
        self.invalidate_contents();
        self.invalidate_size();
    }

    /// Read the current zoom factor.
    pub fn zoom(&self) -> f64 {
        self.imp().zoom.get()
    }

    /// Set the zoom factor and emit `invalidate-size` so the
    /// `GtkPicture` re-lays-out (and the `GtkScrolledWindow`
    /// reconfigures its scrollbars).
    pub fn set_zoom(&self, zoom: f64) {
        self.imp().zoom.set(zoom);
        self.invalidate_size();
    }
}

impl Default for ScaledPaintable {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// AppState + UI
// =============================================================================

/// Viewer state shared across UI callbacks via `Rc<RefCell<...>>`.
struct AppState {
    /// Path of the image currently displayed.
    current: Option<PathBuf>,
    /// Sorted list of sibling image files in the parent folder.
    folder_images: Vec<PathBuf>,
}

/// Entry point. Boots GTK, builds the viewer window, and runs the
/// GApplication until the window is closed.
fn main() {
    env_logger::init();
    log::info!("Astra Photos v0.3.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

/// Construct the main viewer window: 900x600, glassmorphism-dark, with
/// a toolbar at the top and a `GtkPicture` inside a `GtkScrolledWindow`.
fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Astra Photos")
        .default_width(900)
        .default_height(600)
        .build();
    window.add_css_class("astra-photos");

    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();
    window.set_child(Some(&root));

    // --- Toolbar ----------------------------------------------------
    let toolbar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .margin_start(8)
        .margin_end(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    toolbar.add_css_class("astra-toolbar");
    root.append(&toolbar);

    let btn_open = Button::builder()
        .label("Open")
        .css_classes(["astra-button"])
        .build();
    let btn_in = Button::builder()
        .label("Zoom In")
        .css_classes(["astra-button"])
        .build();
    let btn_out = Button::builder()
        .label("Zoom Out")
        .css_classes(["astra-button"])
        .build();
    let btn_reset = Button::builder()
        .label("Reset Zoom")
        .css_classes(["astra-button"])
        .build();
    let btn_prev = Button::builder()
        .label("Previous")
        .css_classes(["astra-button"])
        .build();
    let btn_next = Button::builder()
        .label("Next")
        .css_classes(["astra-button"])
        .build();

    toolbar.append(&btn_open);
    toolbar.append(&btn_in);
    toolbar.append(&btn_out);
    toolbar.append(&btn_reset);
    toolbar.append(&btn_prev);
    toolbar.append(&btn_next);

    // --- Image area -------------------------------------------------
    // A `GtkPicture` whose paintable is our `ScaledPaintable`. The
    // picture reacts to `invalidate-size` from the paintable by
    // re-laying-out, which in turn drives the `GtkScrolledWindow`
    // scrollbars.
    let scrolled = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    let picture = Picture::builder().build();
    let paintable = ScaledPaintable::new();
    picture.set_paintable(Some(&paintable));
    scrolled.set_child(Some(&picture));
    root.append(&scrolled);

    // --- Shared state -----------------------------------------------
    let state = Rc::new(RefCell::new(AppState {
        current: None,
        folder_images: Vec::new(),
    }));

    // --- Open -------------------------------------------------------
    // File chooser filtered to PNG/JPEG/WebP/GIF. Runs on the main
    // thread (modal) - no worker thread needed.
    {
        let paintable = paintable.clone();
        let state = state.clone();
        let window = window.clone();
        btn_open.connect_clicked(move |_| {
            let filter = FileFilter::new();
            filter.set_name(Some("Images"));
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/jpeg");
            filter.add_mime_type("image/webp");
            filter.add_mime_type("image/gif");

            let dialog = FileChooserDialog::new(
                Some("Open Image"),
                Some(&window),
                FileChooserAction::Open,
                &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
            );
            dialog.add_filter(&filter);

            let paintable = paintable.clone();
            let state = state.clone();
            dialog.connect_response(move |d, response| {
                if response == ResponseType::Ok {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            load_image(&state, &paintable, &path);
                        }
                    }
                }
                d.close();
            });
            dialog.present();
        });
    }

    // --- Zoom In ----------------------------------------------------
    {
        let paintable = paintable.clone();
        btn_in.connect_clicked(move |_| {
            let new_zoom = (paintable.zoom() * ZOOM_STEP).min(MAX_ZOOM);
            paintable.set_zoom(new_zoom);
            log::debug!("zoom in -> {:.2}", new_zoom);
        });
    }

    // --- Zoom Out ---------------------------------------------------
    {
        let paintable = paintable.clone();
        btn_out.connect_clicked(move |_| {
            let new_zoom = (paintable.zoom() / ZOOM_STEP).max(MIN_ZOOM);
            paintable.set_zoom(new_zoom);
            log::debug!("zoom out -> {:.2}", new_zoom);
        });
    }

    // --- Reset Zoom -------------------------------------------------
    {
        let paintable = paintable.clone();
        btn_reset.connect_clicked(move |_| {
            paintable.set_zoom(1.0);
            log::debug!("zoom reset -> 1.00");
        });
    }

    // --- Previous ---------------------------------------------------
    {
        let paintable = paintable.clone();
        let state = state.clone();
        btn_prev.connect_clicked(move |_| {
            go_sibling(&state, &paintable, -1);
        });
    }

    // --- Next -------------------------------------------------------
    {
        let paintable = paintable.clone();
        let state = state.clone();
        btn_next.connect_clicked(move |_| {
            go_sibling(&state, &paintable, 1);
        });
    }

    window.present();
}

/// Load the image at `path`, swap it into the paintable, refresh the
/// folder listing, and reset the zoom factor.
fn load_image(state: &Rc<RefCell<AppState>>, paintable: &ScaledPaintable, path: &Path) {
    let file = gio::File::for_path(path);
    match gdk::Texture::from_file(&file) {
        Ok(texture) => {
            let dims = (texture.width(), texture.height());
            paintable.set_texture(Some(&texture));
            paintable.set_zoom(1.0);
            let mut s = state.borrow_mut();
            s.folder_images = scan_folder(path);
            s.current = Some(path.to_path_buf());
            log::info!(
                "loaded {} ({}x{}, {} siblings)",
                path.display(),
                dims.0,
                dims.1,
                s.folder_images.len()
            );
        }
        Err(e) => log::error!("Failed to load {}: {}", path.display(), e),
    }
}

/// Move to the previous (`delta = -1`) or next (`delta = +1`) image
/// in the folder listing, wrapping around at the ends.
fn go_sibling(state: &Rc<RefCell<AppState>>, paintable: &ScaledPaintable, delta: i32) {
    // Snapshot what we need under the borrow, then drop it before any
    // potentially slow IO in `gdk::Texture::from_file`.
    let (folder_len, next_path) = {
        let s = state.borrow();
        let Some(current) = s.current.as_ref() else {
            log::warn!("no image loaded - cannot navigate");
            return;
        };
        let Some(pos) = s.folder_images.iter().position(|p| p == current) else {
            log::warn!("current image not in folder listing");
            return;
        };
        let len = s.folder_images.len() as i32;
        if len == 0 {
            return;
        }
        let next = (pos as i32 + delta).rem_euclid(len) as usize;
        (len, s.folder_images[next].clone())
    };
    let _ = folder_len;

    let file = gio::File::for_path(&next_path);
    match gdk::Texture::from_file(&file) {
        Ok(texture) => {
            paintable.set_texture(Some(&texture));
            paintable.set_zoom(1.0);
            state.borrow_mut().current = Some(next_path.clone());
            log::info!("navigated to {}", next_path.display());
        }
        Err(e) => log::error!("Failed to load {}: {}", next_path.display(), e),
    }
}

/// Scan the parent directory of `path` for sibling image files and
/// return them sorted lexicographically. Returns an empty `Vec` if
/// the parent is unreadable or `path` has no parent.
fn scan_folder(path: &Path) -> Vec<PathBuf> {
    let Some(parent) = path.parent() else {
        return Vec::new();
    };
    let mut images: Vec<PathBuf> = std::fs::read_dir(parent)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if !p.is_file() {
                return None;
            }
            let ext = p.extension()?.to_string_lossy().to_lowercase();
            if IMAGE_EXTS.contains(&ext.as_str()) {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    images.sort();
    images
}
