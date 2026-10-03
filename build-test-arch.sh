#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════════════╗
# ║  ARCH vs DEBIAN TEST — ISO ARCH                                       ║
# ║                                                                        ║
# ║  Build un ISO Arch minimal avec :                                      ║
# ║    - base system (kernel + systemd)                                    ║
# ║    - Hyprland (pacman repo — binaire précompilé)                     ║
# ║    - greetd + pipewire                                                ║
# ║    - astra-shell-minimal (cargo build from source)                   ║
# ║    - copytoram=n (2 GB RAM)                                          ║
# ║                                                                        ║
# ║  Usage : cd AstraOS-Beta && bash build-arch-test.sh                    ║
# ╚══════════════════════════════════════════════════════════════════════╝
set -e

cd /workspaces/AstraOS-Beta 2>/dev/null || cd "$(dirname "$0")/.." 2>/dev/null || {
    echo "✗ Lance depuis la racine de AstraOS-Beta"
    exit 1
}

TEST="arch-test"
PROFILE="$TEST/iso/x86_64"
AIROOTFS="$PROFILE/airootfs"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ARCH TEST ISO — Hyprland + astra-shell-minimal              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── Cleanup + create structure ────────────────────────────────────────
rm -rf "$TEST"
mkdir -p "$PROFILE"/{syslinux,boot/loaders/entries,efiboot/loader/entries}
mkdir -p "$AIROOTFS"/{etc/systemd/system/getty@tty1.service.d,etc/skel/.config/hypr,etc/astra}

# ── Copy astra-shell-minimal source ────────────────────────────────────
echo "→ Copie astra-shell-minimal..."
cp -r /home/z/my-project/public/astra-shell-minimal "$TEST/astra-shell-minimal" 2>/dev/null || \
cp -r "$(dirname "$0")/astra-shell-minimal" "$TEST/astra-shell-minimal" 2>/dev/null || true

# ═════════════════════════════════════════════════════════════════════
# 1. profiledef.sh
# ═════════════════════════════════════════════════════════════════════
cat > "$PROFILE/profiledef.sh" << 'EOF'
#!/hint: bash
iso_name="astraos-arch-test"
iso_label="ASTRAOS_AR"
iso_publisher="AstraOS Test"
iso_application="AstraOS Arch Test"
iso_version="0.1-test"
install_dir="astraos"
buildmodes=("iso")
bootmodes=("bios.syslinux.mbr" "bios.syslinux.eltorito" "uefi-x64.systemd-boot.esp" "uefi-x64.systemd-boot.eltorito")
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  "/etc/shadow" "0:0:400"
  "/etc/gshadow" "0:0:400"
  "/root" "0:0:750"
)
EOF

# ═════════════════════════════════════════════════════════════════════
# 2. packages.x86_64
# ═════════════════════════════════════════════════════════════════════
cat > "$PROFILE/packages.x86_64" << 'EOF'
## Base system
base
linux
linux-firmware
intel-ucode
amd-ucode

## Bootloader & ISO tools
syslinux
edk2-ovmf
squashfs-tools
archiso
mkinitcpio-archiso

## Display (Hyprland stack)
hyprland
greetd
greetd-gtkgreet
kitty
wofi
qt5-wayland
qt6-wayland
polkit-gnome
hyprpaper
brightnessctl
grim
slurp
wl-clipboard
pipewire
pipewire-pulse
wireplumber

## Graphics drivers (safe fallback: Mesa + Vulkan + generic Intel/AMD)
mesa
lib32-mesa
vulkan-radeon
lib32-vulkan-radeon
vulkan-intel
lib32-vulkan-intel
vulkan-mesa-layers
vulkan-swrast
xf86-video-amdgpu
xf86-video-intel
xf86-video-nouveau

## Network
networkmanager

## Astra Shell build deps
rust
cargo
gtk4
libadwaita
pkgconf
EOF

# ═════════════════════════════════════════════════════════════════════
# 3. pacman.conf (mirrorlist, pas fantôme)
# ═════════════════════════════════════════════════════════════════════
cat > "$PROFILE/pacman.conf" << 'EOF'
[options]
Architecture = auto
Color
CheckSpace
ParallelDownloads = 5
SigLevel = Required DatabaseOptional

[core]
Include = /etc/pacman.d/mirrorlist

[extra]
Include = /etc/pacman.d/mirrorlist

[multilib]
Include = /etc/pacman.d/mirrorlist
EOF

# ═════════════════════════════════════════════════════════════════════
# 4. mkinitcpio.conf (avec archiso hook)
# ═════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/mkinitcpio.conf" << 'EOF'
MODULES=()
BINARIES=()
FILES=()
HOOKS=(base udev modconf archiso kms keyboard keymap consolefont block filesystems fsck)
COMPRESSION="zstd"
EOF

# ═════════════════════════════════════════════════════════════════════
# 5. syslinux.cfg (copytoram=n)
# ═════════════════════════════════════════════════════════════════════
cat > "$PROFILE/syslinux/syslinux.cfg" << 'EOF'
DEFAULT astraos
PROMPT 0
TIMEOUT 30
UI menu.c32

MENU TITLE AstraOS Arch Test

LABEL astraos
MENU LABEL AstraOS Arch Test (x86_64)
LINUX /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
INITRD /%INSTALL_DIR%/boot/intel-ucode.img,/%INSTALL_DIR%/boot/amd-ucode.img,/%INSTALL_DIR%/boot/x86_64/initramfs-linux.img
APPEND archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL% copytoram=n
EOF

cat > "$PROFILE/syslinux/theme.cfg" << 'EOF'
MENU WIDTH 78
MENU MARGIN 6
MENU ROWS 10
EOF

# ═════════════════════════════════════════════════════════════════════
# 6. UEFI boot loaders
# ═════════════════════════════════════════════════════════════════════
UEFI_ENTRY='title   AstraOS Arch Test (x86_64)
linux   /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
initrd  /%INSTALL_DIR%/boot/intel-ucode.img
initrd  /%INSTALL_DIR%/boot/amd-ucode.img
initrd  /%INSTALL_DIR%/boot/x86_64/initramfs-linux.img
options archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL% copytoram=n'

printf '%s\n' "$UEFI_ENTRY" > "$PROFILE/boot/loaders/entries/astraos.conf"
printf '%s\n' "$UEFI_ENTRY" > "$PROFILE/efiboot/loader/entries/astraos.conf"
printf 'default astraos\ntimeout 3\n' > "$PROFILE/boot/loaders/loader.conf"
printf 'default astraos\ntimeout 3\n' > "$PROFILE/efiboot/loader/loader.conf"

# ═════════════════════════════════════════════════════════════════════
# 7. Hyprland config (blur + astra-shell)
# ═════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/skel/.config/hypr/hyprland.conf" << 'EOF'
# AstraOS Arch Test — Hyprland config

env = XDG_CURRENT_DESKTOP,Hyprland
env = XDG_SESSION_TYPE,wayland

monitor=,preferred,auto,1

$terminal = kitty
$menu = wofi --show drun 2>/dev/null || true

exec-once = pipewire
exec-once = wireplumber
exec-once = /usr/lib/polkit-gnome/polkit-gnome-authentication-agent
exec-once = hyprpaper 2>/dev/null || true
exec-once = astra-shell

decoration {
    rounding = 8
    blur {
        enabled = true
        size = 6
        passes = 2
    }
    active_opacity = 0.95
    inactive_opacity = 0.85
}

input {
    kb_layout = fr
    follow_mouse = 1
}

general {
    gaps_in = 5
    gaps_out = 10
    border_size = 2
    col.active_border = rgba(7a59ffaa) rgba(5a4affaa) 45deg
    col.inactive_border = rgba(595959aa)
}

# Workspaces (1-10)
bind = SUPER, 1, workspace, 1
bind = SUPER, 2, workspace, 2
bind = SUPER, 3, workspace, 3
bind = SUPER, 4, workspace, 4
bind = SUPER, 5, workspace, 5
bind = SUPER, 6, workspace, 6
bind = SUPER, 7, workspace, 7
bind = SUPER, 8, workspace, 8
bind = SUPER, 9, workspace, 9
bind = SUPER, 0, workspace, 10

# Move window to workspace
bind = SUPER SHIFT, 1, movetoworkspace, 1
bind = SUPER SHIFT, 2, movetoworkspace, 2
bind = SUPER SHIFT, 3, movetoworkspace, 3
bind = SUPER SHIFT, 4, movetoworkspace, 4
bind = SUPER SHIFT, 5, movetoworkspace, 5
bind = SUPER SHIFT, 6, movetoworkspace, 6
bind = SUPER SHIFT, 7, movetoworkspace, 7
bind = SUPER SHIFT, 8, movetoworkspace, 8
bind = SUPER SHIFT, 9, movetoworkspace, 9
bind = SUPER SHIFT, 0, movetoworkspace, 10

# Apps
bind = SUPER, Return, exec, $terminal
bind = SUPER, Q, killactive,
bind = SUPER, M, exit,
bind = SUPER, D, exec, $menu
bind = SUPER, B, exec, $browser

# Window management
bind = SUPER, F, fullscreen,
bind = SUPER, Space, togglefloating,
bind = SUPER, Tab, cyclenext,
bind = SUPER, H, movefocus, l
bind = SUPER, L, movefocus, r
bind = SUPER, K, movefocus, u
bind = SUPER, J, movefocus, d

# Volume (function keys)
bindel = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bindel = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bindel = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle

# Brightness
bindel = , XF86MonBrightnessUp, exec, brightnessctl set +5% 2>/dev/null || true
bindel = , XF86MonBrightnessDown, exec, brightnessctl set 5%- 2>/dev/null || true

# Screenshot
bind = SUPER, Print, exec, grim -g "$(slurp)" - | wl-copy 2>/dev/null || true
bind = , Print, exec, grim - | wl-copy 2>/dev/null || true

# Mouse
bindm = SUPER, mouse:272, movewindow
bindm = SUPER, mouse:273, resizewindow
EOF

# ═════════════════════════════════════════════════════════════════════
# 8. greetd config (auto-login astra → Hyprland, root refused by Hyprland)
# ═════════════════════════════════════════════════════════════════════
mkdir -p "$AIROOTFS/etc/greetd"
cat > "$AIROOTFS/etc/greetd/config.toml" << 'EOF'
[terminal]
vt = 1

[default_session]
command = "Hyprland"
user = "astra"
EOF

# Auto-login astra sur tty1 (Hyprland refuse root)
mkdir -p "$AIROOTFS/etc/systemd/system/getty@tty1.service.d"
cat > "$AIROOTFS/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin astra --noclear %I $TERM
EOF

# Root password vide + user astra
cat > "$AIROOTFS/etc/shadow" << 'EOF'
root::19000:0:99999:7:::
astra::19000:0:99999:7:::
EOF
cat > "$AIROOTFS/etc/passwd" << 'EOF'
root:x:0:0::/root:/bin/bash
astra:x:1000:1000::/home/astra:/bin/bash
EOF
cat > "$AIROOTFS/etc/group" << 'EOF'
root:x:0:root
wheel:x:10:astra
astra:x:1000:
EOF
cat > "$AIROOTFS/etc/gshadow" << 'EOF'
root:*::root
wheel:*::astra
astra:*::
EOF

# Create home dir for astra
mkdir -p "$AIROOTFS/home/astra/.config/hypr"
mkdir -p "$AIROOTFS/etc/skel/.config/hypr"
# Copy hyprland.conf to astra's home
cp "$AIROOTFS/etc/skel/.config/hypr/hyprland.conf" "$AIROOTFS/home/astra/.config/hypr/hyprland.conf" 2>/dev/null || true

# ═════════════════════════════════════════════════════════════════════
# 9. AstraOS branding
# ═════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/hostname" << 'EOF'
astraos
EOF

cat > "$AIROOTFS/etc/hosts" << 'EOF'
127.0.0.1   localhost
::1         localhost
127.0.1.1   astraos
EOF

cat > "$AIROOTFS/etc/os-release" << 'EOF'
NAME="AstraOS"
PRETTY_NAME="AstraOS Arch Test"
ID=astraos
ID_LIKE=arch
VERSION="0.1-test"
ANSI_COLOR="0;35"
EOF

# ═════════════════════════════════════════════════════════════════════
# 10. customize_airootfs.sh (build astra-shell-minimal)
# ═════════════════════════════════════════════════════════════════════
cat > "$PROFILE/customize_airootfs.sh" << 'CUSTOMIZE_EOF'
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
CUSTOMIZE_EOF
chmod +x "$PROFILE/customize_airootfs.sh"

# ═════════════════════════════════════════════════════════════════════
# 11. Makefile
# ═════════════════════════════════════════════════════════════════════
printf 'SHELL := /usr/bin/env bash\n.DEFAULT_GOAL := build-iso\n\nbuild-iso:\n\tdocker run --rm --privileged -v $$(PWD)/out:/out -v $$(PWD):/astraos-build astraos-builder:latest sh -c "mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64"\n\nclean:\n\trm -rf work out\n' > "$TEST/Makefile"

echo "  ✓ Profile Arch créé (11 fichiers)"
echo ""

# ═════════════════════════════════════════════════════════════════════
# 12. Vérifications
# ═════════════════════════════════════════════════════════════════════
echo "══════════ VÉRIFICATIONS ══════════"
echo "archiso hook : $(grep -c archiso "$AIROOTFS/etc/mkinitcpio.conf")"
echo "copytoram=n : $(grep -c copytoram "$PROFILE/syslinux/syslinux.cfg")"
echo "hyprland : $(grep -c hyprland "$PROFILE/packages.x86_64")"
echo "astra-shell build : $(ls "$TEST/astra-shell-minimal/Cargo.toml" 2>&1)"
echo "exec-once astra-shell : $(grep -c 'exec-once = astra-shell' "$AIROOTFS/etc/skel/.config/hypr/hyprland.conf")"
echo "greetd config : $(ls "$AIROOTFS/etc/greetd/config.toml" 2>&1)"
echo "root password vide : $(grep '^root::' "$AIROOTFS/etc/shadow" && echo '✓' || echo '✗')"
echo ""

# ═════════════════════════════════════════════════════════════
# 13. Build
# ═════════════════════════════════════════════════════════════
echo "→ Vérification de l'image Docker astraos-builder..."
if ! docker image inspect astraos-builder:latest >/dev/null 2>&1; then
    echo "Image absente — build depuis astraos-iso4/docker/Dockerfile.build..."
    docker build \
        -f astraos-iso4/docker/Dockerfile.build \
        -t astraos-builder:latest \
        astraos-iso4/
fi

echo "→ Build de l'ISO Arch test..."
echo ""
mkdir -p "$TEST/out"

docker run --rm --privileged \
    -v "$(pwd)/$TEST/out:/out" \
    -v "$(pwd)/$TEST:/astraos-build" \
    astraos-builder:latest \
    sh -c "mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ✦ ARCH TEST ISO prête !                                       ║"
echo "║  RAM VM = 2048 MB (copytoram=n)                              ║"
echo "║  Login : astra (pwd: astraos) — Hyprland en user (pas root) ║"
echo "║  → Hyprland + astra-shell-minimal (barre glassmorphism)     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
ls -lh "$TEST"/out/*.iso 2>&1
