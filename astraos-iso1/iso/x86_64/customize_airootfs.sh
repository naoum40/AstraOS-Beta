#!/usr/bin/env bash
# AstraOS airootfs customization script
# Runs INSIDE the chroot during archiso build

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

# Create AstraOS config directories
mkdir -p /etc/astra
mkdir -p /usr/share/backgrounds/astraos
mkdir -p /usr/share/pixmaps/astraos
mkdir -p /usr/share/icons/astraos
echo "✓ AstraOS directories created"

# Install paru-bin from AUR (in chroot, requires network access in build container)
# This may fail if AUR is not accessible during build - that's OK, paru will be installed at first boot if missing
if ! command -v paru &> /dev/null; then
    echo "→ Attempting paru-bin installation from AUR..."
    # We need a temporary build user since makepkg refuses to run as root
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

# Final message
echo "=== AstraOS airootfs customization complete ==="
echo "    Default user: astra (password: astraos)"
echo "    Default root password: astraos (CHANGE AT INSTALL)"
echo "    Locale: fr_FR.UTF-8"
echo "    Keyboard: fr-latin9"
echo "    Timezone: Europe/Paris"
