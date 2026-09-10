#!/usr/bin/env python3
"""
Génère un script bash unique qui crée toute l'arborescence AstraOS ISO 1.
Usage: python3 generate-installer.py > install-astraos-iso1.sh
"""
import os
import sys

ASTRA_ROOT = "/home/z/my-project/astraos"

TEXT_FILES = [
    ".devcontainer/devcontainer.json",
    ".gitignore",
    "LICENSE",
    "Makefile",
    "README.md",
    "astra-core/astra.conf",
    "astra-core/hyprland.conf",
    "docs/ARCHITECTURE.md",
    "docs/BUILD.md",
    "docs/ROADMAP.md",
    "iso/x86_64/airootfs/boot/grub/themes/astraos/README.md",
    "iso/x86_64/airootfs/boot/grub/themes/astraos/theme.txt",
    "iso/x86_64/airootfs/etc/astra/boot-banner",
    "iso/x86_64/airootfs/etc/astra/release-info",
    "iso/x86_64/airootfs/etc/astra/version",
    "iso/x86_64/airootfs/etc/default/grub",
    "iso/x86_64/airootfs/etc/fstab",
    "iso/x86_64/airootfs/etc/greetd/config.toml",
    "iso/x86_64/airootfs/etc/group",
    "iso/x86_64/airootfs/etc/gshadow",
    "iso/x86_64/airootfs/etc/hostname",
    "iso/x86_64/airootfs/etc/hosts",
    "iso/x86_64/airootfs/etc/locale.conf",
    "iso/x86_64/airootfs/etc/locale.gen",
    "iso/x86_64/airootfs/etc/mkinitcpio.conf",
    "iso/x86_64/airootfs/etc/passwd",
    "iso/x86_64/airootfs/etc/shadow",
    "iso/x86_64/airootfs/etc/skel/.bashrc",
    "iso/x86_64/airootfs/etc/skel/.config/astra/desktop.conf",
    "iso/x86_64/airootfs/etc/skel/.config/astra/version",
    "iso/x86_64/airootfs/etc/skel/.config/hypr/astra.conf",
    "iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf",
    "iso/x86_64/airootfs/etc/skel/.config/kitty/kitty.conf",
    "iso/x86_64/airootfs/etc/skel/.config/starship.toml",
    "iso/x86_64/airootfs/etc/sudoers.d/10-astraos",
    "iso/x86_64/airootfs/etc/vconsole.conf",
    "iso/x86_64/airootfs/root/.bash_profile",
    "iso/x86_64/airootfs/root/.bashrc",
    "iso/x86_64/airootfs/root/customize_airootfs.sh",
    "iso/x86_64/boot/loaders/entries/astraos-fallback.conf",
    "iso/x86_64/boot/loaders/entries/astraos.conf",
    "iso/x86_64/boot/loaders/loader.conf",
    "iso/x86_64/boot/syslinux/syslinux.cfg",
    "iso/x86_64/boot/syslinux/theme.cfg",
    "iso/x86_64/customize_airootfs.sh",
    "iso/x86_64/efiboot/loader/entries/astraos.conf",
    "iso/x86_64/efiboot/loader/loader.conf",
    "iso/x86_64/packages.x86_64",
    "iso/x86_64/pacman.conf",
    "iso/x86_64/profiledef.sh",
    "scripts/copy-assets.sh",
]

DIRS = set()
for f in TEXT_FILES:
    d = os.path.dirname(f)
    if d:
        DIRS.add(d)

ASSET_DIRS = [
    "assets/logos",
    "assets/wallpapers",
    "assets/fonts",
    "iso/x86_64/airootfs/usr/share/backgrounds/astraos",
    "iso/x86_64/airootfs/usr/share/pixmaps/astraos",
    "iso/x86_64/airootfs/usr/share/icons/astraos",
]

# Fichiers à rendre exécutables
EXECUTABLES = [
    "iso/x86_64/profiledef.sh",
    "iso/x86_64/customize_airootfs.sh",
    "iso/x86_64/airootfs/root/customize_airootfs.sh",
    "scripts/copy-assets.sh",
]

MARKER = "ASTRAOS_EOF_MARKER_42_ZAI"


def main():
    out = []
    out.append("#!/usr/bin/env bash")
    out.append("# AstraOS ISO 1 (v0.1.0) — Installer script")
    out.append("# Generated automatically by Z.ai Code")
    out.append("# Usage: bash install-astraos-iso1.sh")
    out.append("")
    out.append("set -e -u")
    out.append("")
    out.append('echo "╔══════════════════════════════════════════════════════════════╗"')
    out.append('echo "║          AstraOS ISO 1 (v0.1.0) — Installer                 ║"')
    out.append('echo "║          Arch Linux-based OS for general public            ║"')
    out.append('echo "║          by Astra Corporation (mmtstudio)                  ║"')
    out.append('echo "╚══════════════════════════════════════════════════════════════╝"')
    out.append("")
    out.append("ASTRA_ROOT=\"${1:-astraos}\"")
    out.append('echo "→ Creating AstraOS project in: $ASTRA_ROOT"')
    out.append("mkdir -p \"$ASTRA_ROOT\"")
    out.append("cd \"$ASTRA_ROOT\"")
    out.append("")
    out.append('echo "→ Creating directory structure..."')
    # Créer tous les dossiers
    all_dirs = sorted(DIRS | set(ASSET_DIRS))
    for d in all_dirs:
        out.append(f'mkdir -p "{d}"')
    out.append("")
    out.append('echo "→ Creating files..."')
    out.append("")

    # Créer chaque fichier texte avec heredoc quoté
    for filepath in TEXT_FILES:
        full_path = os.path.join(ASTRA_ROOT, filepath)
        try:
            with open(full_path, "r", encoding="utf-8") as f:
                content = f.read()
        except Exception as e:
            print(f"WARN: cannot read {full_path}: {e}", file=sys.stderr)
            continue

        # Vérifier que le marker n'est pas déjà dans le contenu
        if MARKER in content:
            # Si oui, on utilise un autre marker
            alt_marker = MARKER + "_ALT"
            while alt_marker in content:
                alt_marker += "X"
            marker = alt_marker
        else:
            marker = MARKER

        out.append(f'echo "  → {filepath}"')
        out.append(f"cat << '{marker}' > \"{filepath}\"")
        out.append(content)
        out.append(marker)
        out.append("")

    # Rendre les fichiers exécutables
    out.append('echo "→ Setting executable permissions..."')
    for exe in EXECUTABLES:
        out.append(f'chmod +x "{exe}"')
    out.append("")

    # Note pour les assets binaires
    out.append('echo ""')
    out.append('echo "╔══════════════════════════════════════════════════════════════╗"')
    out.append('echo "║  ⚠️  ACTION REQUISE:Uploader les assets binaires            ║"')
    out.append('echo "╚══════════════════════════════════════════════════════════════╝"')
    out.append('echo ""')
    out.append('echo "Les fichiers images (logos + wallpapers) ne sont pas inclus."')
    out.append('echo "Vous devez les uploader manuellement via VS Code (drag&drop)."')
    out.append('echo ""')
    out.append('echo "Fichiers à uploader dans assets/logos/:"')
    out.append('echo "  - logo-astraos.png  (logo officiel AstraOS)"')
    out.append('echo "  - logo-boot.png     (logo boot/splash)"')
    out.append('echo ""')
    out.append('echo "Fichiers à uploader dans assets/wallpapers/:"')
    out.append('echo "  - default-violet.png"')
    out.append('echo "  - default-bleu.png"')
    out.append('echo "  - classic.png"')
    out.append('echo ""')
    out.append('echo "Puis exécuter: bash scripts/copy-assets.sh"')
    out.append('echo ""')
    out.append('echo "✅ AstraOS ISO 1 structure created successfully!"')
    out.append('echo ""')
    out.append('echo "Next steps:"')
    out.append('echo "  1. Upload image assets (logos + wallpapers)"')
    out.append('echo "  2. Run: bash scripts/copy-assets.sh"')
    out.append('echo "  3. Build ISO: make build-iso"')
    out.append('echo ""')
    out.append('echo "🎯 AstraOS project ready at: $(pwd)"')

    print("\n".join(out))


if __name__ == "__main__":
    main()
