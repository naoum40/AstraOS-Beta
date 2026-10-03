use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label,
    PasswordEntry, Spinner, Orientation, CssProvider, StyleContext,
};
use glib::clone;
use std::process::Command;
use std::fs;

const APP_ID: &str = "org.astraos.Setup";
const SETUP_DONE_FILE: &str = "/etc/astraos-setup-done";

fn main() {
    if std::path::Path::new(SETUP_DONE_FILE).exists() {
        let _ = Command::new("astra-shell").spawn();
        return;
    }

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let css = r#"
        window { background-color: #0b0d17; }
        .logo { font-size: 48px; font-weight: bold; color: #818cf8; margin-bottom: 10px; }
        .subtitle { font-size: 14px; color: #94a3b8; margin-bottom: 30px; }
        .field-label { color: #f1f5f9; font-size: 13px; margin-bottom: 5px; }
        entry { background-color: rgba(255,255,255,0.06); color: #f1f5f9; border-radius: 8px; padding: 10px 14px; border: 1px solid rgba(129,140,248,0.2); }
        entry:focus { border-color: #8b5cf6; }
        .start-btn { background: linear-gradient(135deg, #818cf8, #8b5cf6, #E040FB); color: white; font-weight: bold; font-size: 15px; border-radius: 10px; padding: 12px 32px; border: none; }
        spinner { color: #E040FB; margin: 10px; }
    "#;
    let provider = CssProvider::new();
    provider.load_from_data(css.as_bytes());
    StyleContext::add_provider_for_display(
        &app.default_display().expect("display"), &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app).title("AstraOS Setup")
        .default_width(520).default_height(480)
        .decorated(false).fullscreen().build();

    let container = GtkBox::builder()
        .orientation(Orientation::Vertical).spacing(8)
        .halign(gtk4::Align::Center).valign(gtk4::Align::Center)
        .margin_top(60).margin_bottom(60).margin_start(60).margin_end(60).build();

    let logo = Label::builder().label("✦ AstraOS").css_classes(["logo"]).build();
    container.append(&logo);
    let subtitle = Label::builder().label("First-Boot Setup").css_classes(["subtitle"]).build();
    container.append(&subtitle);

    let spinner = Spinner::builder().build();
    spinner.set_spinning(true);
    spinner.set_size_request(40, 40);
    container.append(&spinner);

    let spacer = Label::builder().label(" ").build();
    container.append(&spacer);

    let user_label = Label::builder().label("Username").css_classes(["field-label"]).halign(gtk4::Align::Start).build();
    container.append(&user_label);
    let user_entry = Entry::builder().placeholder_text("astra").build();
    container.append(&user_entry);

    let pass_label = Label::builder().label("Password").css_classes(["field-label"]).halign(gtk4::Align::Start).margin_top(10).build();
    container.append(&pass_label);
    let pass_entry = PasswordEntry::builder().show_peek_icon(true).build();
    container.append(&pass_entry);

    let spacer2 = Label::builder().label(" ").build();
    container.append(&spacer2);
    let btn = Button::builder().label("Start AstraOS").css_classes(["start-btn"]).build();

    btn.connect_clicked(clone!(@strong user_entry, @strong pass_entry, @strong window => move |_| {
        let username = user_entry.text().to_string();
        let password = pass_entry.text().to_string();
        let username = if username.is_empty() { "astra".to_string() } else { username };
        let password = if password.is_empty() { "astraos".to_string() } else { password };

        let _ = Command::new("useradd").args(["-m", "-G", "wheel", "-s", "/bin/bash", &username]).status();
        let _ = Command::new("sh").args(["-c", &format!("echo '{}:{}' | chpasswd", username, password)]).status();
        let _ = Command::new("sh").args(["-c", &format!("echo 'root:{}' | chpasswd", password)]).status();
        let _ = fs::write(SETUP_DONE_FILE, "done");
        let _ = Command::new("astra-shell").spawn();
        window.close();
    }));

    container.append(&btn);
    window.set_child(Some(&container));
    window.present();
}
