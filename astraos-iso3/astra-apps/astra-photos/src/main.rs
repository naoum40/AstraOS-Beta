use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::gdk;
use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, FileChooserAction,
    FileChooserDialog, FileFilter, Label, Orientation, Picture, ResponseType, ScrolledWindow,
};

const APP_ID: &str = "com.astraos.Photos";

const SUPPORTED_EXT: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];

fn is_image(path: &Path) -> bool {
    path.extension()
        .map(|e| {
            let e = e.to_string_lossy().to_lowercase();
            SUPPORTED_EXT.contains(&e.as_str())
        })
        .unwrap_or(false)
}

struct AppState {
    current_path: Option<PathBuf>,
    current_dir: Option<PathBuf>,
    files_in_dir: Vec<PathBuf>,
    current_index: Option<usize>,
    zoom: f64,
    texture: Option<gdk::Texture>,
}

impl AppState {
    fn new() -> Self {
        Self {
            current_path: None,
            current_dir: None,
            files_in_dir: Vec::new(),
            current_index: None,
            zoom: 1.0,
            texture: None,
        }
    }
}

fn load_image(
    path: PathBuf,
    picture: &Picture,
    state: &Rc<RefCell<AppState>>,
) -> Result<(), String> {
    let file = gio::File::for_path(&path);
    let texture = gdk::Texture::from_file(&file).map_err(|e| e.to_string())?;

    let dir = path.parent().map(|p| p.to_path_buf());
    let mut files: Vec<PathBuf> = Vec::new();
    if let Some(dir) = &dir {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if is_image(&p) {
                    files.push(p);
                }
            }
            files.sort();
        }
    }
    let current_index = files.iter().position(|p| *p == path);

    {
        let mut s = state.borrow_mut();
        s.current_path = Some(path);
        s.current_dir = dir;
        s.files_in_dir = files;
        s.current_index = current_index;
        s.texture = Some(texture.clone());
        s.zoom = 1.0;
    }

    picture.set_paintable(Some(&texture));
    apply_zoom(picture, state);
    Ok(())
}

fn apply_zoom(picture: &Picture, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    if let Some(tex) = &s.texture {
        let w = tex.width() as f64;
        let h = tex.height() as f64;
        let scaled_w = (w * s.zoom).round() as i32;
        let scaled_h = (h * s.zoom).round() as i32;
        picture.set_size_request(scaled_w, scaled_h);
    }
}

fn open_file_dialog(
    window: &ApplicationWindow,
    picture: &Picture,
    state: &Rc<RefCell<AppState>>,
    status: &Label,
) {
    let filter = FileFilter::new();
    filter.set_name(Some("Images"));
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    filter.add_mime_type("image/jpg");
    filter.add_mime_type("image/webp");
    filter.add_mime_type("image/gif");

    let dialog = FileChooserDialog::new(
        Some("Open Image"),
        Some(window),
        FileChooserAction::Open,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Open", ResponseType::Accept),
        ],
    );
    dialog.set_filter(&filter);

    let picture_clone = picture.clone();
    let state_clone = state.clone();
    let status_clone = status.clone();
    dialog.connect_response(move |d, response| {
        if response == ResponseType::Accept {
            if let Some(file) = d.file() {
                if let Some(path) = file.path() {
                    match load_image(path.clone(), &picture_clone, &state_clone) {
                        Ok(_) => {
                            status_clone.set_text(&format!("Loaded: {}", path.display()));
                        }
                        Err(e) => {
                            status_clone.set_text(&format!("Error: {}", e));
                        }
                    }
                }
            }
        }
        d.close();
    });
    dialog.show();
}

fn navigate(direction: i32, picture: &Picture, state: &Rc<RefCell<AppState>>, status: &Label) {
    let next_path = {
        let s = state.borrow();
        if let Some(idx) = s.current_index {
            if s.files_in_dir.is_empty() {
                None
            } else {
                let len = s.files_in_dir.len() as i32;
                let new_idx = ((idx as i32 + direction).rem_euclid(len)) as usize;
                s.files_in_dir.get(new_idx).cloned()
            }
        } else {
            None
        }
    };

    if let Some(path) = next_path {
        match load_image(path.clone(), picture, state) {
            Ok(_) => {
                status.set_text(&format!("Loaded: {}", path.display()));
            }
            Err(e) => {
                status.set_text(&format!("Error: {}", e));
            }
        }
    } else {
        status.set_text("No images to navigate");
    }
}

fn adjust_zoom(factor: f64, picture: &Picture, state: &Rc<RefCell<AppState>>, status: &Label) {
    {
        let mut s = state.borrow_mut();
        s.zoom *= factor;
        if s.zoom < 0.5 {
            s.zoom = 0.5;
        }
        if s.zoom > 4.0 {
            s.zoom = 4.0;
        }
    }
    apply_zoom(picture, state);
    let s = state.borrow();
    status.set_text(&format!("Zoom: {:.2}x", s.zoom));
}

fn reset_zoom(picture: &Picture, state: &Rc<RefCell<AppState>>, status: &Label) {
    {
        let mut s = state.borrow_mut();
        s.zoom = 1.0;
    }
    apply_zoom(picture, state);
    status.set_text("Zoom: 1.00x");
}

fn main() {
    env_logger::init();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Photos")
            .default_width(900)
            .default_height(600)
            .build();

        let css = CssProvider::new();
        css.load_from_data(
            "
            .astra-photos {
                background: rgba(18, 18, 32, 0.85);
            }
            .astra-photos toolbar {
                background: rgba(30, 25, 50, 0.6);
                border-radius: 12px;
                padding: 8px;
                margin: 6px;
            }
            .astra-photos button {
                background: rgba(120, 80, 200, 0.45);
                color: white;
                border-radius: 10px;
                padding: 8px 14px;
                font-weight: 600;
            }
            .astra-photos button:hover {
                background: rgba(160, 100, 220, 0.65);
            }
            .astra-photos button:active {
                background: rgba(100, 60, 180, 0.7);
            }
            .astra-photos scrolledwindow {
                background: rgba(0, 0, 0, 0.3);
                border-radius: 12px;
                margin: 8px;
            }
            .astra-photos label.status {
                color: rgba(255, 255, 255, 0.7);
                font-size: 12px;
                padding: 4px 12px;
            }
            ",
        );
        let display = WidgetExt::display(&window);
        gtk4::style_context_add_provider_for_display(
            &display,
            &css,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let outer = GtkBox::builder().orientation(Orientation::Vertical).build();

        let toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .css_classes(["toolbar"])
            .build();

        let btn_open = Button::builder().label("Open").build();
        let btn_zoom_in = Button::builder().label("Zoom In").build();
        let btn_zoom_out = Button::builder().label("Zoom Out").build();
        let btn_zoom_reset = Button::builder().label("Reset Zoom").build();
        let btn_prev = Button::builder().label("Previous").build();
        let btn_next = Button::builder().label("Next").build();

        toolbar.append(&btn_open);
        toolbar.append(&btn_prev);
        toolbar.append(&btn_next);
        toolbar.append(&btn_zoom_out);
        toolbar.append(&btn_zoom_in);
        toolbar.append(&btn_zoom_reset);

        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let picture = Picture::builder()
            .can_shrink(false)
            .keep_aspect_ratio(true)
            .build();

        scrolled.set_child(Some(&picture));

        let status = Label::builder()
            .label("No image loaded")
            .css_classes(["status"])
            .halign(gtk4::Align::Start)
            .build();

        outer.append(&toolbar);
        outer.append(&scrolled);
        outer.append(&status);

        window.set_child(Some(&outer));

        let state = Rc::new(RefCell::new(AppState::new()));

        {
            let window_clone = window.clone();
            let picture_clone = picture.clone();
            let state_clone = state.clone();
            let status_clone = status.clone();
            btn_open.connect_clicked(move |_| {
                open_file_dialog(&window_clone, &picture_clone, &state_clone, &status_clone);
            });
        }
        {
            let picture_clone = picture.clone();
            let state_clone = state.clone();
            let status_clone = status.clone();
            btn_zoom_in.connect_clicked(move |_| {
                adjust_zoom(1.25, &picture_clone, &state_clone, &status_clone);
            });
        }
        {
            let picture_clone = picture.clone();
            let state_clone = state.clone();
            let status_clone = status.clone();
            btn_zoom_out.connect_clicked(move |_| {
                adjust_zoom(1.0 / 1.25, &picture_clone, &state_clone, &status_clone);
            });
        }
        {
            let picture_clone = picture.clone();
            let state_clone = state.clone();
            let status_clone = status.clone();
            btn_zoom_reset.connect_clicked(move |_| {
                reset_zoom(&picture_clone, &state_clone, &status_clone);
            });
        }
        {
            let picture_clone = picture.clone();
            let state_clone = state.clone();
            let status_clone = status.clone();
            btn_prev.connect_clicked(move |_| {
                navigate(-1, &picture_clone, &state_clone, &status_clone);
            });
        }
        {
            let picture_clone = picture.clone();
            let state_clone = state.clone();
            let status_clone = status.clone();
            btn_next.connect_clicked(move |_| {
                navigate(1, &picture_clone, &state_clone, &status_clone);
            });
        }

        window.add_css_class("astra-photos");
        window.present();
    });

    app.run();
}
