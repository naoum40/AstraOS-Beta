use crate::config::ShellConfig;
use glib::Propagation;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, FlowBox, FlowBoxChild, SearchEntry};

type AppEntry = (&'static str, &'static str, &'static str);

const STATIC_APPS: &[AppEntry] = &[
    ("\u{1F310}", "Firefox", "firefox"),                // 🌐
    ("\u{1F4C1}", "Files", "thunar"),                   // 📁
    ("\u{1F4BB}", "Terminal", "kitty"),                 // 💻
    ("\u{2699}\u{FE0F}", "Settings", "astra-settings"), // ⚙️
    ("\u{1F4DD}", "Text Editor", "vim"),                // 📝
    ("\u{1F3AC}", "VLC", "vlc"),                        // 🎬
    ("\u{1F4CA}", "Task Manager", "btop"),              // 📊
    ("\u{1F3A8}", "Display Settings", "wdisplays"),     // 🎨
];

pub struct Launcher {
    #[allow(dead_code)]
    window: ApplicationWindow,
    #[allow(dead_code)]
    search: SearchEntry,
    #[allow(dead_code)]
    grid: FlowBox,
}

impl Launcher {
    pub fn new(app: &Application, _config: &ShellConfig) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Launcher")
            .default_width(620)
            .default_height(550)
            .decorated(false)
            .visible(false) 
            .build();

        window.add_css_class("astra-launcher");

        let main_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();
        window.set_child(Some(&main_box));

        let search = SearchEntry::builder()
            .placeholder_text("Rechercher des applications...")
            .css_classes(["astra-search"])
            .build();
        main_box.append(&search);

        let grid = FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::Single)
            .column_spacing(12)
            .row_spacing(12)
            .build();

        let scrolled = gtk4::ScrolledWindow::builder()
            .child(&grid)
            .vexpand(true)
            .build();
        main_box.append(&scrolled);

        Self::populate_apps(&grid);

        let grid_clone = grid.clone();
        search.connect_changed(clone!(
            #[weak]
            grid_clone,
            move |entry| {
                let text = entry.text().to_lowercase();
                let mut i = 0;
                while let Some(flow_child) = grid_clone.child_at_index(i) {
                    if let Some(btn) = flow_child.child() {
                        if let Some(button) = btn.downcast_ref::<gtk4::Button>() {
                            let label = button.label().unwrap_or_default().to_lowercase();
                            let tooltip = button.tooltip_text().unwrap_or_default().to_lowercase();
                            let visible = label.contains(&text) || tooltip.contains(&text);
                            flow_child.set_visible(visible);
                        }
                    }
                    i += 1;
                }
            }
        ));

        let window_clone = window.clone();
        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                window_clone.set_visible(false);
                return Propagation::Stop;
            }
            Propagation::Proceed
        });
        window.add_controller(controller);

        Self {
            window,
            search,
            grid,
        }
    }

    fn populate_apps(grid: &FlowBox) {
        for (icon, name, exec) in STATIC_APPS {
            let btn = gtk4::Button::builder()
                .label(&format!("{}\n{}", icon, name))
                .tooltip_text(*name)
                .width_request(96)
                .height_request(96)
                .css_classes(["astra-app-icon"])
                .build();

            let exec_cmd = exec.to_string();
            btn.connect_clicked(move |_| {
                log::info!("Launching: {}", exec_cmd);
                match std::process::Command::new(&exec_cmd).spawn() {
                    Ok(_) => log::debug!("Spawned {} OK", exec_cmd),
                    Err(e) => log::warn!("Failed to spawn {}: {}", exec_cmd, e),
                }
            });

            let child = FlowBoxChild::new();
            child.set_child(Some(&btn));
            grid.insert(&child, -1); // -1 = append
        }
    }

    #[allow(dead_code)] 
    pub fn toggle(&self) {
        let visible = self.window.is_visible();
        self.window.set_visible(!visible);
        if !visible {
            self.window.present();
            self.search.grab_focus();
        }
    }

    #[allow(dead_code)] 
    pub fn present(&self) {
        self.window.present();
    }
}
