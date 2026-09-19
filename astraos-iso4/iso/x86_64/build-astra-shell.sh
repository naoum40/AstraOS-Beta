#!/usr/bin/env bash
# Build Astra Shell inside the chroot during ISO creation
set -e -u

echo "=== Building Astra Shell ==="

# Astra Shell source is in /usr/src/astra-shell (copied by customize script)
ASTRA_SHELL_SRC="/usr/src/astra-shell"

if [ ! -d "$ASTRA_SHELL_SRC" ]; then
    echo "WARN: Astra Shell source not found at $ASTRA_SHELL_SRC, skipping build"
    exit 0
fi

cd "$ASTRA_SHELL_SRC"

# Build with cargo (release mode)
echo "→ Building Astra Shell with cargo..."
cargo build --release

# Install binary to /usr/bin/
install -Dm755 target/release/astra-shell /usr/bin/astra-shell
echo "✓ Astra Shell installed to /usr/bin/astra-shell"

# Install CSS theme
mkdir -p /usr/share/astraos/themes
install -Dm644 assets/css/glassmorphism.css /usr/share/astraos/themes/glassmorphism.css
install -Dm644 assets/css/welcome.css /usr/share/astraos/themes/welcome.css
echo "✓ CSS themes installed to /usr/share/astraos/themes/"

# Install desktop entry
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
