# AstraOS GRUB Theme

Minimal dark GRUB theme with magenta accent, matching the AstraOS visual
identity (magenta star logo + deep space background + soft slate text).

## Status

**ISO 1 (v0.1):** This theme is **NOT shipped in the live ISO**. The live ISO
boots via syslinux (BIOS) or systemd-boot (UEFI), not GRUB. The `theme.txt`
file lives here inside `airootfs/boot/grub/themes/astraos/` so that when a
user installs AstraOS to disk via Calamares, the post-install hook can
produce a properly themed GRUB boot menu.

## Required assets (installed at install time)

The following binary asset files are NOT committed to this repo — they will
be copied by `customize_airootfs.sh` (or by a post-install hook in Calamares):

| Asset                  | Source                                          |
|------------------------|-------------------------------------------------|
| `background.png`       | `assets/wallpapers/default-violet.png` (scaled) |
| `highlight_*.png`      | generated from AstraOS magenta palette (#E040FB)|
| `slider_*.png`         | generated from AstraOS magenta palette          |
| `menu_*.png`           | generated from AstraOS dark panel palette        |

Fonts (`DejaVu Sans`, `DejaVu Sans Bold`) are shipped by the `grub` package
itself and don't need to be added here.

## Install-time directive

Suggested snippet for `customize_airootfs.sh` or Makefile:

```sh
THEME_DIR="${AIROOTFS}/boot/grub/themes/astraos"
install -Dm0644 astraos/assets/wallpapers/default-violet.png "${THEME_DIR}/background.png"
# Generate highlight_*.png / slider_*.png / menu_*.png from magenta swatches
# (left as a task for the build pipeline — see makefile target `grub-theme-assets`)
```

## Color reference (AstraOS palette)

| Token              | Hex       | Usage                         |
|--------------------|-----------|-------------------------------|
| Magenta accent     | `#E040FB` | titles, selected items        |
| Deep space bg      | `#0b0d17` | desktop color                 |
| Soft slate text    | `#f1f5f9` | unselected menu items         |
| Muted slate        | `#94a3b8` | hint / progress text          |
| Dim slate          | `#475569` | footer / copyright            |
