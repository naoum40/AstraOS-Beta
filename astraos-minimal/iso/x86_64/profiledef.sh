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
