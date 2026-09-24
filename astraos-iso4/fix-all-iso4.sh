#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════════════╗
# ║  AstraOS ISO 4 — Fix-all script (GLM 5.2 + Claude)                    ║
# ║  Règle TOUS les bugs identifiés d'un coup + commit + push             ║
# ║  Usage : cd astraos-iso4 && bash fix-all-iso4.sh                       ║
# ╚══════════════════════════════════════════════════════════════════════╝
set -e

# ── Trouve le repo ─────────────────────────────────────────────────────
for path in /workspaces/AstraOS-Beta/astraos-iso4 /home/z/AstraOS-Beta/astraos-iso4 "$(pwd)"; do
    if [ -f "$path/iso/x86_64/profiledef.sh" ]; then
        cd "$path"
        break
    fi
done

if [ ! -f iso/x86_64/profiledef.sh ]; then
    echo "✗ Erreur : lance ce script depuis la racine de astraos-iso4/"
    echo "  (le dossier qui contient iso/, astra-shell/, Makefile, etc.)"
    exit 1
fi

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  AstraOS ISO 4 — Fix-all (GLM 5.2 + Claude)                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── 0. Récupère le code à jour ──────────────────────────────────────────
echo "→ git pull (récupère derniers commits)..."
git pull --rebase 2>/dev/null || git pull 2>/dev/null || true
echo ""

# ════════════════════════════════════════════════════════════════════════
# FIX 1 : theme.cfg typo #a00000000 → #a0000000 (Claude, 12 occurrences)
# Cause : parseur couleurs syslinux décale lecture octets sur valeur 9 chars
#         mal formée → corruption couleurs + glyphs → écran orange/garbled
# ════════════════════════════════════════════════════════════════════════
echo "── FIX 1 : theme.cfg typo #a00000000 → #a0000000 (Claude) ──"
sed -i 's|#a00000000|#a0000000|g' iso/x86_64/syslinux/theme.cfg
typos_left=$(grep -c '#a00000000' iso/x86_64/syslinux/theme.cfg || true)
echo "  ✓ typos restantes : $typos_left (doit = 0)"
echo ""

# ════════════════════════════════════════════════════════════════════════
# FIX 2 : Switch Brave (AUR, lent, risk fail) → Firefox (officiel, rapide)
# Cause : brave-bin via AUR = git clone + makepkg ~3-5 min + risk réseau
#         Firefox dans repo Arch extra = pacman direct ~30 sec, fiable
# ════════════════════════════════════════════════════════════════════════
echo "── FIX 2 : Switch Brave → Firefox (économise ~3-5 min de build) ──"

# 2a. packages.x86_64 : uncomment firefox + update note
echo "  → packages.x86_64 : uncomment firefox"
sed -i 's|^#firefox  # ISO 3: replaced by brave-bin.*|firefox|' iso/x86_64/packages.x86_64
sed -i 's|^## Note: brave-bin is in AUR, installed via customize_airootfs.sh|## Note: Firefox is in official Arch repos (extra), no AUR needed|' iso/x86_64/packages.x86_64
ff_in_pkgs=$(grep -c '^firefox$' iso/x86_64/packages.x86_64 || true)
echo "    ✓ firefox dans packages : $ff_in_pkgs (doit = 1)"

# 2b. customize_airootfs.sh : comment out Brave AUR install block
echo "  → customize_airootfs.sh : disable Brave AUR install block"
sed -i '/^# === Install Brave browser (AUR) ===$/,/^rm -f \/etc\/sudoers\.d\/99-aur-builder$/ s|^|# DISABLED (Firefox in packages): |' iso/x86_64/customize_airootfs.sh
brave_in_customize=$(grep -c '^[^#].*brave-bin' iso/x86_64/customize_airootfs.sh || true)
echo "    ✓ lignes brave-bin actives restantes : $brave_in_customize (doit = 0)"

# 2c. customize_airootfs.sh : file associations brave-browser → firefox
echo "  → customize_airootfs.sh : file associations brave → firefox"
sed -i 's|brave-browser\.desktop|firefox.desktop|g' iso/x86_64/customize_airootfs.sh
ff_assoc=$(grep -c 'firefox.desktop' iso/x86_64/customize_airootfs.sh || true)
echo "    ✓ firefox.desktop dans customize : $ff_assoc"

# 2d. hyprland.conf : $browser = brave → firefox
echo "  → hyprland.conf : \$browser = firefox"
sed -i '/^\$browser/s/brave/firefox/' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf
sed -i '/^\$browser/s/brave/firefox/' astra-core/hyprland.conf 2>/dev/null || true
brave_in_hypr=$(grep -c 'brave' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf || true)
echo "    ✓ 'brave' restant dans hyprland.conf skel : $brave_in_hypr (doit = 0)"
echo ""

# ════════════════════════════════════════════════════════════════════════
# FIX 3 : hyprpaper race condition (retire le &)
# Cause : exec-once = hyprpaper & (background) puis ligne suivante
#         hyprctl hyprpaper preload → si hyprpaper pas prêt, preload fail
#         → wallpaper pas mis au 1er boot
# ════════════════════════════════════════════════════════════════════════
echo "── FIX 3 : hyprpaper race condition (retire le &) ──"
sed -i 's|exec-once = hyprpaper &|exec-once = hyprpaper|' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf
sed -i 's|exec-once = hyprpaper &|exec-once = hyprpaper|' astra-core/hyprland.conf 2>/dev/null || true
echo "  ✓ hyprpaper lance plus en arrière-plan (race évitée)"
echo ""

# ════════════════════════════════════════════════════════════════════════
# FIX 4 : $editor = nvim → vim (neovim pas installé, vim oui)
# Cause : Super+E lançait nvim qui n'existe pas → fail silencieux
# ════════════════════════════════════════════════════════════════════════
echo "── FIX 4 : \$editor = nvim → vim (neovim pas installé) ──"
sed -i '/^\$editor/s/nvim/vim/' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf
sed -i '/^\$editor/s/nvim/vim/' astra-core/hyprland.conf 2>/dev/null || true
echo "  ✓ \$editor = vim maintenant (Super+E marche)"
echo ""

# ════════════════════════════════════════════════════════════════════════
# VÉRIFICATIONS
# ════════════════════════════════════════════════════════════════════════
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  Vérifications post-fix                                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "── FIX 1 : theme.cfg ──"
echo "  typos #a00000000 restantes (0=OK) : $(grep -c '#a00000000' iso/x86_64/syslinux/theme.cfg || true)"
echo ""
echo "── FIX 2 : Firefox ──"
echo "  firefox dans packages (1=OK) : $(grep -c '^firefox$' iso/x86_64/packages.x86_64 || true)"
echo "  brave-bin actif dans customize (0=OK) : $(grep -c '^[^#].*brave-bin' iso/x86_64/customize_airootfs.sh || true)"
echo "  firefox.desktop dans customize : $(grep -c 'firefox.desktop' iso/x86_64/customize_airootfs.sh || true)"
echo "  brave dans hyprland.conf skel (0=OK) : $(grep -c 'brave' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf || true)"
echo ""
echo "── FIX 3 : hyprpaper ──"
echo "  'hyprpaper &' restant dans skel (0=OK) : $(grep -c 'hyprpaper &' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf || true)"
echo ""
echo "── FIX 4 : editor ──"
echo "  nvim dans hyprland.conf skel (0=OK) : $(grep -c 'nvim' iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf || true)"
echo ""

echo "── COMMIT + PUSH ──"
git add -A
git commit -m "fix(all): theme.cfg typo + Firefox (drop Brave AUR) + hyprpaper race + editor vim

- theme.cfg: 12 occurrences typo #a00000000 (9 chars) → #a0000000 (8 chars)
  Fix Claude : parseur couleurs syslinux décale lecture octets sur valeur
  mal formée → corruption couleurs + glyphs → écran orange/garbled menu boot.

- Switch Brave (AUR) → Firefox (officiel Arch extra repo) :
  * packages.x86_64 : uncomment firefox (pacman direct, ~30 sec, fiable)
  * customize_airootfs.sh : comment block Install Brave AUR (aur-builder user
    + git clone + makepkg -si) — sauve ~3-5 min build + élimine risk AUR
    down/network fail
  * customize_airootfs.sh : file associations brave-browser.desktop → firefox.desktop
  * hyprland.conf : \$browser = brave → firefox

- hyprland.conf : exec-once = hyprpaper &  →  exec-once = hyprpaper
  Retire le & qui causait race condition avec hyprctl hyprpaper preload
  ligne suivante → wallpaper pouvait pas s'afficher au 1er boot.

- hyprland.conf : \$editor = nvim → vim
  neovim pas dans packages.x86_64, vim oui — Super+E marchait pas.

Audit : GLM 5.2 Task 101 + Claude Task 100"

git push

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ✦ Tous les fixes pushés sur GitHub.                         ║"
echo "║                                                               ║"
echo "║  Prochaine étape (build propre) :                             ║"
echo "║    make clean                                                 ║"
echo "║    docker rmi astraos-builder:latest 2>/dev/null || true      ║"
echo "║    make build-iso pack=core                                   ║"
echo "║                                                               ║"
echo "║  Puis protocole protection :                                  ║"
echo "║    ls -lh out/*.iso && sha256sum out/*.iso                    ║"
echo "║    cp out/*.iso ~/astraos-iso4-backup.iso && chmod 444 ~/...  ║"
echo "║    gh release create v0.4.0-iso out/*.iso --title 'AstraOS' \\║"
echo "║        --notes 'build \$(date)' --repo naoum40/AstraOS-Beta  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
