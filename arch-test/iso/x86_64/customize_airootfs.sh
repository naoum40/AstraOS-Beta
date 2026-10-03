#!/usr/bin/env bash
set -e -u

echo "=== AstraOS Arch Test customization ==="

# Create user astra (Hyprland refuses root — Wayland security)
useradd -m -G wheel -s /bin/bash astra 2>/dev/null || true
echo "astra:astraos" | chpasswd 2>/dev/null || true
mkdir -p /home/astra/.config/hypr
cp /etc/skel/.config/hypr/hyprland.conf /home/astra/.config/hypr/ 2>/dev/null || true
chown -R astra:astra /home/astra 2>/dev/null || true

# Enable services
systemctl enable greetd.service 2>/dev/null || true
systemctl enable NetworkManager.service 2>/dev/null || true
systemctl enable pipewire.service 2>/dev/null || true
systemctl enable pipewire-pulse.service 2>/dev/null || true
systemctl enable wireplumber.service 2>/dev/null || true

# Activer le theme Plymouth AstraOS
plymouth-set-default-theme astraos 2>/dev/null || true
echo "✓ Plymouth theme AstraOS activé"

# Timezone
ln -sf /usr/share/zoneinfo/Europe/Paris /etc/localtime 2>/dev/null || true

# Locale
echo "en_US.UTF-8 UTF-8" > /etc/locale.gen
echo "fr_FR.UTF-8 UTF-8" >> /etc/locale.gen
locale-gen 2>/dev/null || true
echo "LANG=en_US.UTF-8" > /etc/locale.conf

# === Build astra-shell-minimal ===
echo "→ Building astra-shell-minimal..."
ASTRA_SHELL_SRC="/usr/src/astra-shell-minimal"
mkdir -p "$ASTRA_SHELL_SRC"

if [ -d /astraos-build/astra-shell-minimal ]; then
    cp -r /astraos-build/astra-shell-minimal/* "$ASTRA_SHELL_SRC/"
    cd "$ASTRA_SHELL_SRC"
    cargo build --release 2>&1 || {
        echo "WARN: cargo build failed, astra-shell not installed"
        exit 0
    }
    install -Dm755 target/release/astra-shell /usr/bin/astra-shell
    echo "✓ astra-shell-minimal installed to /usr/bin/astra-shell"
else
    echo "WARN: astra-shell-minimal source not found"
fi

echo "=== AstraOS Arch Test customization complete ==="
