#!/usr/bin/env bash
set -e -u

echo "=== Building Astra Shell ==="

ASTRA_SHELL_SRC="/usr/src/astra-shell"

if [ ! -d "$ASTRA_SHELL_SRC" ]; then
    echo "WARN: Astra Shell source not found at $ASTRA_SHELL_SRC, skipping build"
    exit 0
fi

cd "$ASTRA_SHELL_SRC"

echo "→ Building Astra Shell with cargo..."
cargo build --release

install -Dm755 target/release/astra-shell /usr/bin/astra-shell
echo "✓ Astra Shell installed to /usr/bin/astra-shell"

mkdir -p /usr/share/astraos/themes
install -Dm644 assets/css/glassmorphism.css /usr/share/astraos/themes/glassmorphism.css
install -Dm644 assets/css/welcome.css /usr/share/astraos/themes/welcome.css
echo "✓ CSS themes installed to /usr/share/astraos/themes/"

mkdir -p /usr/share/wayland-sessions
cat > /usr/share/wayland-sessions/astra.desktop << EOF
[Desktop Entry]
Name=AstraOS
Comment=AstraOS Shell Session
Exec=astra-shell
Type=Application
DesktopNames=astraos
EOF
echo "✓ Wayland session entry created"

echo "=== Astra Shell build complete ==="
