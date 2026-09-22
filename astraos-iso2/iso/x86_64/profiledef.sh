#!/hint: bash
#
# SPDX-License-Identifier: GPL-3.0-or-later

# shellcheck disable=SC2034

# AstraOS ISO 1 (v0.1) — archiso profile definition
# Base Arch Linux + Hyprland + AstraOS branding (minimum viable).
# Based on the official Arch Linux archiso template.

iso_name="astraos"
iso_label="ASTRAOS_$(date +%Y%m)"
iso_publisher="AstraOS Project <https://astraos.org>"
iso_application="AstraOS Live/Rescue Media"
iso_version="0.1.0"
install_dir="astraos"
buildmodes=("iso")
bootmodes=("bios.syslinux" "uefi.systemd-boot")
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  "/etc/shadow" "0:0:400"
  "/etc/gshadow" "0:0:400"
  "/root" "0:0:750"
  "/etc/sudoers.d" "0:0:750"
  "/etc/sudoers.d/10-astraos" "0:0:440"
)
