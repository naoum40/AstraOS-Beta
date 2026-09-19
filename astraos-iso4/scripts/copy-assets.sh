#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════════════╗
# ║              AstraOS — Stage assets into the airootfs                 ║
# ║     Copies wallpapers + logos from assets/ into the archiso profile    ║
# ║            © 2026 AstraOS Project — Astra Corporation                  ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# Run from the repo root (or from inside the build container — paths below
# are relative to the repo root which is /astraos-build in the container).
#
#   bash scripts/copy-assets.sh
#
# Safe to re-run: it overwrites the destination files.

set -euo pipefail

# ── Resolve repo root ────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

ASSETS_DIR="${REPO_ROOT}/assets"
AIROOTFS="${REPO_ROOT}/iso/x86_64/airootfs"

WALLPAPERS_DEST="${AIROOTFS}/usr/share/backgrounds/astraos"
LOGOS_DEST="${AIROOTFS}/usr/share/pixmaps/astraos"

# ── Helpers ──────────────────────────────────────────────────────────────
log()  { printf '\033[35m✦\033[0m %s\n' "$*"; }
ok()   { printf '\033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '\033[33m!\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }

# ── Sanity checks ───────────────────────────────────────────────────────
[ -d "${ASSETS_DIR}" ] || die "assets/ not found at ${ASSETS_DIR}"
[ -d "${AIROOTFS}" ]   || die "airootfs/ not found at ${AIROOTFS}"

mkdir -p "${WALLPAPERS_DEST}" "${LOGOS_DEST}"

# ── Wallpapers ───────────────────────────────────────────────────────────
copy_wallpaper() {
    local src="$1" dest="$2"
    if [ -f "${src}" ]; then
        cp -f "${src}" "${dest}"
        ok "wallpaper  → $(realpath --relative-to="${REPO_ROOT}" "${dest}")"
    else
        warn "missing wallpaper: ${src}"
    fi
}

copy_wallpaper "${ASSETS_DIR}/wallpapers/default-violet.png" \
               "${WALLPAPERS_DEST}/default-violet.png"
copy_wallpaper "${ASSETS_DIR}/wallpapers/default-bleu.png"  \
               "${WALLPAPERS_DEST}/default-bleu.png"
copy_wallpaper "${ASSETS_DIR}/wallpapers/classic.png"      \
               "${WALLPAPERS_DEST}/classic.png"

# ── Logos ───────────────────────────────────────────────────────────────
copy_logo() {
    local src="$1" dest="$2"
    if [ -f "${src}" ]; then
        cp -f "${src}" "${dest}"
        ok "logo       → $(realpath --relative-to="${REPO_ROOT}" "${dest}")"
    else
        warn "missing logo: ${src}"
    fi
}

copy_logo "${ASSETS_DIR}/logos/logo-astraos.png" \
          "${LOGOS_DEST}/logo.png"
copy_logo "${ASSETS_DIR}/logos/logo-boot.png"   \
          "${LOGOS_DEST}/logo-boot.png"

# ── Summary ─────────────────────────────────────────────────────────────
log "AstraOS assets staged into airootfs:"
log "  ${WALLPAPERS_DEST#${AIROOTFS}/}  ($(ls -1 "${WALLPAPERS_DEST}" 2>/dev/null | wc -l) file(s))"
log "  ${LOGOS_DEST#${AIROOTFS}/}       ($(ls -1 "${LOGOS_DEST}"      2>/dev/null | wc -l) file(s))"
