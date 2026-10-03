# Auto-launch Hyprland sur tty1 (après autologin astra)
# Hyprland refuse root, donc on le lance en user astra
if [[ -z "$DISPLAY" && "$XDG_VTNR" -eq 1 ]]; then
    exec Hyprland
fi
