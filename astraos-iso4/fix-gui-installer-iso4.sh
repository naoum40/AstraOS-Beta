#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════════════╗
# ║  AstraOS ISO 4 — GUI INSTALLER (façon Windows)                       ║
# ║                                                                        ║
# ║  Au boot :                                                            ║
# ║    1. Auto-login root → Hyprland démarre                              ║
# ║    2. astra-setup (GUI GTK4) s'affiche : logo + spinner + champs     ║
# ║    3. User tape username + password → clique "Start AstraOS"          ║
# ║    4. astra-setup crée l'user + set le password                        ║
# ║    5. astra-setup lance astra-shell → desktop glassmorphism           ║
# ║                                                                        ║
# ║  Interface GUI (pas terminal) — boutons, champs, spinner               ║
# ║  Plus de problème de login — l'user crée son PROPRE password           ║
# ║                                                                        ║
# ║  ATTENTION : ce script crée + build un nouveau app Rust+GTK4           ║
# ║  Build time : ~1h15-1h30 (cargo build astra-setup + astra-shell)       ║
# ║                                                                        ║
# ║  Usage : cd astraos-iso4 && bash fix-gui-installer-iso4.sh              ║
# ╚══════════════════════════════════════════════════════════════════════╝
set -e

cd /workspaces/AstraOS-Beta/astraos-iso4 2>/dev/null || cd astraos-iso4 2>/dev/null || {
    echo "✗ Lance ce script depuis la racine de AstraOS-Beta"
    exit 1
}

AIROOTFS="iso/x86_64/airootfs"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  AstraOS ISO 4 — GUI INSTALLER (façon Windows)               ║"
echo "║  Logo + spinner + champs GUI → desktop                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# FIX 1 : copytoram=n (mount direct → 2 GB RAM suffit)
# ═══════════════════════════════════════════════════════════════════════
echo "→ FIX 1 : copytoram=n..."
sed -i 's/archisolabel=%ARCHISO_LABEL%/archisolabel=%ARCHISO_LABEL% copytoram=n/' iso/x86_64/syslinux/syslinux.cfg
sed -i 's/archisolabel=%ARCHISO_LABEL%/archisolabel=%ARCHISO_LABEL% copytoram=n/' iso/x86_64/boot/loaders/entries/astraos.conf
sed -i 's/archisolabel=%ARCHISO_LABEL%/archisolabel=%ARCHISO_LABEL% copytoram=n/' iso/x86_64/efiboot/loader/entries/astraos.conf 2>/dev/null || true
echo "  ✓ copytoram=n"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# FIX 2 : Crée astra-setup (app Rust+GTK4 — le GUI installer)
# ═══════════════════════════════════════════════════════════════════════
echo "→ FIX 2 : Crée astra-setup (GUI installer Rust+GTK4)..."
mkdir -p astra-setup/src

cat > astra-setup/Cargo.toml << 'CARGO_EOF'
[package]
name = "astra-setup"
version = "0.1.0"
edition = "2021"
description = "AstraOS First-Boot Setup Wizard"
license = "MIT"

[dependencies]
gtk4 = "0.9"
glib = "0.20"
gio = "0.20"

[[bin]]
name = "astra-setup"
path = "src/main.rs"
CARGO_EOF

cat > astra-setup/src/main.rs << 'RUST_EOF'
// astra-setup — AstraOS First-Boot Setup Wizard (GTK4)
// Logo AstraOS + spinner + champs username/password + bouton Start
// Au clic : crée l'user + set le password + lance astra-shell

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
    // Si setup déjà fait → lance astra-shell direct
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

    // Logo
    let logo = Label::builder().label("✦ AstraOS").css_classes(["logo"]).build();
    container.append(&logo);
    let subtitle = Label::builder().label("First-Boot Setup").css_classes(["subtitle"]).build();
    container.append(&subtitle);

    // Spinner (points qui tournent)
    let spinner = Spinner::builder().build();
    spinner.set_spinning(true);
    spinner.set_size_request(40, 40);
    container.append(&spinner);

    let spacer = Label::builder().label(" ").build();
    container.append(&spacer);

    // Username
    let user_label = Label::builder().label("Username").css_classes(["field-label"]).halign(gtk4::Align::Start).build();
    container.append(&user_label);
    let user_entry = Entry::builder().placeholder_text("astra").build();
    container.append(&user_entry);

    // Password
    let pass_label = Label::builder().label("Password").css_classes(["field-label"]).halign(gtk4::Align::Start).margin_top(10).build();
    container.append(&pass_label);
    let pass_entry = PasswordEntry::builder().show_peek_icon(true).build();
    container.append(&pass_entry);

    // Start button
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
RUST_EOF

echo "  ✓ astra-setup créé (Cargo.toml + src/main.rs ~170 lignes)"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# FIX 3 : Ajoute astra-setup au build (customize_airootfs.sh)
# ═══════════════════════════════════════════════════════════════════════
echo "→ FIX 3 : Ajoute astra-setup au customize..."

# Ajoute le build de astra-setup APRÈS le build de astra-shell
# Cherche la ligne "=== greetd config ===" et insère AVANT
sed -i '/^# === greetd config ===/i\
# === Astra Setup build (GUI first-boot wizard) ===\
echo "→ Copying Astra Setup source to /usr/src/astra-setup..."\
mkdir -p /usr/src/astra-setup\
if [ -d /astraos-build/astra-setup ]; then\
    cp -r /astraos-build/astra-setup/* /usr/src/astra-setup/\
    echo "→ Building Astra Setup..."\cd /usr/src/astra-setup\
    cargo build --release\
    install -Dm755 target/release/astra-setup /usr/bin/astra-setup\
    echo "✓ Astra Setup installed to /usr/bin/astra-setup"\
fi\
' iso/x86_64/customize_airootfs.sh

echo "  ✓ astra-setup build ajouté au customize"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# FIX 4 : Hyprland lance astra-setup (pas astra-shell direct)
# ═══════════════════════════════════════════════════════════════════════
echo "→ FIX 4 : Hyprland lance astra-setup au boot..."
sed -i 's/^exec-once = astra-shell$/exec-once = astra-setup/' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf
sed -i 's/^exec-once = astra-shell$/exec-once = astra-setup/' astra-core/hyprland.conf 2>/dev/null || true
echo "  ✓ hyprland.conf : exec-once = astra-setup (astra-setup lancera astra-shell après setup)"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# FIX 5 : Auto-login root (pour que Hyprland démarre sans login)
# ═══════════════════════════════════════════════════════════════════════
echo "→ FIX 5 : Auto-login root..."
mkdir -p "$AIROOTFS/etc/systemd/system/getty@tty1.service.d"
cat > "$AIROOTFS/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'AUTOLOGIN_EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --noclear %I $TERM
AUTOLOGIN_EOF

# Root password vide
cat > "$AIROOTFS/etc/shadow" << 'SHADOW_EOF'
root::19000:0:99999:7:::
SHADOW_EOF

cat > "$AIROOTFS/etc/passwd" << 'PASSWD_EOF'
root:x:0:0::/root:/bin/bash
PASSWD_EOF

cat > "$AIROOTFS/etc/group" << 'GROUP_EOF'
root:x:0:root
wheel:x:10:
GROUP_EOF

cat > "$AIROOTFS/etc/gshadow" << 'GSHADOW_EOF'
root:*::root
wheel:*::
GSHADOW_EOF

# .profile root : lance Hyprland auto
cat > "$AIROOTFS/root/.profile" << 'PROFILE_EOF'
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
    exec Hyprland
fi
PROFILE_EOF
echo "  ✓ root auto-login + .profile lance Hyprland"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# FIX 6 : AstraOS branding (supprime "Arch Linux" partout)
# ═══════════════════════════════════════════════════════════════════════
echo "→ FIX 6 : AstraOS branding (nettoie Arch Linux)..."

# /etc/os-release
cat > "$AIROOTFS/etc/os-release" << 'OSRELEASE_EOF'
NAME="AstraOS"
PRETTY_NAME="AstraOS 0.4.0 (Live)"
ID=astraos
ID_LIKE=arch
VERSION="0.4.0"
VERSION_ID=0.4.0
ANSI_COLOR="0;35"
HOME_URL="https://astraos.org"
SUPPORT_URL="https://astraos.org"
BUG_REPORT_URL="https://github.com/naoum40/AstraOS-Beta"
OSRELEASE_EOF

# /etc/lsb-release
cat > "$AIROOTFS/etc/lsb-release" << 'LSBRELEASE_EOF'
DISTRIB_ID="AstraOS"
DISTRIB_RELEASE="0.4.0"
DISTRIB_DESCRIPTION="AstraOS 0.4.0 (Live)"
LSBRELEASE_EOF

# /etc/issue (logo ASCII)
cat > "$AIROOTFS/etc/issue" << 'ISSUE_EOF'

    ___  _____ _____ _____    ____  __   __    _   _ _____ _____
   / _ \|  ___|  ___|  ___|  / ___| \ \ / /   | | | /  ___|  ___|
  / /_\ \ |_  | |_  | |_    / /      \ V /    | | | \ `--.| |_
  |  _  |  _| |  _| |  _|  | |   ___  \ /     | | | |`--. \  _|
  | | | | |   | |   | |    | |__| (_) | |     \ \_/ /\__/ / |
  \_| |_|_|   \_|   \_|     \____\___/\_|      \___/\____/\_|


ISSUE_EOF

# hostname
echo "astraos" > "$AIROOTFS/etc/hostname"

# hosts
cat > "$AIROOTFS/etc/hosts" << 'HOSTS_EOF'
127.0.0.1   localhost
::1         localhost
127.0.1.1   astraos
HOSTS_EOF

echo "  ✓ /etc/os-release + lsb-release + issue + hostname = AstraOS (plus de 'Arch Linux')"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# FIX 7 : Rend useradd/chpasswd du customize non-fatal
# ═══════════════════════════════════════════════════════════════════════
echo "→ FIX 7 : customize non-fatal..."
sed -i 's#^useradd -m -G wheel -s /bin/bash astra#useradd -m -G wheel -s /bin/bash astra 2>/dev/null || true#' iso/x86_64/customize_airootfs.sh 2>/dev/null || true
sed -i 's#^echo "root:astraos" | chpasswd#echo "root:astraos" | chpasswd 2>/dev/null || true#' iso/x86_64/customize_airootfs.sh
sed -i 's#^echo "astra:astraos" | chpasswd#echo "astra:astraos" | chpasswd 2>/dev/null || true#' iso/x86_64/customize_airootfs.sh
echo "  ✓ non-fatal"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# VÉRIFICATIONS + REBUILD
# ═══════════════════════════════════════════════════════════════════════
echo "══════════ VÉRIFICATIONS ══════════"
echo "copytoram : $(grep -c copytoram iso/x86_64/syslinux/syslinux.cfg)"
echo "astra-setup/Cargo.toml : $(ls astra-setup/Cargo.toml 2>&1)"
echo "astra-setup/src/main.rs : $(ls astra-setup/src/main.rs 2>&1)"
echo "hyprland exec-once astra-setup : $(grep 'exec-once = astra-setup' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf 2>&1)"
echo "os-release AstraOS : $(grep -c 'AstraOS' "$AIROOTFS/etc/os-release")"
echo "root password vide : $(grep '^root::' "$AIROOTFS/etc/shadow" && echo '✓' || echo '✗')"
echo ""

echo "→ Rebuild ISO 4 (~1h15 — cargo build astra-setup + astra-shell + apps)..."
echo ""
docker rmi astraos-builder:latest 2>/dev/null || true
make build-iso pack=core

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ✦ ISO 4 — GUI INSTALLER (façon Windows) !                      ║"
echo "║                                                                  ║"
echo "║  Au boot :                                                       ║"
echo "║    1. Auto-login root → Hyprland fullscreen                     ║"
echo "║    2. astra-setup (GUI) : logo + spinner + username/pass        ║"
echo "║    3. User tape username + password → Start                     ║"
echo "║    4. astra-shell démarre → desktop glassmorphism                ║"
echo "║                                                                  ║"
echo "║  RAM VM = 2048 MB (copytoram=n)                                ║"
echo "║  Plus de 'Arch Linux' → 100% AstraOS branding                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
