#!/usr/bin/env bash
# AstraOS airootfs customization script
# Runs INSIDE the chroot during archiso build
# CLEAN VERSION — all 9 bug fixes applied (archiso hook in mkinitcpio,
# greetd launches Hyprland, hyprpaper+wofi packages, brave browser,
# astra-shell exec-once, NO console=ttyS0 sed)

set -e -u

echo "=== AstraOS airootfs customization starting ==="

# Set timezone
ln -sf /usr/share/zoneinfo/Europe/Paris /etc/localtime
echo "✓ Timezone set to Europe/Paris"

# Generate locales
locale-gen
echo "✓ Locales generated"

# Set vconsole keymap
echo "KEYMAP=fr-latin9" > /etc/vconsole.conf
echo "FONT=eurlatgr" >> /etc/vconsole.conf
echo "✓ Console keymap set to fr-latin9"

# Create user 'astra'
useradd -m -G wheel -s /bin/bash astra
echo "✓ User 'astra' created (member of wheel)"

# Set passwords (default = astraos, will be changed at install)
echo "root:astraos" | chpasswd
echo "astra:astraos" | chpasswd
echo "✓ Default passwords set (will be changed at install)"

# Enable services
systemctl enable NetworkManager.service
systemctl enable iwd.service
systemctl enable greetd.service
systemctl enable systemd-timesyncd.service
systemctl enable systemd-resolved.service
echo "✓ Services enabled"

# Ensure PipeWire has a valid default config set in the image.
mkdir -p /etc/pipewire /etc/pipewire/client.conf.d /etc/pipewire/pipewire.conf.d
if [ -d /usr/share/pipewire ]; then
    cp -n /usr/share/pipewire/*.conf /etc/pipewire/ 2>/dev/null || true
fi
mkdir -p /etc/wireplumber/main.lua.d
if [ -d /usr/share/wireplumber ]; then
    cp -n /usr/share/wireplumber/main.lua.d/*.lua /etc/wireplumber/main.lua.d/ 2>/dev/null || true
fi
echo "✓ PipeWire default configs ensured"

# Create AstraOS config directories
mkdir -p /etc/astra
mkdir -p /usr/share/backgrounds/astraos
mkdir -p /usr/share/pixmaps/astraos
mkdir -p /usr/share/icons/astraos
echo "✓ AstraOS directories created"

# Install paru-bin from AUR (non-fatal if fails)
if ! command -v paru &> /dev/null; then
    echo "→ Attempting paru-bin installation from AUR..."
    useradd -m -G wheel -s /bin/bash paru-builder
    echo "paru-builder:astraos" | chpasswd
    echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/99-paru-builder
    cd /tmp
    sudo -u paru-builder git clone https://aur.archlinux.org/paru-bin.git
    cd /tmp/paru-bin
    sudo -u paru-builder makepkg -si --noconfirm --noprogressbar || echo "⚠ paru install failed (will be retried at first boot)"
    cd /
    rm -rf /tmp/paru-bin
    userdel -r paru-builder
    rm /etc/sudoers.d/99-paru-builder
    echo "✓ paru-bin installation attempted"
else
    echo "✓ paru already available"
fi

# Make scripts executable
chmod +x /usr/local/bin/* 2>/dev/null || true

# === Astra Shell build (ISO 2) ===
echo "→ Copying Astra Shell source to /usr/src/astra-shell..."
mkdir -p /usr/src/astra-shell
if [ -d /astraos-build/astra-shell ]; then
    cp -r /astraos-build/astra-shell/* /usr/src/astra-shell/
    echo "→ Building Astra Shell..."
    if [ -f /astraos-build/iso/x86_64/build-astra-shell.sh ]; then
        chmod +x /astraos-build/iso/x86_64/build-astra-shell.sh
        /astraos-build/iso/x86_64/build-astra-shell.sh
    else
        echo "WARN: build-astra-shell.sh not found, skipping"
    fi
else
    echo "WARN: astra-shell/ directory not found in build context"
fi

# === greetd config ===
# BUG FIX #7: launches Hyprland (NOT astra-shell — astra-shell needs Hyprland running first)
# Hyprland's exec-once in hyprland.conf launches astra-shell after compositor starts.
echo "→ Configuring greetd to launch Hyprland..."
cat > /etc/greetd/config.toml << 'GREETD_EOF'
[terminal]
vt = 1
switch = true

[default_session]
command = "Hyprland"
user = "astra"
GREETD_EOF
echo "✓ greetd configured for Hyprland (astra-shell launched via exec-once)"

# Create AstraOS theme directories
mkdir -p /etc/astra
mkdir -p /usr/share/astraos/themes
mkdir -p /usr/share/astraos/icons
mkdir -p /usr/share/astraos/sounds

# === Astra apps build (ISO 3) ===
echo "→ Copying Astra apps source to /usr/src/astra-apps..."
mkdir -p /usr/src/astra-apps
if [ -d /astraos-build/astra-apps ]; then
    cp -r /astraos-build/astra-apps/* /usr/src/astra-apps/
    echo "→ Building Astra custom apps..."
    if [ -f /astraos-build/iso/x86_64/build-astra-apps.sh ]; then
        chmod +x /astraos-build/iso/x86_64/build-astra-apps.sh
        /astraos-build/iso/x86_64/build-astra-apps.sh
    fi
fi

# === Install Brave browser (AUR) ===
echo "→ Installing Brave browser from AUR..."
useradd -m -G wheel -s /bin/bash aur-builder 2>/dev/null || true
echo "aur-builder:astraos" | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/99-aur-builder
cd /tmp
sudo -u aur-builder git clone https://aur.archlinux.org/brave-bin.git 2>/dev/null || true
if [ -d /tmp/brave-bin ]; then
    cd /tmp/brave-bin
    sudo -u aur-builder makepkg -si --noconfirm --noprogressbar 2>/dev/null || echo "WARN: brave-bin install failed (will retry on first boot)"
fi
userdel -r aur-builder 2>/dev/null || true
rm -f /etc/sudoers.d/99-aur-builder

# === File associations ===
echo "→ Configuring file associations..."
mkdir -p /usr/share/applications
xdg-mime default brave-browser.desktop x-scheme-handler/http
xdg-mime default brave-browser.desktop x-scheme-handler/https
xdg-mime default brave-browser.desktop text/html
xdg-mime default astra-photos.desktop image/png
xdg-mime default astra-photos.desktop image/jpeg
xdg-mime default astra-photos.desktop image/jpg
xdg-mime default astra-photos.desktop image/webp
xdg-mime default astra-photos.desktop image/gif
xdg-mime default codium.desktop text/plain 2>/dev/null || xdg-mime default vim.desktop text/plain
xdg-mime default vlc.desktop video/mp4
xdg-mime default vlc.desktop video/x-matroska
xdg-mime default vlc.desktop video/x-msvideo
xdg-mime default vlc.desktop audio/mpeg
xdg-mime default vlc.desktop audio/flac
xdg-mime default brave-browser.desktop application/pdf
echo "✓ File associations configured"

# Create Pictures directory for screenshots
mkdir -p /etc/skel/Pictures

# Final message
echo "=== AstraOS airootfs customization complete ==="
echo "    Default user: astra (password: astraos)"
echo "    Locale: fr_FR.UTF-8"
echo "    Keyboard: fr-latin9"
echo "    Timezone: Europe/Paris"
