#!/usr/bin/env bash
set -e

# Vérifie qu'on est dans le bon dossier
if [ ! -f iso/x86_64/pacman.conf ]; then
    echo "✗ Erreur : lance ce script depuis la racine de astraos-iso4/"
    exit 1
fi

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  AstraOS ISO 4 — Application des 4 fixes (audit GLM 5.2)     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── 🔴 BUG #10 (CRITIQUE — BUILD BLOCKER) : pacman.conf mirror fantôme ──
echo "→ BUG #10 : pacman.conf mirror fantôme..."
sed -i 's|^Server = file:///mnt/archbuild/repo$|Include = /etc/pacman.d/mirrorlist|' \
    iso/x86_64/pacman.conf
echo "  ✓ [core] et [extra] utilisent maintenant Include = /etc/pacman.d/mirrorlist"
echo ""

# ── 🔴 BUG #11 (CRITIQUE — ROBUSTESSE) : paru git clone sans || true ──
echo "→ BUG #11 : paru git clone non-fatal..."
sed -i \
    's#sudo -u paru-builder git clone https://aur.archlinux.org/paru-bin.git$#sudo -u paru-builder git clone https://aur.archlinux.org/paru-bin.git 2>\&1 || true#' \
    iso/x86_64/customize_airootfs.sh
echo "  ✓ paru git clone a maintenant 2>&1 || true"
echo ""

# ── 🟡 BUG #12 (MINEUR) : copie obsolète de customize ──
echo "→ BUG #12 : suppression copie obsolète customize..."
rm -f iso/x86_64/airootfs/root/customize_airootfs.sh
echo "  ✓ iso/x86_64/airootfs/root/customize_airootfs.sh supprimé"
echo ""

# ── 🟡 BUG #13 (MINEUR) : build-astra-shell.sh écrasait astra.desktop ──
echo "→ BUG #13 : réécriture build-astra-shell.sh (suppression heredoc)..."
cat > iso/x86_64/build-astra-shell.sh << 'ASTRA_SHELL_EOF'
#!/usr/bin/env bash
# Build Astra Shell inside the chroot during ISO creation
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

# NOTE: /usr/share/wayland-sessions/astra.desktop is NOT recreated here.
# It already exists in the airootfs with a more complete content
# (Name[fr], Comment[fr], X-GDM-SessionRegisters=true).
# Recreating it here would overwrite that better version.

echo "=== Astra Shell build complete ==="
ASTRA_SHELL_EOF
chmod +x iso/x86_64/build-astra-shell.sh
echo "  ✓ build-astra-shell.sh réécrit (heredoc astra.desktop supprimé)"
echo ""

# ── Vérifications ──
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Vérifications post-fix                                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"

echo ""
echo "── #10 : pacman.conf [core] + [extra] ──"
grep -A1 -E '^\[(core|extra)\]' iso/x86_64/pacman.conf

echo ""
echo "── #11 : paru git clone (doit avoir || true) ──"
grep 'paru-bin.git' iso/x86_64/customize_airootfs.sh

echo ""
echo "── #12 : copie obsolète (doit afficher 'No such file') ──"
ls iso/x86_64/airootfs/root/customize_airootfs.sh 2>&1

echo ""
echo "── #13 : build-astra-shell.sh (0 = OK) ──"
grep -c 'cat > /usr/share/wayland-sessions' iso/x86_64/build-astra-shell.sh || true

echo ""

# ── Commit + push ──
git add -A
git commit -m "fix(iso4-build): pacman.conf mirror + paru robustness + cleanup

- BUG #10 (CRITIQUE, build blocker): pacman.conf pointait vers
  file:///mnt/archbuild/repo (mirror fantôme inexistant dans le Dockerfile.build).
  mkarchiso failait à pacman -Sy dans le chroot. Remplacé par
  Include = /etc/pacman.d/mirrorlist pour [core] et [extra].

- BUG #11 (robustesse): sudo -u paru-builder git clone... n'avait pas de
  || true. Avec set -e, un AUR down crashait le build. Ajouté 2>&1 || true.

- BUG #12 (maintenance): supprimé airootfs/root/customize_airootfs.sh
  (copie stale sans les 9 fixes, source de confusion).

- BUG #13 (doublon): build-astra-shell.sh recréait astra.desktop via heredoc
  avec un contenu minimal, écrasant le fichier airootfs plus complet.
  Heredoc supprimé.

Audit : GLM 5.2 — Task ID 72-audit-deep-iso4-build-pipeline"

echo ""
git push

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ✦ Fixes pushés sur GitHub.                                  ║"
echo "║  Prochaine étape : build sur Github                          ║"
echo "║    cd astraos-iso4 && make build-iso pack=core               ║"
echo "╚══════════════════════════════════════════════════════════════╝"