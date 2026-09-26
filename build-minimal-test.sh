#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════════════╗
# ║  AstraOS MINIMAL TEST ISO  (v2 — COMPLETE, rien oublié)              ║
# ║  Crée un ISO minimal : fond bleu + "AstraOS" centré (ASCII plain)   ║
# ║  BUT : tester si ton wrapper QEMU peut booter UN archiso ISO         ║
# ║  Si ce minimal boot → wrapper OK, AstraOS ISO 4 a un bug spécifique  ║
# ║  Si ce minimal fail aussi → wrapper cassé                            ║
# ║  Usage : cd AstraOS-Beta && bash build-minimal-test.sh               ║
# ╚══════════════════════════════════════════════════════════════════════╝
set -e

# ── Trouve le repo ─────────────────────────────────────────────────────
for path in /workspaces/AstraOS-Beta /home/z/AstraOS-Beta "$(pwd)/.."; do
    if [ -d "$path/astraos-iso4" ]; then
        cd "$path"
        break
    fi
done

if [ ! -d astraos-iso4 ]; then
    echo "✗ Lance ce script depuis la racine de AstraOS-Beta (le dossier qui contient astraos-iso4/)"
    exit 1
fi

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  AstraOS MINIMAL TEST ISO v2 — COMPLETE                        ║"
echo "║  fond bleu + 'AstraOS' centré (ASCII plain, pas Unicode)      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── Crée la structure du profile minimal ────────────────────────────────
MINIMAL="astraos-minimal"
PROFILE="$MINIMAL/iso/x86_64"
AIROOTFS="$PROFILE/airootfs"

echo "→ Création du profile minimal complet..."
rm -rf "$MINIMAL"
mkdir -p "$PROFILE/syslinux" "$PROFILE/boot/loaders/entries" "$PROFILE/efiboot/loader/entries"
mkdir -p "$AIROOTFS/etc/systemd/system/getty@tty1.service.d"

# ═══════════════════════════════════════════════════════════════════════
# 1. profiledef.sh — archiso profile config
# ═══════════════════════════════════════════════════════════════════════
cat > "$PROFILE/profiledef.sh" << 'EOF'
#!/hint: bash
# AstraOS Minimal Test ISO profile
iso_name="astraos-minimal"
iso_label="ASTRAOS_MIN"
iso_publisher="AstraOS Test"
iso_application="AstraOS Minimal Test"
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

# ═══════════════════════════════════════════════════════════════════════
# 2. packages.x86_64 — minimal packages
# ═══════════════════════════════════════════════════════════════════════
cat > "$PROFILE/packages.x86_64" << 'EOF'
## AstraOS Minimal Test — packages minimum
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
EOF

# ═══════════════════════════════════════════════════════════════════════
# 3. pacman.conf — copie de iso4 (avec mirrorlist fix)
# ═══════════════════════════════════════════════════════════════════════
cp astraos-iso4/iso/x86_64/pacman.conf "$PROFILE/pacman.conf"

# ═══════════════════════════════════════════════════════════════════════
# 4. mkinitcpio.conf — avec archiso hook
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/mkinitcpio.conf" << 'EOF'
MODULES=()
BINARIES=()
FILES=()
HOOKS=(base udev modconf archiso kms keyboard keymap consolefont block filesystems fsck)
COMPRESSION="zstd"
EOF

# ═══════════════════════════════════════════════════════════════════════
# 5. syslinux.cfg — fond bleu via vt.color=0x1f (white on blue)
# ═══════════════════════════════════════════════════════════════════════
cat > "$PROFILE/syslinux/syslinux.cfg" << 'EOF'
DEFAULT astraos
PROMPT 0
TIMEOUT 30
UI menu.c32

MENU TITLE AstraOS Minimal Test

LABEL astraos
MENU LABEL AstraOS Minimal Test (x86_64)
LINUX /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
INITRD /%INSTALL_DIR%/boot/intel-ucode.img,/%INSTALL_DIR%/boot/amd-ucode.img,/%INSTALL_DIR%/boot/x86_64/initramfs-linux.img
APPEND archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL% vt.color=0x1f
EOF

# ═══════════════════════════════════════════════════════════════════════
# 6. theme.cfg — minimal (pas de couleurs custom → pas de bug typo)
# ═══════════════════════════════════════════════════════════════════════
cat > "$PROFILE/syslinux/theme.cfg" << 'EOF'
# Minimal theme — no custom colors (avoid theme.cfg bugs)
MENU WIDTH 78
MENU MARGIN 6
MENU ROWS 10
EOF

# ═══════════════════════════════════════════════════════════════════════
# 7. UEFI boot loaders (systemd-boot) — boot/ + efiboot/
# ═══════════════════════════════════════════════════════════════════════
UEFI_ENTRY='title   AstraOS Minimal Test (x86_64)
linux   /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
initrd  /%INSTALL_DIR%/boot/intel-ucode.img
initrd  /%INSTALL_DIR%/boot/amd-ucode.img
initrd  /%INSTALL_DIR%/boot/x86_64/initramfs-linux.img
options archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL% vt.color=0x1f'

UEFI_LOADER='default astraos
timeout 3
console-mode keep
editor no'

# boot/loaders/ (sur l'ISO 9660)
printf '%s\n' "$UEFI_ENTRY" > "$PROFILE/boot/loaders/entries/astraos.conf"
printf '%s\n' "$UEFI_LOADER" > "$PROFILE/boot/loaders/loader.conf"

# efiboot/loader/ (sur l'ESP FAT image — nécessaire pour uefi-x64.systemd-boot.esp)
printf '%s\n' "$UEFI_ENTRY" > "$PROFILE/efiboot/loader/entries/astraos.conf"
printf '%s\n' "$UEFI_LOADER" > "$PROFILE/efiboot/loader/loader.conf"

# ═══════════════════════════════════════════════════════════════════════
# 8. /etc/issue — "AstraOS" ASCII plain (PAS Unicode — marche sur console ASCII)
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/issue" << 'ISSUE_EOF'




    ___  _____ _____ _____    ____  __   __    _   _ _____ _____
   / _ \|  ___|  ___|  ___|  / ___| \ \ / /   | | | /  ___|  ___|
  / /_\ \ |_  | |_  | |_    / /      \ V /    | | | \ `--.| |_
  |  _  |  _| |  _| |  _|  | |   ___  \ /     | | | |`--. \  _|
  | | | | |   | |   | |    | |__| (_) | |     \ \_/ /\__/ / |
  \_| |_|_|   \_|   \_|     \____\___/\_|      \___/\____/\_|



                     Minimal Test Build v0.1

                Login: root (no password required)




ISSUE_EOF

# ═══════════════════════════════════════════════════════════════════════
# 9. Autologin root sur tty1
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/systemd/system/getty@tty1.service.d/autologin.conf" << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --noclear %I $TERM
EOF

# ═══════════════════════════════════════════════════════════════════════
# 10. /etc/hostname
# ═══════════════════════════════════════════════════════════════════════
echo "astraos-minimal" > "$AIROOTFS/etc/hostname"

# ═══════════════════════════════════════════════════════════════════════
# 11. /etc/hosts — localhost resolution
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/hosts" << 'EOF'
127.0.0.1   localhost
::1         localhost
127.0.1.1   astraos-minimal
EOF

# ═══════════════════════════════════════════════════════════════════════
# 12. /etc/vconsole.conf — keymap + font console (UTF-8)
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/vconsole.conf" << 'EOF'
KEYMAP=us
FONT=eurlatgr
EOF

# ═══════════════════════════════════════════════════════════════════════
# 13. /etc/locale.conf — locale par défaut
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/locale.conf" << 'EOF'
LANG=en_US.UTF-8
EOF

# ═══════════════════════════════════════════════════════════════════════
# 14. /etc/locale.gen — en_US.UTF-8 décommenté (pour locale-gen)
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/locale.gen" << 'EOF'
en_US.UTF-8 UTF-8
EOF

# ═══════════════════════════════════════════════════════════════════════
# 15. /etc/shadow — root sans mot de passe (pour autologin)
# ═══════════════════════════════════════════════════════════════════════
cat > "$AIROOTFS/etc/shadow" << 'EOF'
root::19000:0:99999:7:::
EOF

# ═══════════════════════════════════════════════════════════════════════
# 16. Makefile — comme astraos-iso4 (avec tabs pour make)
# ═══════════════════════════════════════════════════════════════════════
printf '%s\n' \
    '# AstraOS Minimal Test ISO — Makefile' \
    'SHELL := /usr/bin/env bash' \
    '.DEFAULT_GOAL := build-iso' \
    '' \
    'PACK     ?= core' \
    'IMAGE    := astraos-builder' \
    'PWD      := $(shell pwd)' \
    'ISO_NAME ?= astraos-minimal' \
    'OUT_DIR  := out' \
    'WORK_DIR := work' \
    '' \
    '.PHONY: help build-iso clean' \
    '' \
    'help:' \
    '   @echo "Targets:"' \
    '   @echo "  build-iso  - Build the minimal test ISO (default)"' \
    '   @echo "  clean      - Remove work/ and out/"' \
    '' \
    'build-iso:' \
    '   @printf "AstraOS Minimal - building test ISO\n"' \
    '   @rm -rf $(WORK_DIR) $(OUT_DIR)' \
    '   docker build \' \
    '       --build-arg PACK=$(PACK) \' \
    '       -f ../astraos-iso4/docker/Dockerfile.build \' \
    '       -t $(IMAGE):latest ../astraos-iso4' \
    '   @mkdir -p $(OUT_DIR)' \
    '   docker run --rm \' \
    '       --privileged \' \
    '       -e PACK=$(PACK) \' \
    '       -v $(PWD)/$(OUT_DIR):/out \' \
    '       -v $(PWD):/astraos-build \' \
    '       $(IMAGE):latest \' \
    '       sh -c "exec mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64"' \
    '   @printf "ISO ready -> $(OUT_DIR)/$(ISO_NAME).iso\n"' \
    '' \
    'clean:' \
    '   @printf "Cleaning...\n"' \
    '   rm -rf $(WORK_DIR) $(OUT_DIR)' \
    > "$MINIMAL/Makefile"

echo "  ✓ Profile minimal complet créé (16 fichiers)"
echo ""

# ── Check image Docker + build si manquante ────────────────────────────
if ! docker image inspect astraos-builder:latest >/dev/null 2>&1; then
    echo "→ Image astraos-builder:latest pas trouvée — build depuis astraos-iso4/docker/Dockerfile.build..."
    docker build \
        -f astraos-iso4/docker/Dockerfile.build \
        -t astraos-builder:latest \
        astraos-iso4/
    echo "  ✓ Image Docker prête"
else
    echo "✓ Image Docker astraos-builder:latest déjà cachée — skip docker build"
fi
echo ""

# ── Build l'ISO minimal ────────────────────────────────────────────────
echo "→ Build de l'ISO minimal..."
echo ""
mkdir -p "$MINIMAL/out"

docker run --rm --privileged \
    -v "$(pwd)/$MINIMAL/out:/out" \
    -v "$(pwd)/$MINIMAL:/astraos-build" \
    astraos-builder:latest \
    sh -c "mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ✦ ISO minimal buildée !                                        ║"
echo "║  Fichier : astraos-minimal/out/*.iso                           ║"
echo "║                                                                  ║"
echo "║  TEST dans ton wrapper QEMU :                                   ║"
echo "║    - Si ça boote (fond bleu + 'AstraOS' ASCII art)            ║"
echo "║      → wrapper OK, AstraOS ISO 4 a un bug spécifique            ║"
echo "║    - Si ça fail aussi (VFS panic)                               ║"
echo "║      → wrapper cassé, faut fix le wrapper                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
ls -lh "$MINIMAL"/out/*.iso 2>&1
