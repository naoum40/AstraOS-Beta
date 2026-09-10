#!/usr/bin/env bash
# ╔══════════════════════════════════════════════════════════════════════╗
# ║              AstraOS ISO 2 — Build ISO Script                       ║
# ║     This script assembles the necessary files and directories for    ║
# ║                     creating the AstraOS ISO 2                      ║
# ║            © 2026 AstraOS Project — Astra Corporation                ║
# ╚══════════════════════════════════════════════════════════════════════╝

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

ISO_DIR="${REPO_ROOT}/iso/x86_64"
AIROOTFS="${ISO_DIR}/airootfs"
OUTPUT_ISO="${REPO_ROOT}/astraos-iso2.iso"

log()  { printf '\033[35m✦\033[0m %s\n' "$*"; }
ok()   { printf '\033[32m✓\033[0m %s\n' "$*"; }
warn() { printf '\033[33m!\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[31m✗\033[0m %s\n' "$*" >&2; exit 1; }

# Check if necessary directories exist
[ -d "${AIROOTFS}" ] || die "airootfs/ not found at ${AIROOTFS}"

# Copy assets
"${SCRIPT_DIR}/copy-assets.sh"

# Create ISO
log "Creating ISO image at ${OUTPUT_ISO}..."
genisoimage -o "${OUTPUT_ISO}" -R -J -V "AstraOS ISO 2" \
    -b isolinux/isolinux.bin -c isolinux/boot.cat \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    "${ISO_DIR}"

ok "ISO image created successfully: ${OUTPUT_ISO}"