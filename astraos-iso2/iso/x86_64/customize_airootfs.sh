#!/usr/bin/env bash

set -e -u

echo "=== AstraOS airootfs customization starting ==="

ln -sf /usr/share/zoneinfo/Europe/Paris /etc/localtime
echo "✓ Timezone set to Europe/Paris"

locale-gen
echo "✓ Locales generated"

echo "KEYMAP=fr-latin9" > /etc/vconsole.conf
echo "FONT=eurlatgr" >> /etc/vconsole.conf
echo "✓ Console keymap set to fr-latin9"

useradd -m -G wheel -s /bin/bash astra
echo "✓ User 'astra' created (member of wheel)"

echo "root:astraos" | chpasswd
echo "astra:astraos" | chpasswd
echo "✓ Default passwords set (will be changed at install)"

systemctl enable NetworkManager.service
systemctl enable iwd.service
systemctl enable greetd.service
systemctl enable systemd-timesyncd.service
systemctl enable systemd-resolved.service
echo "✓ Services enabled"

mkdir -p /etc/astra
mkdir -p /usr/share/backgrounds/astraos
mkdir -p /usr/share/pixmaps/astraos
mkdir -p /usr/share/icons/astraos
echo "✓ AstraOS directories created"

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

chmod +x /usr/local/bin/* 2>/dev/null || true

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
    echo "      Astra Shell will be built on first boot or in dev container"
fi

echo "→ Configuring greetd to use Astra Shell..."
if [ -f /etc/greetd/config.toml ]; then
    cat > /etc/greetd/config.toml << 'GREETD_EOF'
[terminal]
vt = 1
switch = true

[default_session]
command = "astra-shell"
user = "astra"
GREETD_EOF
    echo "✓ greetd configured for Astra Shell"
fi

mkdir -p /etc/astra
mkdir -p /usr/share/astraos/themes
mkdir -p /usr/share/astraos/icons
mkdir -p /usr/share/astraos/sounds

echo "=== AstraOS airootfs customization complete ==="
echo "    Default user: astra (password: astraos)"
echo "    Default root password: astraos (CHANGE AT INSTALL)"
echo "    Locale: fr_FR.UTF-8"
echo "    Keyboard: fr-latin9"
echo "    Timezone: Europe/Paris"