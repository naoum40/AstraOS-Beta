if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
    exec Hyprland
fi
