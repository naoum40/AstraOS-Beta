#!/usr/bin/env bash
# Build all Astra custom apps during ISO creation
# Runs INSIDE the chroot during archiso build.
set -e -u

echo "=== Building Astra custom apps ==="

APPS_SRC="/usr/src/astra-apps"
APPS=(
    "astra-settings"
    "astra-screenshot"
    "astra-photos"
    "astra-pass"
    "astra-widgets"
    "astra-sticky-notes"
    "astra-minesweeper"
    "astra-tictactoe"
    "astra-snake"
)

for app in "${APPS[@]}"; do
    app_dir="$APPS_SRC/$app"
    if [ -d "$app_dir" ]; then
        echo "→ Building $app..."
        cd "$app_dir"
        cargo build --release
        install -Dm755 "target/release/$app" "/usr/bin/$app"
        echo "✓ $app installed to /usr/bin/$app"
    else
        echo "WARN: $app source not found at $app_dir, skipping"
    fi
done

# Install desktop entries
mkdir -p /usr/share/applications
for app in "${APPS[@]}"; do
    if [ -f "$APPS_SRC/$app/$app.desktop" ]; then
        install -Dm644 "$APPS_SRC/$app/$app.desktop" "/usr/share/applications/$app.desktop"
    fi
done

echo "=== Astra apps build complete ==="
