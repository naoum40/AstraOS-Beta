use crate::config::ShellConfig;
use crate::hyprland_ipc::HyprlandClient;
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox};
use gtk4::{Application, ApplicationWindow, Button, Label, Orientation};

pub struct Bar {
    window: ApplicationWindow,
}

impl Bar {

    pub fn new(app: &Application, _config: &ShellConfig, _hyprland: &HyprlandClient) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Shell Bar")
            .default_width(800)
            .default_height(48)
            .decorated(false)
            .build();

        window.add_css_class("astra-bar");

        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        window.set_child(Some(&container));

        let start_button = Button::builder()
            .label("\u{2726}") // ✦ four-pointed star
            .tooltip_text("AstraOS Menu")
            .css_classes(["astra-button", "astra-start"])
            .build();
        start_button.connect_clicked(|_| {
            log::info!("Start button clicked - launcher toggle (TODO: action map)");
        });
        container.append(&start_button);

        let sep = Label::new(Some("|"));
        sep.set_opacity(0.3);
        container.append(&sep);

        let workspace_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .build();
        for i in 1..=9 {
            let ws_btn = Button::builder()
                .label(&i.to_string())
                .tooltip_text(&format!("Switch to workspace {}", i))
                .css_classes(["astra-button", "astra-workspace"])
                .build();
            let ws_id = i;
            ws_btn.connect_clicked(move |_| {
                log::info!("Switch to workspace {}", ws_id);
            });
            workspace_box.append(&ws_btn);
        }
        container.append(&workspace_box);

        let spacer = GtkBox::builder().hexpand(true).build();
        container.append(&spacer);

        let clock = Label::builder()
            .label("--:--")
            .css_classes(["astra-clock"])
            .build();
        container.append(&clock);

        let clock_clone = clock.clone();
        glib::source::timeout_add_local(std::time::Duration::from_secs(1), move || {
            let now = chrono::Local::now();
            clock_clone.set_label(&now.format("%H:%M").to_string());
            glib::ControlFlow::Continue
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
