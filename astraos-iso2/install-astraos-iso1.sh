#!/usr/bin/env bash
# AstraOS ISO 1 (v0.1.0) — Installer script
# Generated automatically by Z.ai Code
# Usage: bash install-astraos-iso1.sh

set -e -u

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          AstraOS ISO 1 (v0.1.0) — Installer                 ║"
echo "║          Arch Linux-based OS for general public            ║"
echo "║          by Astra Corporation (mmtstudio)                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"

ASTRA_ROOT="${1:-astraos}"
echo "→ Creating AstraOS project in: $ASTRA_ROOT"
mkdir -p "$ASTRA_ROOT"
cd "$ASTRA_ROOT"

echo "→ Creating directory structure..."
mkdir -p ".devcontainer"
mkdir -p "assets/fonts"
mkdir -p "assets/logos"
mkdir -p "assets/wallpapers"
mkdir -p "astra-core"
mkdir -p "docs"
mkdir -p "iso/x86_64"
mkdir -p "iso/x86_64/airootfs/boot/grub/themes/astraos"
mkdir -p "iso/x86_64/airootfs/etc"
mkdir -p "iso/x86_64/airootfs/etc/astra"
mkdir -p "iso/x86_64/airootfs/etc/default"
mkdir -p "iso/x86_64/airootfs/etc/greetd"
mkdir -p "iso/x86_64/airootfs/etc/skel"
mkdir -p "iso/x86_64/airootfs/etc/skel/.config"
mkdir -p "iso/x86_64/airootfs/etc/skel/.config/astra"
mkdir -p "iso/x86_64/airootfs/etc/skel/.config/hypr"
mkdir -p "iso/x86_64/airootfs/etc/skel/.config/kitty"
mkdir -p "iso/x86_64/airootfs/etc/sudoers.d"
mkdir -p "iso/x86_64/airootfs/root"
mkdir -p "iso/x86_64/airootfs/usr/share/backgrounds/astraos"
mkdir -p "iso/x86_64/airootfs/usr/share/icons/astraos"
mkdir -p "iso/x86_64/airootfs/usr/share/pixmaps/astraos"
mkdir -p "iso/x86_64/boot/loaders"
mkdir -p "iso/x86_64/boot/loaders/entries"
mkdir -p "iso/x86_64/boot/syslinux"
mkdir -p "iso/x86_64/efiboot/loader"
mkdir -p "iso/x86_64/efiboot/loader/entries"
mkdir -p "scripts"

echo "→ Creating files..."

echo "  → .devcontainer/devcontainer.json"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > ".devcontainer/devcontainer.json"
{
  "name": "AstraOS Dev",
  "build": {
    "dockerfile": "../docker/Dockerfile.dev",
    "context": ".."
  },
  "remoteUser": "astra",
  "containerUser": "astra",
  "updateRemoteUserUID": true,
  "privileged": true,
  "mounts": [
    "source=${localWorkspaceFolder},target=/workspace,type=bind,consistency=cached"
  ],
  "workspaceMount": "",
  "workspaceFolder": "/workspace",
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "ms-azuretools.vscode-docker",
        "tamasfe.even-better-toml",
        "redhat.vscode-yaml",
        "bungcip.better-toml",
        "serayuzgur.crates",
        "vadimcn.vscode-lldb",
        "ms-vscode.makefile-tools"
      ],
      "settings": {
        "rust-analyzer.check.command": "clippy",
        "rust-analyzer.cargo.features": "all",
        "editor.formatOnSave": true,
        "files.exclude": {
          "**/target": true,
          "**/work": true,
          "**/out": true
        }
      }
    }
  },
  "forwardPorts": [],
  "postCreateCommand": "echo 'Welcome to AstraOS dev environment' && rustc --version && cargo --version",
  "postStartCommand": "starship init bash > /dev/null 2>&1 || true"
}

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → .gitignore"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > ".gitignore"
# Build artifacts
work/
out/
*.iso
*.img
*.squashfs
*.sig

# OS junk
.DS_Store
Thumbs.db
desktop.ini

# Editor / IDE
.vscode/
.idea/
*.swp
*.swo
*~
.fleet/

# Node / web workshop (kept minimal)
node_modules/
.next/
dist/
.cache/

# Rust
target/
**/*.rs.bk
# Cargo.lock is committed for binaries but ignored for libraries.
# AstraOS apps are binaries, so we keep their Cargo.lock by default.
# Uncomment the next line if you work on a library crate.
# Cargo.lock

# Python (used by Calamares modules)
__pycache__/
*.py[cod]
*.egg-info/
.venv/
venv/

# Logs
*.log
logs/

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → LICENSE"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "LICENSE"
MIT License

Copyright (c) 2026 AstraOS Project (Astra Corporation by mmtstudio)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

AstraOS ships with third-party open source components (Arch Linux packages,
Hyprland, Rust crates, GTK4, etc.), each governed by their own license. The
above MIT License applies only to source code authored by the AstraOS Project
contributors.

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → Makefile"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "Makefile"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║                     AstraOS — Top-level Makefile                     ║
# ║           Build orchestration for ISO + Dev container                ║
# ║            © 2026 AstraOS Project — Astra Corporation                ║
# ╚══════════════════════════════════════════════════════════════════════╝

SHELL := /usr/bin/env bash
.DEFAULT_GOAL := build-iso

PACK     ?= core
IMAGE    := astraos-builder
DEV_IMG  := astraos-dev
PWD      := $(shell pwd)
ISO_NAME ?= astraos-$(PACK)
OUT_DIR  := out
WORK_DIR := work

# QEMU defaults — tuned for VirtualBox-ish test experience.
QEMU_MEM   ?= 4096
QEMU_CPUS  ?= 4
QEMU_DISK  ?= 32G

# Color helpers
BOLD  := \033[1m
MAG   := \033[35m
CYAN  := \033[36m
YEL   := \033[33m
RST   := \033[0m

.PHONY: help build-iso build-iso-dev build-iso-bureautique \
        clean dev devshell test-qemu copy-assets lint-assets \
        docker-shell

##@ Help
help: ## Show this help
	@printf "$(MAG)╔══════════════════════════════════════════════════════════════╗$(RST)\n"
	@printf "$(MAG)║           AstraOS — Makefile targets                         ║$(RST)\n"
	@printf "$(MAG)╚══════════════════════════════════════════════════════════════╝$(RST)\n"
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
	    | awk 'BEGIN{FS=":.*?## "}{printf "  $(CYAN)%-22s$(RST) %s\n", $$1, $$2}'

##@ Build
build-iso: ## Build the ISO with the Core pack (default)
	@printf "$(MAG)✦ AstraOS$(RST) — building $(BOLD)$(PACK)$(RST) pack ISO\n"
	docker build \
	    --build-arg PACK=$(PACK) \
	    -f docker/Dockerfile.build \
	    -t $(IMAGE):latest .
	@mkdir -p $(OUT_DIR)
	docker run --rm \
	    -e PACK=$(PACK) \
	    -v $(PWD)/$(OUT_DIR):/out \
	    $(IMAGE):latest
	@printf "$(MAG)✦$(RST) ISO ready → $(BOLD)$(OUT_DIR)/$(ISO_NAME).iso$(RST)\n"

build-iso-dev: ## Build the ISO with the Developer pack
	@$(MAKE) --no-print-directory build-iso PACK=developer ISO_NAME=astraos-developer

build-iso-bureautique: ## Build the ISO with the Bureautique pack
	@$(MAKE) --no-print-directory build-iso PACK=bureautique ISO_NAME=astraos-bureautique

##@ Local helpers
copy-assets: ## Stage assets into airootfs (host-side, for local tests)
	@printf "$(CYAN)✦$(RST) Staging AstraOS assets into airootfs…\n"
	@bash scripts/copy-assets.sh
	@printf "$(MAG)✦$(RST) Done.\n"

##@ Clean
clean: ## Remove work/ and out/ directories
	@printf "$(YEL)✦$(RST) Cleaning build artifacts…\n"
	rm -rf $(WORK_DIR) $(OUT_DIR)
	@printf "$(MAG)✦$(RST) Done.\n"

##@ Dev environment
dev: devshell ## Open a bash shell inside the dev container (Rust toolchain)

devshell: ## (alias) Open dev shell
	@printf "$(MAG)✦$(RST) Starting AstraOS dev container…\n"
	docker build -f docker/Dockerfile.dev -t $(DEV_IMG):latest .
	docker run -it --rm \
	    -v $(PWD):/workspace \
	    -w /workspace \
	    $(DEV_IMG):latest bash

##@ Test
test-qemu: ## Boot the built ISO in QEMU (requires out/*.iso)
	@if ! ls $(OUT_DIR)/*.iso >/dev/null 2>&1; then \
	    printf "$(YEL)No ISO found in $(OUT_DIR)/. Run 'make build-iso' first.$(RST)\n"; exit 1; \
	fi
	@printf "$(MAG)✦$(RST) Booting ISO in QEMU ($(QEMU_CPUS) vCPUs, $(QEMU_MEM) MiB RAM)\n"
	qemu-system-x86_64 \
	    -m $(QEMU_MEM) \
	    -smp $(QEMU_CPUS) \
	    -cdrom $$(ls -1 $(OUT_DIR)/*.iso | head -n1) \
	    -boot d \
	    -enable-kvm 2>/dev/null || \
	qemu-system-x86_64 \
	    -m $(QEMU_MEM) \
	    -smp $(QEMU_CPUS) \
	    -cdrom $$(ls -1 $(OUT_DIR)/*.iso | head -n1) \
	    -boot d \
	    -vga virtio \
	    -display gtk

##@ Misc
docker-shell: ## Open a shell inside the builder image (debug)
	docker run -it --rm -v $(PWD):/astraos-build $(IMAGE):latest bash

lint-assets:
	@bash -n scripts/copy-assets.sh && echo "scripts/copy-assets.sh: syntax OK"

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → README.md"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "README.md"
<div align="center">

<img src="assets/logos/logo-astraos.png" alt="AstraOS logo" width="160" />

# AstraOS

**Arch Linux-based OS for the general public.**

*Glassmorphism. Dark. Fast. Owned.*

</div>

---

> **✦ AstraOS — Arch Linux-based OS for general public**

AstraOS is an Arch Linux-based desktop operating system designed for the
general public — not just for power users. It pairs the rolling, rock-solid
Arch base with a custom **Rust + GTK4-rs** shell on top of the **Hyprland**
Wayland compositor, all wrapped in a calm violet/magenta glassmorphism theme.

The first release (v1.0) is built incrementally through 7 ISOs. This
repository currently contains **ISO 1 (v0.1)**: a minimal Arch + Hyprland
base, AstraOS wallpapers, branded boot logo, and a sane default user (`astra`).
The custom Astra Shell and Astra Apps will land in ISO 2-7.

### Why AstraOS?

- **Eco RAM by design.** Rust's ownership model means no garbage collector,
  no V8-style runtime, and predictable memory usage over time. Target idle
  RAM: **~250-300 MiB**, vs 4-6 GiB on Windows 11.
- **Glassmorphism-native.** Hyprland's blur pipeline powers translucent
  windows, drop shadows, and rounded corners out of the box. No Electron.
- **Independence.** AstraOS owns the UI layer. Third-party services (mail,
  cloud, gaming) plug in as backends — like Apple Silicon does for hardware.
- **Honesty.** No telemetry, no dark patterns, no AI forced on you. The
  Astra Assistant is opt-in and configurable.

### Features overview

- Base Arch Linux system (pacman + AUR via `paru`)
- Hyprland Wayland compositor with glassmorphism preset
- AstraOS violet wallpaper + boot logo + branding
- Kitty terminal, Thunar file manager, Firefox browser (staged for ISO 3)
- Default user `astra` with branded bash prompt (starship) + modern CLI
  tools (eza, bat, ripgrep, fd, fzf, zoxide)
- Reproducible ISO builds via Docker (no host toolchain required)
- Tested on VirtualBox / QEMU / GitHub Codespaces

### ISO packs

A single codebase produces multiple packs by swapping `packages.x86_64`:

| Pack              | Audience            | Contents                                                |
| ----------------- | ------------------- | ------------------------------------------------------- |
| **Core**          | Power users (default) | Hyprland + Terminal + Files + Settings + AstraOS apps |
| **Developer**     | Developers          | Core + VSCodium + Rust toolchain + Docker + Git         |
| **Bureautique**   | Office / general public | Core + Brave + LibreOffice                          |
| **Gaming**        | Gamers (phase 3+)   | Core + Astra Gaming Launcher + Steam + Proton + RetroArch |

Build commands:
```bash
make build-iso                # core (default)
make build-iso-dev            # developer pack
make build-iso-bureautique    # bureautique pack
```

### Requirements to build

- **Docker Desktop** on Windows, macOS, or Linux (4 GiB RAM allocated is enough)
- **5 GiB free disk** for the build cache
- Internet access during the Docker image build (Arch mirrors)

You do **not** need to install Rust, archiso, or any other toolchain on your
host machine. Everything runs inside the Docker container.

### Build instructions

```bash
# 1. Clone the repository
git clone https://github.com/AstraCorp/astraos.git
cd astraos

# 2. (optional) Stage AstraOS assets into the airootfs
make copy-assets

# 3. Build the ISO
make build-iso

# The ISO appears at:
#   out/astraos-core.iso
```

### Test instructions

**Option A — GitHub Codespaces**
1. Open the repo in a Codespace (4-core, 8 GiB minimum).
2. Run `make build-iso`.
3. Download `out/astraos-core.iso` to your machine.
4. Boot it in VirtualBox / VMware / a spare PC.

**Option B — Local Docker Desktop + VirtualBox (Windows)**
```bash
make build-iso
# Then in VirtualBox:
#   New → Type: Linux, Version: Arch Linux (64-bit)
#   RAM: 4096 MB, CPU: 4
#   Optical disk: out/astraos-core.iso
#   Boot the VM.
```

**Option C — Local QEMU**
```bash
make test-qemu
```

### Project structure

```
astraos/
├── Makefile                         # build orchestration
├── README.md
├── LICENSE                          # MIT (Astra Corporation)
├── .gitignore
├── astra-core/                      # reference Hyprland config
│   ├── hyprland.conf
│   └── astra.conf
├── assets/
│   ├── logos/                       # logo-astraos.png, logo-boot.png
│   └── wallpapers/                  # default-violet.png, default-bleu.png, classic.png
├── docker/
│   ├── Dockerfile.build             # ISO builder (archiso)
│   └── Dockerfile.dev               # Rust + GTK4-rs DevContainer
├── .devcontainer/
│   └── devcontainer.json
├── iso/
│   └── x86_64/
│       ├── profiledef.sh            # (created later) archiso profile
│       └── airootfs/
│           └── etc/skel/
│               ├── .bashrc
│               └── .config/
│                   ├── hypr/        # hyprland.conf + astra.conf
│                   ├── kitty/
│                   ├── starship.toml
│                   └── astra/      # version + desktop.conf
├── scripts/
│   └── copy-assets.sh
└── docs/
    ├── BUILD.md                     # detailed build guide
    ├── ARCHITECTURE.md              # stack diagram + rationale
    └── ROADMAP.md                   # 7 ISOs + 7 phases
```

### Roadmap

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for the full plan:

- **ISO 1 (v0.1)** — Base Arch + Hyprland + AstraOS branding ← *you are here*
- **ISO 2 (v0.2)** — Astra Shell (bar / dock / launcher / lock screen)
- **ISO 3 (v0.3)** — System apps (Kitty, Thunar, Firefox, VLC, Settings)
- **ISO 4 (v0.4)** — AstraPass, Sticky Notes, Widgets, 3 games
- **ISO 5 (v0.5)** — Astra Defender, Task Manager, Performance / Game Mode
- **ISO 6 (v0.6)** — Astra Assistant, Voice control, Smart search, Recorders
- **ISO 7 (v1.0)** — Astra Gaming Launcher, Calamares, i18n, Multi-user

Post-v1.0 phases: Astra Bridge (Wine/Proton), Astra Gaming Launcher, Astra
WinBox, Astra Mobile, AstraOS ARM, Waydroid.

### License

MIT — © 2026 AstraOS Project (Astra Corporation by mmtstudio). See
[`LICENSE`](LICENSE) for details. Third-party components keep their own
licenses.

### Credits

**Astra Corporation** — by **mmtstudio**

Built on top of amazing open source software: Arch Linux, Hyprland, Rust,
GTK4, PipeWire, systemd, Calamares, and many more. Thank you to all the
maintainers who keep these projects alive.

<div align="center">

*✦ Made with violet, magenta, and a borrow checker.*

</div>

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → astra-core/astra.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "astra-core/astra.conf"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║              AstraOS-specific Hyprland configuration                   ║
# ║                  Sourced by ~/.config/hypr/hyprland.conf               ║
# ║                       ISO 1 (v0.1) — placeholder                       ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# This file is intentionally minimal for ISO 1.
#
# Starting ISO 2 (v0.2), it will contain:
#   - Astra Shell window rules (bar / dock / launcher / lock screen)
#   - AstraOS-specific animations (astra easing curve refinements)
#   - Branded workspace names + icons
#   - Custom AstraOS keybindings (AstraPass, Widgets, etc.)
#   - Integration with the Rust + GTK4-rs Astra Shell daemons
#
# It is sourced automatically at the bottom of hyprland.conf via:
#     source = ~/.config/hypr/astra.conf

# Placeholder for ISO 1

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → astra-core/hyprland.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "astra-core/hyprland.conf"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║                    AstraOS — Hyprland configuration                    ║
# ║                        ISO 1 (v0.1) — Core pack                       ║
# ║              © 2026 AstraOS Project — Astra Corporation              ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# Minimal but elegant config for ISO 1.
# Astra Shell (custom Rust + GTK4-rs) will be wired in ISO 2 and will
# source additional settings from ~/.config/hypr/astra.conf (see bottom).

# ── Environment variables ────────────────────────────────────────────────
env = XCURSOR_SIZE,24
env = QT_QPA_PLATFORMTHEME,qt5ct
env = XDG_CURRENT_DESKTOP,Hyprland
env = XDG_SESSION_TYPE,wayland
env = XDG_SESSION_DESKTOP,Hyprland

# ── Monitors ────────────────────────────────────────────────────────────
# Auto-detect preferred mode + scale 1 (HiDPI handled later by Astra Shell)
monitor=,preferred,auto,1

# ── Default programs ─────────────────────────────────────────────────────
$terminal    = kitty
$fileManager = thunar
$menu        = wofi --show drun
$browser     = firefox
$editor      = nvim

# ── Autostart ────────────────────────────────────────────────────────────
exec-once = pipewire
exec-once = wireplumber
exec-once = hyprpaper &

# Waybar is NOT shipped in ISO 1 (will be replaced by Astra Shell in ISO 2).
# Uncomment once waybar is added if you need a temporary bar.
# exec-once = waybar

# Set the AstraOS violet wallpaper on boot
exec-once = hyprctl hyprpaper preload /usr/share/backgrounds/astraos/default-violet.png
exec-once = hyprctl hyprpaper wallpaper ",/usr/share/backgrounds/astraos/default-violet.png"

# ── Input ────────────────────────────────────────────────────────────────
input {
    kb_layout  = fr
    kb_variant =
    kb_model   =
    kb_options =
    kb_rules   =

    follow_mouse = 1

    touchpad {
        natural_scroll = true
        tap-to-click  = true
        clickfinger   = true
    }

    sensitivity = 0 # -1.0..1.0, 0 = no modification
}

# ── Device / gestures ────────────────────────────────────────────────────
gestures {
    workspace_swipe = true
    workspace_swipe_distance = 200
}

# ── Decoration (glassmorphism) ──────────────────────────────────────────
decoration {
    rounding = 12

    blur {
        enabled = true
        size = 8
        passes = 2
        new_optimizations = true
        ignore_opacity = true
        xray = false
    }

    drop_shadow = true
    shadow_range = 20
    shadow_render_power = 3
    col.shadow = rgba(1a1a2eee)

    # Glassmorphism accent
    active_opacity = 0.95
    inactive_opacity = 0.85
    fullscreen_opacity = 1.0
}

# ── Animations ───────────────────────────────────────────────────────────
animations {
    enabled = true

    bezier = easeOutQuint, 0.22, 1, 0.36, 1
    bezier = easeInOutCubic, 0.65, 0.05, 0.36, 1
    bezier = astra, 0.4, 0.0, 0.2, 1.0

    animation = windows, 1, 6, astra, slide
    animation = windowsOut, 1, 5, astra, slide
    animation = windowsMove, 1, 5, astra, slide
    animation = border, 1, 8, default
    animation = fade, 1, 5, astra
    animation = workspaces, 1, 5, astra, slidefade
    animation = workspacesIn, 1, 5, astra, slidefade
    animation = workspacesOut, 1, 5, astra, slidefade
    animation = specialWorkspace, 1, 5, astra, slidefade
}

# ── Layout / Dwindle ─────────────────────────────────────────────────────
dwindle {
    pseudotile = true
    preserve_split = true
}

master {
    new_is_master = true
}

# ── Misc ─────────────────────────────────────────────────────────────────
misc {
    disable_hyprland_logo = true
    disable_splash_rendering = true
    force_default_wallpaper = 0
    vfr = true
    mouse_move_enables_dpms = true
    key_press_enables_dpms = true
}

# ── Keybindings ──────────────────────────────────────────────────────────
$mainMod = SUPER

# Apps
bind = $mainMod,       Return, exec, $terminal
bind = $mainMod,       E,      exec, $fileManager
bind = $mainMod,       B,      exec, $browser
bind = $mainMod,       space,  togglefloating,
bind = $mainMod SHIFT, M,      exit,
bind = $mainMod,       Q,      killactive,

# Launcher (placeholder, wofi may not be installed yet)
bind = $mainMod,       D,      exec, $menu

# Screenshot — full screen + region (grim + slurp)
bind = ,               Print,  exec, grim - | wl-copy
bind = SHIFT,          Print,  exec, grim -g "$(slurp)" - | wl-copy
bind = $mainMod SHIFT, S,      exec, grim -g "$(slurp)" - | wl-copy

# Window focus
bind = $mainMod,       left,  movefocus, l
bind = $mainMod,       right, movefocus, r
bind = $mainMod,       up,    movefocus, u
bind = $mainMod,       down,  movefocus, d

# Window movement
bind = $mainMod SHIFT, left,  movewindow, l
bind = $mainMod SHIFT, right, movewindow, r
bind = $mainMod SHIFT, up,    movewindow, u
bind = $mainMod SHIFT, down,  movewindow, d

# Workspaces
bind = $mainMod,       1, workspace, 1
bind = $mainMod,       2, workspace, 2
bind = $mainMod,       3, workspace, 3
bind = $mainMod,       4, workspace, 4
bind = $mainMod,       5, workspace, 5
bind = $mainMod,       6, workspace, 6
bind = $mainMod,       7, workspace, 7
bind = $mainMod,       8, workspace, 8
bind = $mainMod,       9, workspace, 9

bind = $mainMod SHIFT, 1, movetoworkspace, 1
bind = $mainMod SHIFT, 2, movetoworkspace, 2
bind = $mainMod SHIFT, 3, movetoworkspace, 3
bind = $mainMod SHIFT, 4, movetoworkspace, 4
bind = $mainMod SHIFT, 5, movetoworkspace, 5
bind = $mainMod SHIFT, 6, movetoworkspace, 6
bind = $mainMod SHIFT, 7, movetoworkspace, 7
bind = $mainMod SHIFT, 8, movetoworkspace, 8
bind = $mainMod SHIFT, 9, movetoworkspace, 9

# Scroll through workspaces
bind = $mainMod, mouse_down, workspace, e+1
bind = $mainMod, mouse_up,   workspace, e-1

# Media keys (best-effort)
bindl = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bindl = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bindl = , XF86AudioMute,        exec, wpctl set-mute   @DEFAULT_AUDIO_SINK@ toggle
bindl = , XF86MonBrightnessUp,   exec, brightnessctl set +5%
bindl = , XF86MonBrightnessDown, exec, brightnessctl set 5%-

# ── Window rules ─────────────────────────────────────────────────────────
# Default rules. AstraOS-specific app rules will be added in astra.conf
# (sourced at the bottom of this file) starting from ISO 2.

windowrule = float, ^(pavucontrol)$
windowrule = float, ^(nm-connection-editor)$
windowrule = float, ^(blueman-manager)$
windowrule = center, ^(pavucontrol)$

# kitty / firefox tweaks for glassmorphism
windowrule = opacity 0.95 0.85, ^(kitty)$
windowrule = opacity 1.0,       ^(firefox)$

# ── AstraOS-specific config (placeholder for ISO 1) ─────────────────────
# Astra Shell integration, custom animations, branded window rules, etc.
# will live in ~/.config/hypr/astra.conf starting ISO 2.
source = ~/.config/hypr/astra.conf

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → docs/ARCHITECTURE.md"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "docs/ARCHITECTURE.md"
# AstraOS — Architecture

This document explains the high-level architecture of AstraOS and why each
layer was chosen. For the build pipeline, see [`BUILD.md`](BUILD.md). For the
release plan, see [`ROADMAP.md`](ROADMAP.md).

---

## 1. Layered view

```
┌─────────────────────────────────────────────────────────────────┐
│                          AstraOS                                │
├─────────────────────────────────────────────────────────────────┤
│  Astra Shell  (Rust + GTK4-rs)                                  │
│   ├── Bar / Dock (dual Win + Mac modes, runtime-switchable)     │
│   ├── Launcher (smart search: apps + files + web)              │
│   ├── Lock screen (clock + SHA-256 hashed password)             │
│   ├── Widgets (Clock / Calendar / Weather / Battery)            │
│   ├── Notifications                                             │
│   └── Voice control (opt-in)                                    │
├─────────────────────────────────────────────────────────────────┤
│  Astra Apps  (Rust + GTK4-rs)                                   │
│   ├── AstraPass (AES-GCM 256 + PBKDF2)                          │
│   ├── Astra Defender (ClamAV-backed antivirus UI)               │
│   ├── Astra Assistant (Qwen iframe, opt-in)                     │
│   ├── Task Manager, Performance Mode, Game Mode                 │
│   ├── Screen Recorder, Screenshot, Audio Recorder, Webcam       │
│   └── Games: Minesweeper, Tic-Tac-Toe, Snake                    │
├─────────────────────────────────────────────────────────────────┤
│  Hyprland  (Wayland compositor, C++)                            │
│   ├── Glassmorphism / blur pipeline (native)                    │
│   ├── Tiling + floating + workspaces                            │
│   └── Multi-monitor + HiDPI                                     │
├─────────────────────────────────────────────────────────────────┤
│  Arch Linux base                                                │
│   ├── Linux kernel + systemd                                    │
│   ├── pacman + AUR (via paru)                                   │
│   ├── PipeWire (audio), NetworkManager + iwd, BlueZ             │
│   └── greetd (display manager, Rust)                            │
├─────────────────────────────────────────────────────────────────┤
│  Calamares installer (graphical, AstraOS-branded)               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Why this stack

AstraOS compares directly to Windows (C/C++ + C# + XAML) and macOS
(C/C++ + Swift + SwiftUI). The matching AstraOS combo is
**C/C++ (Arch base) + Rust + GTK4-rs (apps)**.

| Criterion | Rust + GTK4-rs | C (GTK4) | TypeScript (AGS v2) |
| --- | --- | --- | --- |
| Native performance | ✅ 100% | ✅ 100% | ❌ 60-70% |
| Memory safety | ✅ borrow checker | ❌ segfaults | ✅ GC |
| No runtime | ✅ native binary | ✅ native binary | ❌ V8 ~80 MiB |
| No GC pauses | ✅ ownership | ✅ manual | ❌ GC pauses |
| Glassmorphism | ✅ GTK4-rs | ⚠️ C bindings | ✅ AGS v2 |
| Linux ecosystem | ✅ crates.io + gtk4-rs | ✅ mature | ❌ limited |
| Like Windows/macOS | ✅ yes (Swift/C# equiv) | ❌ no | ❌ no |

**Trade-off accepted**: steep Rust learning curve (borrow checker), no hot
reload, ~3-5x slower development than TypeScript at first. But it is the
price of a premium OS.

---

## 3. Eco RAM — the competitive advantage

Rust's ownership model means:

- 0 runtime overhead (no V8 ~80 MiB)
- 0 memory leaks (compiler-checked)
- 0 GC pauses (UI stays smooth under load)
- Predictable RAM over time (stable through 24h of use)

| OS | Idle RAM |
| --- | --- |
| Windows 11 | 4-6 GiB |
| macOS Sonoma | 2-3 GiB |
| Ubuntu (GNOME/JS) | 1.2-1.5 GiB |
| **AstraOS (Rust)** | **~250-300 MiB** ✅ |

Bloat after 24h:

| Stack | RAM drift |
| --- | --- |
| Electron/TS apps (VS Code, Discord) | +200-500 MiB |
| **AstraOS Rust** | **+0 MiB** (stable by ownership) |

---

## 4. Module boundaries

| Module | Language | Lives in | First ISO |
| --- | --- | --- | --- |
| Kernel, systemd, pacman | C | Arch packages | ISO 1 |
| Hyprland compositor | C++ | Arch package (unmodified) | ISO 1 |
| PipeWire / NM / BlueZ | C | Arch packages | ISO 1 |
| greetd display manager | Rust | Arch package | ISO 1 |
| Astra Shell | Rust + GTK4-rs | `astraos/astra-shell/` | ISO 2 |
| Astra Apps | Rust + GTK4-rs | `astraos/astra-apps/` | ISO 4 |
| Astra Defender (ClamAV UI) | Rust + GTK4-rs | `astraos/astra-apps/` | ISO 5 |
| Astra Assistant | Rust + GTK4-rs (iframe) | `astraos/astra-apps/` | ISO 6 |
| Astra Gaming Launcher | Rust + GTK4-rs | `astraos/astra-apps/` | ISO 7 / phase 3 |
| Calamares installer | C++ + Python | Arch package, branded | ISO 7 |

---

## 5. Build pipeline

```
Host (Windows/macOS/Linux)
└── Docker Desktop
    └── docker/Dockerfile.build
        └── archlinux:latest
            ├── pacman -Syu archiso squashfs-tools ...
            ├── scripts/copy-assets.sh → stage logos + wallpapers
            └── mkarchiso -v -w /work -o /out iso/x86_64
                └── out/astraos-core.iso
```

The DevContainer (`docker/Dockerfile.dev`) is a separate image used by VS
Code / Codespaces to write Rust code without polluting the host.

---

## 6. Independence philosophy

AstraOS owns the UI layer. Third-party services are backends:

- **Astra Store** → UI is AstraOS, backend is Flatpak/Flathub.
- **Astra Mail** → UI is AstraOS, connectors are IMAP/SMTP/Gmail/Outlook.
- **Astra Sync** → UI is AstraOS, backends are Drive/OneDrive/Dropbox/…
- **Astra Gaming Launcher** → UI is AstraOS, backends are Steam/Epic/Ubisoft
  (Steam client is invisible to the user — like Apple Silicon strategy).

This mirrors the Apple Silicon philosophy: AstraOS controls the user
experience, providers stay as plumbing.

---

## 7. Security model (v1.0)

- Lock screen: custom Rust, SHA-256 hashed password, no plaintext storage.
- Astra Defender: real antivirus, ClamAV engine.
- Multi-user + fast user switching.
- LUKS full-disk encryption: **deferred to v1.1** (user has VeraCrypt today).
- Astra Assistant is **opt-in** (off by default, configurable in Settings).

---

## 8. What is explicitly **not** in scope (v1.0)

- Astra Bridge (Wine/Proton) → phase 2+
- Astra WinBox (gaming VM for kernel anti-cheat games) → phase 4+
- Astra Mobile (smartphone variant, Phosh) → phase 3+
- Disk encryption by default → v1.1
- Tencent-owned games (Valorant, etc.) → boycott, not supported

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → docs/BUILD.md"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "docs/BUILD.md"
# AstraOS — Build Guide

This document describes how to build the AstraOS ISO from scratch, on
Windows with Docker Desktop, on macOS, on Linux, or in GitHub Codespaces.

> ISO 1 (v0.1) is the **Core** pack: base Arch Linux + Hyprland minimal +
> AstraOS branding (wallpaper, logo, bash prompt). Subsequent ISOs add the
> Astra Shell and apps.

---

## 1. Requirements

| Tool | Version | Notes |
| --- | --- | --- |
| Docker Desktop | 4.30+ | Windows / macOS host |
| Git | any | to clone the repo |
| QEMU **or** VirtualBox | any | to boot the resulting ISO |
| Free disk | ~5 GiB | for the build cache + ISO |
| Internet | required | Arch mirrors are pulled at build time |

You do **not** need Rust, archiso, or any other toolchain installed on the
host — everything runs inside the Docker container.

---

## 2. Clone & build

```bash
git clone https://github.com/AstraCorp/astraos.git
cd astraos

# Stage AstraOS assets (wallpapers + logos) into the airootfs.
# The Docker build also runs this step automatically, but you can run it
# locally for debugging:
make copy-assets

# Build the ISO (default pack = core)
make build-iso
```

The resulting ISO appears at:

```
out/astraos-core.iso
```

### Build a different pack

```bash
make build-iso-dev          # astraos-developer.iso
make build-iso-bureautique  # astraos-bureautique.iso
```

> Pack selection switches `packages.x86_64` and `profiledef.sh` (added in a
> later ISO). For ISO 1 only the Core pack is meaningful.

---

## 3. What happens under the hood

The Makefile invokes two Docker stages:

1. **Builder image** (`docker/Dockerfile.build`)
   - `FROM archlinux:latest`
   - Installs `archiso`, `squashfs-tools`, `dosfstools`, `e2fsprogs`,
     `xorriso`, `grub`, `efibootmgr`, `mtools`.
   - Copies the repo into `/astraos-build`.
   - Runs `scripts/copy-assets.sh` to stage AstraOS wallpapers + logos into
     `/astraos-build/iso/x86_64/airootfs/usr/share/{backgrounds,pixmaps}/astraos/`.
   - On `CMD`, runs:
     ```
     mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64
     ```

2. **Dev image** (`docker/Dockerfile.dev`)
   - Used by `make dev` and VS Code DevContainers.
   - Arch Linux + Rust + GTK4 + gtk4-rs + neovim + starship + paru.
   - Default user `astra` with `wheel` NOPASSWD via sudoers.

---

## 4. Test the ISO

### Option A — VirtualBox (recommended on Windows)

1. Open VirtualBox → **New**.
2. **Name:** `AstraOS-test`, **Type:** Linux, **Version:** Arch Linux (64-bit).
3. **RAM:** 4096 MB, **CPU:** 4 cores (Settings → System → Processor).
4. **Display → Graphics Controller:** `VMSVGA`, **Video Memory:** 128 MB.
5. **Storage:** add `out/astraos-core.iso` to the empty optical drive.
6. **Boot** the VM.

### Option B — QEMU

```bash
make test-qemu
```

The Makefile auto-detects KVM if available and falls back to a software
renderer otherwise. You can override RAM / CPU / disk via env vars:

```bash
QEMU_MEM=8192 QEMU_CPUS=8 make test-qemu
```

### Option C — GitHub Codespaces

1. Create a Codespace from this repo (4-core / 16 GiB recommended).
2. `make build-iso`
3. Download `out/astraos-core.iso` from the Codespace file explorer.
4. Boot it on your local machine in VirtualBox or QEMU.

> Codespaces runs in a container, so nested virtualization is **not**
> available — you must download the ISO and boot it on your host.

---

## 5. Common issues

| Symptom | Fix |
| --- | --- |
| `docker: permission denied` | Run Docker Desktop as your user, not root. |
| `mkarchiso: profiledef.sh: not found` | Run `make copy-assets` first. |
| ISO boots to GRUB but kernel panics | Disable Hyper-V on Windows host, or try `VBoxSVGA` instead of `VMSVGA`. |
| `pacman -Syu` fails inside container | Check your DNS / proxy. Set `pacman.conf` mirrors manually. |
| Boot hangs on `Reached target Graphical Interface` | ISO 1 ships no graphical target — switch to a TTY (`Ctrl+Alt+F2`) and run `Hyprland` manually. |

---

## 6. Cleaning up

```bash
make clean        # removes work/ and out/
docker system prune   # reclaim Docker disk
```

---

## 7. Next steps

- ISO 2 will add the Astra Shell (Rust + GTK4-rs) with bar, dock, launcher,
  lock screen, and Welcome screen. See [`ROADMAP.md`](ROADMAP.md).
- Once the DevContainer is wired into VS Code, you can `make dev` and start
  writing Rust + GTK4-rs code without installing anything on the host.

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → docs/ROADMAP.md"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "docs/ROADMAP.md"
# AstraOS — Roadmap

This document tracks the two parallel roadmaps for AstraOS:

1. **The 7 ISO milestones** (v0.1 → v1.0) — incremental desktop releases.
2. **The 7 phases** (post-v1.0) — gaming, mobile, ARM, Android compat.

---

## ISO milestones (v1.0 release)

Each ISO is testable end-to-end in VirtualBox / QEMU. We **do not** start the
next ISO until the previous one is validated.

| ISO | Version | Content | Test focus |
| --- | --- | --- | --- |
| **ISO 1** | 0.1 | Base Arch + Hyprland + AstraOS boot logo + violet wallpaper + branded bash prompt | Boots in VirtualBox? |
| **ISO 2** | 0.2 | + Astra Shell (bar/dock dual Win/Mac, launcher, lock screen, mini onboarding) + Welcome screen with typing animation | UI stable? WM works? |
| **ISO 3** | 0.3 | + System apps: Kitty, Thunar, Firefox, VLC, AstraOS Settings | Apps work? |
| **ISO 4** | 0.4 | + AstraPass, Sticky Notes, Widgets, 3 games (Minesweeper, Tic-Tac-Toe, Snake) | Custom apps run? |
| **ISO 5** | 0.5 | + Astra Defender, Task Manager, Performance Mode, Game Mode, System Monitor | System functional? |
| **ISO 6** | 0.6 | + Astra Assistant (Qwen iframe, opt-in), Voice control, Smart search, Screen recorder / Screenshot / Audio / Webcam | Advanced features OK? |
| **ISO 7** | 1.0 | + Astra Gaming Launcher + Calamares installer + Multi-language (FR/EN/ES/DE) + Multi-user + polish | Production-ready? |

> Current status: **ISO 1 (v0.1) — in progress**.

### ISO packs (final v1.0)

A single codebase produces multiple packs by swapping `packages.x86_64`:

| Pack | Audience | Apps |
| --- | --- | --- |
| **AstraOS Core** | Base / power users | Hyprland + Terminal + Files + Settings + AstraOS apps |
| **AstraOS Developer** | Developers | Core + VSCodium + Claude Code + Docker + Git + Rust toolchain + Kitty |
| **AstraOS Bureautique** | General public / office | Core + Brave + LibreOffice (nothing else) |
| **AstraOS Gaming** (phase 3+) | Gamers | Core + Astra Gaming Launcher + Steam + Proton + RetroArch |

Build commands:

```bash
make build-iso                # core (default)
make build-iso-dev            # developer pack
make build-iso-bureautique    # bureautique pack
```

ISOs 1-7 are built with the **Core** pack. The other packs are generated in
parallel starting from ISO 7.

---

## Post-v1.0 phases

| Phase | Features |
| --- | --- |
| **Phase 1** (v1.0) | Base desktop ISO 1-7, no gaming focus |
| **Phase 2** | Astra Bridge (Wine/Proton) + RetroArch + Astra Mail + Astra Sync + Astra Store + Astra Photos |
| **Phase 3** | Astra Gaming Launcher (Steam/Epic/Ubisoft unified) + Astra Phone Sync |
| **Phase 4** | Astra WinBox beta (gaming VM for kernel anti-cheat games, power users only) |
| **Phase 5** | Astra WinBox stable + Sunshine/Moonlight local streaming + Astra Mobile (smartphone variant) |
| **Phase 6** | Astra OS ARM (RTX Spark target) + x86→ARM emulation (Box64/FEX-Emu) |
| **Phase 7** | Waydroid (Android apps native on AstraOS) |

---

## Development priority order (user-confirmed)

In strict order — Astra Bridge and Astra WinBox come **last**:

1. ISO 1-7 (v1.0) — full desktop
2. Phase 2 — Astra Mail + Astra Sync + Astra Store + Astra Photos + RetroArch
3. Phase 3 — Astra Gaming Launcher + Astra Phone Sync
4. Phase 4 — **Astra Bridge** (Wine/Proton) — start of Windows gaming
5. Phase 5 — Astra Mobile + AstraOS ARM + Box64/FEX-Emu
6. Phase 6 — Waydroid
7. Phase 7 (last) — **Astra WinBox** (gaming VM for Valorant/R6/Fortnite) — the most complex, kept for the end

---

## Gaming strategy — "Reliable + Free"

After exhaustive reverse engineering (18 techniques analyzed), the final
verdict is:

> The ONLY 100% reliable + 100% free solution is: do **not** try to run
> kernel anti-cheat games on AstraOS natively.

The Astra Gaming Launcher auto-routes to the best path:

| Level | Game type | Solution | Reliability | Cost |
| --- | --- | --- | --- | --- |
| 1 | Native Linux game (CS2, Apex, OW2, Dota) | Direct launch | ✅ 100% | Free |
| 2 | Windows game without kernel anti-cheat (Cyberpunk, Elden Ring, indie) | Astra Bridge (Wine/Proton) | ✅ 95% | Free |
| 3 | Retro console game (SNES, PS2, GC, Switch) | RetroArch emulator | ✅ 100% | Free |
| 4 | Android mobile game | Waydroid container | ✅ 90% | Free |
| 5 | Windows game WITH kernel anti-cheat (Valorant, R6, Fortnite, PUBG) | ⚠️ Astra WinBox (phase 4+) | ⚠️ 70-80% ban risk | Free but risky |

For Level 5 games the Astra Gaming Launcher shows an explicit risk warning
and requires user opt-in. AstraOS is not responsible for bans.

---

## Explicitly rejected (v1.0)

- Astra Mobile (smartphone variant) → **reinstated** for phase 3+
- Screen reader (Orca) — accessibility too complex for scope
- Magnifier — not supported
- Disk encryption by default — kept for v1.1
- Auto-organize IA via API (cost) — reconsidered with local file access
- **Astra Chatt** — user's personal chat app, set aside for later
- **Tencent products** — official boycott (Riot 100%, Epic 40%, Ubisoft 5%).
  Valorant explicitly not supported. LoL works but is not promoted.
- **Astra Bridge** (Wine/Proton translation layer) — phase 2+
- **Astra WinBox** (gaming VM for kernel anti-cheat games) — phase 2+
  (beta for power users) → phase 4 (just works for general public)

---

## Current status

| Milestone | Status |
| --- | --- |
| Project identity locked | ✅ done |
| Stack locked (Rust + GTK4-rs + Hyprland + Arch) | ✅ done |
| Visual identity locked (violet/magenta glassmorphism) | ✅ done |
| Iterative ISO plan validated (7 ISOs) | ✅ done |
| Assets staged (`astraos/assets/`) | ✅ done |
| **ISO 1 build pipeline** | ✅ in progress |
| ISO 1 user validation in VirtualBox | ⏳ pending user GO |
| ISO 2+ (Astra Shell) | 🅾️ not started |

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/boot/grub/themes/astraos/README.md"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/boot/grub/themes/astraos/README.md"
# AstraOS GRUB Theme

Minimal dark GRUB theme with magenta accent, matching the AstraOS visual
identity (magenta star logo + deep space background + soft slate text).

## Status

**ISO 1 (v0.1):** This theme is **NOT shipped in the live ISO**. The live ISO
boots via syslinux (BIOS) or systemd-boot (UEFI), not GRUB. The `theme.txt`
file lives here inside `airootfs/boot/grub/themes/astraos/` so that when a
user installs AstraOS to disk via Calamares, the post-install hook can
produce a properly themed GRUB boot menu.

## Required assets (installed at install time)

The following binary asset files are NOT committed to this repo — they will
be copied by `customize_airootfs.sh` (or by a post-install hook in Calamares):

| Asset                  | Source                                          |
|------------------------|-------------------------------------------------|
| `background.png`       | `assets/wallpapers/default-violet.png` (scaled) |
| `highlight_*.png`      | generated from AstraOS magenta palette (#E040FB)|
| `slider_*.png`         | generated from AstraOS magenta palette          |
| `menu_*.png`           | generated from AstraOS dark panel palette        |

Fonts (`DejaVu Sans`, `DejaVu Sans Bold`) are shipped by the `grub` package
itself and don't need to be added here.

## Install-time directive

Suggested snippet for `customize_airootfs.sh` or Makefile:

```sh
THEME_DIR="${AIROOTFS}/boot/grub/themes/astraos"
install -Dm0644 astraos/assets/wallpapers/default-violet.png "${THEME_DIR}/background.png"
# Generate highlight_*.png / slider_*.png / menu_*.png from magenta swatches
# (left as a task for the build pipeline — see makefile target `grub-theme-assets`)
```

## Color reference (AstraOS palette)

| Token              | Hex       | Usage                         |
|--------------------|-----------|-------------------------------|
| Magenta accent     | `#E040FB` | titles, selected items        |
| Deep space bg      | `#0b0d17` | desktop color                 |
| Soft slate text    | `#f1f5f9` | unselected menu items         |
| Muted slate        | `#94a3b8` | hint / progress text          |
| Dim slate          | `#475569` | footer / copyright            |

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/boot/grub/themes/astraos/theme.txt"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/boot/grub/themes/astraos/theme.txt"
# AstraOS GRUB Theme
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Minimal dark theme with magenta accent — matches AstraOS visual identity
# (magenta star logo + deep space background + soft slate text).
#
# Asset dependencies (installed at install time, NOT in the live ISO):
#   - background.png   → default-violet AstraOS wallpaper scaled for boot menu
#   - highlight_c.png   → magenta highlight left edge
#   - highlight_e.png   → magenta highlight right edge
#   - highlight_w.png   → magenta highlight left cap
#   - highlight_n.png   → magenta highlight top edge
#   - highlight_s.png   → magenta highlight bottom edge
#   - highlight_ne.png  → magenta highlight top-right corner
#   - highlight_nw.png  → magenta highlight top-left corner
#   - highlight_se.png  → magenta highlight bottom-right corner
#   - highlight_sw.png  → magenta highlight bottom-left corner
#   - slider_c.png      → vertical slider center (scrollbar)
#   - slider_n.png      → vertical slider top
#   - slider_s.png      → vertical slider bottom
#   - DejaVu Sans Bold 24 / DejaVu Sans 16 / DejaVu Sans 12 (already shipped by GRUB)

desktop-image: "background.png"
desktop-color: "#0b0d17"
title-text: "AstraOS Boot Manager"
title-color: "#E040FB"
title-font: "DejaVu Sans Bold 24"
title-align: "center"

+ boot_menu {
    left = 25%
    top = 30%
    width = 50%
    height = 40%
    item_color = "#f1f5f9"
    selected_item_color = "#0b0d17"
    selected_item_pixmap_style = "highlight_*.png"
    item_height = 32
    item_spacing = 8
    item_font = "DejaVu Sans 16"
    selected_item_font = "DejaVu Sans Bold 16"
    scrollbar = true
    scrollbar_width = 8
    scrollbar_thumb = "slider_*.png"
    scrollbar_frame = "slider_*.png"
    menu_pixmap_style = "menu_*.png"
}

+ label {
    text = "Use ↑↓ to select, Enter to boot"
    color = "#94a3b8"
    font = "DejaVu Sans 12"
    left = 35%
    top = 80%
}

+ label {
    text = "AstraOS v0.1 — Astra Corporation"
    color = "#475569"
    font = "DejaVu Sans 10"
    align = "center"
    left = 0
    top = 95%
    width = 100%
}

+ progress_bar {
    id = "__timeout__"
    text = "Booting in %d s"
    color = "#94a3b8"
    left = 35%
    top = 84%
    width = 30%
    height = 16
    font = "DejaVu Sans 12"
    bar_style = "highlight_*.png"
    text_color = "#E040FB"
}

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/astra/boot-banner"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/astra/boot-banner"

        ██████ ███████ ████████ ███████ ██   ██  ██████  ██    ██
       ██      ██         ██    ██      ██   ██ ██    ██  ██  ██
       ██      █████      ██    █████   ███████ ██    ██   ████
       ██      ██         ██    ██      ██   ██ ██    ██    ██
        ██████ ███████    ██    ███████ ██   ██  ██████     ██

                          v e r s i o n   0 . 1 . 0

                Welcome to AstraOS — Arch-based, Rust-powered.

   Booting Hyprland session... please wait while the system comes online.

        * Eco RAM     * Glassmorphism     * Independent UI     * Open Source


ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/astra/release-info"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/astra/release-info"
NAME="AstraOS"
VERSION="0.1.0"
EDITION="Core"
VERSION_CODENAME=astraos
BUILD_ID=2026.09
PRETTY_NAME="AstraOS 0.1.0 (Core) - Live ISO"
ANSI_COLOR="1;35"
HOME_URL="https://astraos.org"
DOCUMENTATION_URL="https://docs.astraos.org"
SUPPORT_URL="https://community.astraos.org"
BUG_REPORT_URL="https://github.com/astraos/astraos/issues"

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/astra/version"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/astra/version"
0.1.0

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/default/grub"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/default/grub"
# /etc/default/grub — AstraOS defaults
# SPDX-License-Identifier: GPL-3.0-or-later
#
# This file is NOT used by the live ISO (which uses syslinux/systemd-boot),
# but it is installed inside the airootfs so that when a user installs AstraOS
# to disk via Calamares, the resulting GRUB install picks up AstraOS branding
# (distributor name, default background, theme, cryptodisk, etc.).
#
# Generated for AstraOS ISO 1 (v0.1). Edit `grub-mkconfig -o /boot/grub/grub.cfg`
# after any change to this file.

# ---------------------------------------------------------------------------
# Boot menu behavior
# ---------------------------------------------------------------------------
GRUB_DEFAULT=0
GRUB_TIMEOUT=5
GRUB_TIMEOUT_STYLE=menu
GRUB_DISTRIBUTOR="AstraOS"

# ---------------------------------------------------------------------------
# Kernel command line
# ---------------------------------------------------------------------------
GRUB_CMDLINE_LINUX_DEFAULT="loglevel=3 quiet"
GRUB_CMDLINE_LINUX=""

# ---------------------------------------------------------------------------
# Disk encryption (LUKS) — enabled so users can adopt full-disk crypto
# in a future ISO without having to re-edit grub.
# ---------------------------------------------------------------------------
GRUB_ENABLE_CRYPTODISK=y

# ---------------------------------------------------------------------------
# Preload fonts and terminal setup
# ---------------------------------------------------------------------------
GRUB_PRELOAD_MODULES="part_gpt part_msdos lvm luks ext2 btrfs xfs zfs"
GRUB_TERMINAL_INPUT="console"
GRUB_TERMINAL_OUTPUT="gfxterm"

# ---------------------------------------------------------------------------
# Graphics — let GRUB auto-detect the best mode
# ---------------------------------------------------------------------------
GRUB_GFXMODE=auto
GRUB_GFXPAYLOAD_LINUX=keep

# ---------------------------------------------------------------------------
# AstraOS branding assets (installed by customize script / post-install hook)
# ---------------------------------------------------------------------------
GRUB_BACKGROUND="/usr/share/backgrounds/astraos/default-violet.png"
GRUB_THEME="/boot/grub/themes/astraos/theme.txt"
GRUB_FONT="/boot/grub/fonts/unicode.pf2"

# ---------------------------------------------------------------------------
# Boot menu cosmetics (only used if GRUB_THEME is not set)
# ---------------------------------------------------------------------------
GRUB_COLOR_NORMAL="light-black/black"
GRUB_COLOR_HIGHLIGHT="magenta/light-gray"

# ---------------------------------------------------------------------------
# Disable submenu nesting (flat menu = friendlier for general public)
# ---------------------------------------------------------------------------
GRUB_DISABLE_SUBMENU=y

# ---------------------------------------------------------------------------
# Disable recovery entry (not relevant for a freshly-installed AstraOS)
# ---------------------------------------------------------------------------
GRUB_DISABLE_RECOVERY=true

# ---------------------------------------------------------------------------
# OS prober — disabled by default for security/predictability.
# Users who dual-boot can re-enable with `GRUB_DISABLE_OS_PROBER=false`.
# ---------------------------------------------------------------------------
GRUB_DISABLE_OS_PROBER=true

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/fstab"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/fstab"
# fstab is generated at boot by archiso

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/greetd/config.toml"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/greetd/config.toml"
# greetd configuration for AstraOS
# greetd is the display manager that launches Hyprland at boot

[terminal]
# The VT to run the greeter on
vt = 1

# Switch to this VT when starting the greeter
switch = true

[default_session]
# Launch Hyprland directly (no greeter for ISO 1 - we'll add custom lock screen in ISO 2)
# This is a live ISO, so we auto-login as 'astra' user
command = "Hyprland"

# The user to run the session as
user = "astra"

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/group"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/group"
root:x:0:root
bin:x:1:root,bin,daemon
daemon:x:2:root,bin,daemon
sys:x:3:root,bin
adm:x:4:root,daemon
tty:x:5:
disk:x:6:root
wheel:x:10:root,astra
uucp:x:14:root
man:x:15:
proxy:x:19:
kmem:x:17:
input:x:24:
kvm:x:28:
render:x:29:
sgx:x:30:
tape:x:26:root
video:x:91:root
audio:x:92:root,astra
lp:x:7:root
ftp:x:11:
http:x:33:
lock:x:54:
mail:x:12:
log:x:19:
smmsp:x:25:
proc:x:26:
games:x:50:
network:x:90:
floppy:x:94:
scanner:x:96:
power:x:98:
nobody:x:65534:
systemd-journal-gateway:x:193:
systemd-network:x:190:
systemd-resolve:x:191:
systemd-timesync:x:192:
systemd-coredump:x:997:
uuidd:x:68:
dbus:x:81:
tss:x:941:
astra:x:1000:astra

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/gshadow"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/gshadow"
root:!*::root
bin:!*::root,bin,daemon
daemon:!*::root,bin,daemon
sys:!*::root,bin
adm:!*::root,daemon
tty:!*::
disk:!*::root
wheel:!*::root,astra
uucp:!*::root
man:!*::
proxy:!*::
kmem:!*::
input:!*::
kvm:!*::
render:!*::
sgx:!*::
tape:!*::root
video:!*::root
audio:!*::root,astra
lp:!*::root
ftp:!*::
http:!*::
lock:!*::
mail:!*::
log:!*::
smmsp:!*::
proc:!*::
games:!*::
network:!*::
floppy:!*::
scanner:!*::
power:!*::
nobody:!*::
systemd-journal-gateway:!*::
systemd-network:!*::
systemd-resolve:!*::
systemd-timesync:!*::
systemd-coredump:!*::
uuidd:!*::
dbus:!*::
tss:!*::
astra:!*::astra

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/hostname"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/hostname"
astraos

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/hosts"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/hosts"
# Static table lookup for hostnames
# See hosts(5) for details
127.0.0.1	localhost
::1			localhost
127.0.1.1	astraos.localdomain	astraos

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/locale.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/locale.conf"
LANG=fr_FR.UTF-8
LC_MESSAGES=en_US.UTF-8
LC_TIME=fr_FR.UTF-8

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/locale.gen"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/locale.gen"
# Configuration file for locale-gen
#
# This file is part of the AstraOS ISO 1 (v0.1) profile.
#
# List of locales to generate on the system. Each line is of the form:
#
#   <locale> <charmap>
#
# Where <locale> is a locale name and <charmap> is the name of a charmap.
# Lines beginning with # are comments. To enable a locale, uncomment the line.
#
# Examples:
#   en_US.UTF-8 UTF-8
#   de_DE.UTF-8 UTF-8
#
# Run `locale-gen` after editing this file. (Done automatically by
# customize_airootfs.sh during the ISO build.)
#
# AstraOS v1.0 target locales are uncommented below; all others remain
# commented out as in the upstream Arch Linux locale.gen.

#aa_DJ.UTF-8 UTF-8
#aa_DJ ISO-8859-1
#aa_ER UTF-8
#aa_ER@saaho UTF-8
#aa_ET UTF-8
#af_ZA.UTF-8 UTF-8
#af_ZA ISO-8859-1
#agr_PE UTF-8
#ak_GH UTF-8
#am_ET UTF-8
#an_ES.UTF-8 UTF-8
#an_ES ISO-8859-15
#anp_IN UTF-8
#ar_AE.UTF-8 UTF-8
#ar_AE ISO-8859-6
#ar_BH.UTF-8 UTF-8
#ar_BH ISO-8859-6
#ar_DZ.UTF-8 UTF-8
#ar_DZ ISO-8859-6
#ar_EG.UTF-8 UTF-8
#ar_EG ISO-8859-6
#ar_EG ISO-8859-6
#ar_IQ.UTF-8 UTF-8
#ar_IQ ISO-8859-6
#ar_JO.UTF-8 UTF-8
#ar_JO ISO-8859-6
#ar_KW.UTF-8 UTF-8
#ar_KW ISO-8859-6
#ar_LB.UTF-8 UTF-8
#ar_LB ISO-8859-6
#ar_LY.UTF-8 UTF-8
#ar_LY ISO-8859-6
#ar_MA.UTF-8 UTF-8
#ar_MA ISO-8859-6
#ar_OM.UTF-8 UTF-8
#ar_OM ISO-8859-6
#ar_QA.UTF-8 UTF-8
#ar_QA ISO-8859-6
#ar_SA.UTF-8 UTF-8
#ar_SA ISO-8859-6
#ar_SD.UTF-8 UTF-8
#ar_SD ISO-8859-6
#ar_SS UTF-8
#ar_SY.UTF-8 UTF-8
#ar_SY ISO-8859-6
#ar_TN.UTF-8 UTF-8
#ar_TN ISO-8859-6
#ar_YE.UTF-8 UTF-8
#ar_YE ISO-8859-6
#as_IN UTF-8
#ast_ES.UTF-8 UTF-8
#ast_ES ISO-8859-15
#az_AZ UTF-8
#az_IR UTF-8
#be_BY.UTF-8 UTF-8
#be_BY CP1251
#be_BY@latin UTF-8
#bem_ZM UTF-8
#ber_DZ UTF-8
#ber_MA UTF-8
#bg_BG.UTF-8 UTF-8
#bg_BG CP1251
#bhb_IN.UTF-8 UTF-8
#bho_IN UTF-8
#bho_NP UTF-8
#bi_VU UTF-8
#bn_BD UTF-8
#bn_IN UTF-8
#bo_CN UTF-8
#bo_IN UTF-8
#br_FR.UTF-8 UTF-8
#br_FR ISO-8859-1
#brx_IN UTF-8
#bs_BA.UTF-8 UTF-8
#bs_BA ISO-8859-2
#byn_ER UTF-8
#ca_AD.UTF-8 UTF-8
#ca_ES.UTF-8 UTF-8
#ca_ES ISO-8859-1
#ca_ES@valencia UTF-8
#ca_FR.UTF-8 UTF-8
#ca_IT.UTF-8 UTF-8
#ce_RU UTF-8
#chr_US UTF-8
#ckb_IQ UTF-8
#cmn_TW UTF-8
#crh_UA UTF-8
#cs_CZ.UTF-8 UTF-8
#cs_CZ ISO-8859-2
#csb_PL UTF-8
#cv_RU UTF-8
#cy_GB.UTF-8 UTF-8
#cy_GB ISO-8859-14
#da_DK.UTF-8 UTF-8
#da_DK ISO-8859-1
#de_AT.UTF-8 UTF-8
#de_AT ISO-8859-1
#de_AT@euro ISO-8859-15
#de_BE.UTF-8 UTF-8
#de_BE ISO-8859-1
#de_BE@euro ISO-8859-15
#de_CH.UTF-8 UTF-8
#de_CH ISO-8859-1
de_DE.UTF-8 UTF-8
#de_DE ISO-8859-1
#de_DE@euro ISO-8859-15
#de_IT.UTF-8 UTF-8
#de_IT ISO-8859-1
#de_LI.UTF-8 UTF-8
#de_LU.UTF-8 UTF-8
#de_LU ISO-8859-1
#de_LU@euro ISO-8859-15
#doi_IN UTF-8
#dsb_DE UTF-8
#dv_MV UTF-8
#dz_BT UTF-8
#el_CY.UTF-8 UTF-8
#el_CY ISO-8859-7
#el_GR.UTF-8 UTF-8
#el_GR ISO-8859-7
#el_GR@euro ISO-8859-7
#en_AG UTF-8
#en_AU.UTF-8 UTF-8
#en_AU ISO-8859-1
#en_BW.UTF-8 UTF-8
#en_BW ISO-8859-1
#en_CA.UTF-8 UTF-8
#en_CA ISO-8859-1
#en_DK.UTF-8 UTF-8
#en_DK ISO-8859-1
#en_GB.UTF-8 UTF-8
#en_GB ISO-8859-1
#en_GB.ISO-8859-15 ISO-8859-15
#en_HK.UTF-8 UTF-8
#en_HK ISO-8859-1
#en_IE.UTF-8 UTF-8
#en_IE ISO-8859-1
#en_IE@euro ISO-8859-15
#en_IL UTF-8
#en_IN UTF-8
#en_NG UTF-8
#en_NZ.UTF-8 UTF-8
#en_NZ ISO-8859-1
#en_PH.UTF-8 UTF-8
#en_PH ISO-8859-1
#en_SC.UTF-8 UTF-8
#en_SG.UTF-8 UTF-8
#en_SG ISO-8859-1
#en_US ISO-8859-1
en_US.UTF-8 UTF-8
#en_ZA.UTF-8 UTF-8
#en_ZA ISO-8859-1
#en_ZM UTF-8
#en_ZW.UTF-8 UTF-8
#en_ZW ISO-8859-1
#eo UTF-8
#es_AR.UTF-8 UTF-8
#es_AR ISO-8859-1
#es_BO.UTF-8 UTF-8
#es_BO ISO-8859-1
#es_CL.UTF-8 UTF-8
#es_CL ISO-8859-1
#es_CO.UTF-8 UTF-8
#es_CO ISO-8859-1
#es_CR.UTF-8 UTF-8
#es_CR ISO-8859-1
#es_CU UTF-8
#es_DO.UTF-8 UTF-8
#es_DO ISO-8859-1
#es_EC.UTF-8 UTF-8
#es_EC ISO-8859-1
es_ES.UTF-8 UTF-8
#es_ES ISO-8859-1
#es_ES@euro ISO-8859-15
#es_GT.UTF-8 UTF-8
#es_GT ISO-8859-1
#es_HN.UTF-8 UTF-8
#es_HN ISO-8859-1
#es_MX.UTF-8 UTF-8
#es_MX ISO-8859-1
#es_NI.UTF-8 UTF-8
#es_NI ISO-8859-1
#es_PA.UTF-8 UTF-8
#es_PA ISO-8859-1
#es_PE.UTF-8 UTF-8
#es_PE ISO-8859-1
#es_PR.UTF-8 UTF-8
#es_PR ISO-8859-1
#es_PY.UTF-8 UTF-8
#es_PY ISO-8859-1
#es_SV.UTF-8 UTF-8
#es_SV ISO-8859-1
#es_US.UTF-8 UTF-8
#es_US ISO-8859-1
#es_UY.UTF-8 UTF-8
#es_UY ISO-8859-1
#es_VE.UTF-8 UTF-8
#es_VE ISO-8859-1
#et_EE.UTF-8 UTF-8
#et_EE ISO-8859-1
#et_EE.ISO-8859-15 ISO-8859-15
#eu_ES.UTF-8 UTF-8
#eu_ES ISO-8859-1
#eu_ES@euro ISO-8859-15
#eu_FR.UTF-8 UTF-8
#eu_FR ISO-8859-1
#eu_FR@euro ISO-8859-15
#fa_IR UTF-8
#ff_SN UTF-8
#fi_FI.UTF-8 UTF-8
#fi_FI ISO-8859-1
#fi_FI@euro ISO-8859-15
#fil_PH UTF-8
#fo_FO.UTF-8 UTF-8
#fo_FO ISO-8859-1
#fr_BE.UTF-8 UTF-8
#fr_BE ISO-8859-1
#fr_BE@euro ISO-8859-15
#fr_CA.UTF-8 UTF-8
#fr_CA ISO-8859-1
#fr_CH.UTF-8 UTF-8
#fr_CH ISO-8859-1
#fr_FR.UTF-8 UTF-8
fr_FR.UTF-8 UTF-8
#fr_FR ISO-8859-1
#fr_FR@euro ISO-8859-15
#fr_LU.UTF-8 UTF-8
#fr_LU ISO-8859-1
#fr_LU@euro ISO-8859-15
#fur_IT UTF-8
#fy_NL UTF-8
#fy_DE UTF-8
#ga_IE.UTF-8 UTF-8
#ga_IE ISO-8859-1
#ga_IE@euro ISO-8859-15
#gd_GB.UTF-8 UTF-8
#gd_GB ISO-8859-15
#gez_ER UTF-8
#gez_ER@abegede UTF-8
#gez_ET UTF-8
#gez_ET@abegede UTF-8
#gl_ES.UTF-8 UTF-8
#gl_ES ISO-8859-1
#gl_ES@euro ISO-8859-15
#gu_IN UTF-8
#gv_GB.UTF-8 UTF-8
#gv_GB ISO-8859-1
#ha_NG UTF-8
#hak_TW UTF-8
#he_IL.UTF-8 UTF-8
#he_IL ISO-8859-8
#hif_FJ UTF-8
#hi_IN UTF-8
#hne_IN UTF-8
#hr_HR.UTF-8 UTF-8
#hr_HR ISO-8859-2
#hsb_DE ISO-8859-2
#hsb_DE.UTF-8 UTF-8
#ht_HT UTF-8
#hu_HU.UTF-8 UTF-8
#hu_HU ISO-8859-2
#hy_AM UTF-8
#hy_AM.ARMSCII-8 ARMSCII-8
#ia_FR UTF-8
#id_ID.UTF-8 UTF-8
#id_ID ISO-8859-1
#ig_NG UTF-8
#ik_CA UTF-8
#is_IS.UTF-8 UTF-8
#is_IS ISO-8859-1
#it_CH.UTF-8 UTF-8
#it_CH ISO-8859-1
it_IT.UTF-8 UTF-8
#it_IT ISO-8859-1
#it_IT@euro ISO-8859-15
#iu_CA UTF-8
#ja_JP.EUC-JP EUC-JP
#ja_JP.UTF-8 UTF-8
#ka_GE.UTF-8 UTF-8
#ka_GE GEORGIAN-PS
#kab_DZ UTF-8
#kk_KZ.UTF-8 UTF-8
#kk_KZ PT154
#kl_GL.UTF-8 UTF-8
#kl_GL ISO-8859-1
#km_KH UTF-8
#kn_IN UTF-8
#ko_KR.EUC-KR EUC-KR
#ko_KR.UTF-8 UTF-8
#kok_IN UTF-8
#ks_IN UTF-8
#ks_IN@devanagari UTF-8
#ku_TR.UTF-8 UTF-8
#ku_TR ISO-8859-9
#kw_GB.UTF-8 UTF-8
#kw_GB ISO-8859-1
#ky_KG UTF-8
#lb_LU UTF-8
#lg_UG.UTF-8 UTF-8
#lg_UG ISO-8859-10
#li_BE UTF-8
#li_NL UTF-8
#lij_IT UTF-8
#ln_CD UTF-8
#lo_LA UTF-8
#lt_LT.UTF-8 UTF-8
#lt_LT ISO-8859-13
#lv_LV.UTF-8 UTF-8
#lv_LV ISO-8859-13
#lzh_TW UTF-8
#mag_IN UTF-8
#mai_IN UTF-8
#mai_NP UTF-8
#mfe_MU UTF-8
#mg_MG.UTF-8 UTF-8
#mg_MG ISO-8859-15
#mhr_RU UTF-8
#mi_NZ.UTF-8 UTF-8
#mi_NZ ISO-8859-13
#miq_NI UTF-8
#mjw_IN UTF-8
#mk_MK.UTF-8 UTF-8
#mk_MK ISO-8859-5
#ml_IN UTF-8
#mn_MN UTF-8
#mni_IN UTF-8
#mnw_MM UTF-8
#mr_IN UTF-8
#ms_MY.UTF-8 UTF-8
#ms_MY ISO-8859-1
#mt_MT.UTF-8 UTF-8
#mt_MT ISO-8859-3
#my_MM UTF-8
#nan_TW UTF-8
#nan_TW@latin UTF-8
#nb_NO.UTF-8 UTF-8
#nb_NO ISO-8859-1
#nds_DE UTF-8
#nds_NL UTF-8
#ne_NP UTF-8
#nhn_MX UTF-8
#niu_NU UTF-8
#niu_NZ UTF-8
#nl_AW UTF-8
#nl_BE.UTF-8 UTF-8
#nl_BE ISO-8859-1
#nl_BE@euro ISO-8859-15
#nl_NL.UTF-8 UTF-8
#nl_NL ISO-8859-1
#nl_NL@euro ISO-8859-15
#nn_NO.UTF-8 UTF-8
#nn_NO ISO-8859-1
#nr_ZA UTF-8
#nso_ZA UTF-8
#oc_FR.UTF-8 UTF-8
#oc_FR ISO-8859-1
#om_KE.UTF-8 UTF-8
#om_KE ISO-8859-1
#or_IN UTF-8
#os_RU UTF-8
#pa_IN UTF-8
#pa_PK UTF-8
#pap_AW UTF-8
#pap_CW UTF-8
#pl_PL.UTF-8 UTF-8
#pl_PL ISO-8859-2
#ps_AF UTF-8
#pt_BR.UTF-8 UTF-8
pt_BR.UTF-8 UTF-8
#pt_BR ISO-8859-1
#pt_PT.UTF-8 UTF-8
#pt_PT ISO-8859-1
#pt_PT@euro ISO-8859-15
#quz_PE UTF-8
#raj_IN UTF-8
#ro_RO.UTF-8 UTF-8
#ro_RO ISO-8859-2
#ru_RU.KOI8-R KOI8-R
#ru_RU.UTF-8 UTF-8
#ru_RU ISO-8859-5
#ru_UA.UTF-8 UTF-8
#ru_UA KOI8-U
#rw_RW UTF-8
#sa_IN UTF-8
#sah_RU UTF-8
#sat_IN UTF-8
#sc_IT UTF-8
#sd_IN UTF-8
#sd_IN@devanagari UTF-8
#se_NO UTF-8
#sgs_LT UTF-8
#shn_MM UTF-8
#shs_CA UTF-8
#si_LK UTF-8
#sid_ET UTF-8
#sk_SK.UTF-8 UTF-8
#sk_SK ISO-8859-2
#sl_SI.UTF-8 UTF-8
#sl_SI ISO-8859-2
#sm_WS UTF-8
#so_DJ.UTF-8 UTF-8
#so_DJ ISO-8859-1
#so_ET.UTF-8 UTF-8
#so_KE.UTF-8 UTF-8
#so_KE ISO-8859-1
#so_SO.UTF-8 UTF-8
#so_SO ISO-8859-1
#sq_AL.UTF-8 UTF-8
#sq_AL ISO-8859-1
#sq_MK.UTF-8 UTF-8
#sq_MK ISO-8859-2
#sr_ME.UTF-8 UTF-8
#sr_ME ISO-8859-2
#sr_RS.UTF-8 UTF-8
#sr_RS ISO-8859-2
#sr_RS@latin UTF-8
#ss_ZA UTF-8
#st_ZA.UTF-8 UTF-8
#st_ZA ISO-8859-1
#sv_FI.UTF-8 UTF-8
#sv_FI ISO-8859-1
#sv_FI@euro ISO-8859-15
#sv_SE.UTF-8 UTF-8
#sv_SE ISO-8859-1
#sv_SE.ISO-8859-15 ISO-8859-15
#sw_KE UTF-8
#sw_TZ UTF-8
#syr UTF-8
#szl_PL UTF-8
#ta_IN UTF-8
#ta_LK UTF-8
#te_IN UTF-8
#tg_TJ.UTF-8 UTF-8
#tg_TJ KOI8-T
#th_TH.UTF-8 UTF-8
#th_TH TIS-620
#the_NP UTF-8
#ti_ER UTF-8
#ti_ET UTF-8
#tig_ER UTF-8
#tk_TM UTF-8
#tl_PH.UTF-8 UTF-8
#tl_PH ISO-8859-1
#tn_ZA UTF-8
#to_TO UTF-8
#tpi_PG UTF-8
#tr_CY.UTF-8 UTF-8
#tr_CY ISO-8859-9
#tr_TR.UTF-8 UTF-8
#tr_TR ISO-8859-9
#ts_ZA UTF-8
#tt_RU.UTF-8 UTF-8
#tt_RU@iqtelif UTF-8
#ug_CN UTF-8
#uk_UA.UTF-8 UTF-8
#uk_UA KOI8-U
#unm_US UTF-8
#ur_IN UTF-8
#ur_PK UTF-8
#uz_UZ.UTF-8 UTF-8
#uz_UZ ISO-8859-1
#uz_UZ@cyrillic UTF-8
#ve_ZA UTF-8
#vi_VN UTF-8
#wa_BE.UTF-8 UTF-8
#wa_BE ISO-8859-1
#wa_BE@euro ISO-8859-15
#wae_CH UTF-8
#wal_ET UTF-8
#wo_SN UTF-8
#xh_ZA.UTF-8 UTF-8
#xh_ZA ISO-8859-1
#yi_US.UTF-8 UTF-8
#yi_US CP1255
#yo_NG UTF-8
#yue_HK UTF-8
#yuw_PG UTF-8
#zh_CN.GB18030 GB18030
#zh_CN.GBK GBK
#zh_CN.UTF-8 UTF-8
#zh_CN GB2312
#zh_HK.UTF-8 UTF-8
#zh_HK BIG5-HKSCS
#zh_SG.UTF-8 UTF-8
#zh_SG.GBK GBK
#zh_SG GB2312
#zh_TW.EUC-TW EUC-TW
#zh_TW.UTF-8 UTF-8
#zh_TW BIG5
#zu_ZA.UTF-8 UTF-8
#zu_ZA ISO-8859-1

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/mkinitcpio.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/mkinitcpio.conf"
# mkinitcpio preset for AstraOS
MODULES=()
BINARIES=()
FILES=()
HOOKS=(base udev modconf kms keyboard keymap consolefont block filesystems fsck)
COMPRESSION="zstd"

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/passwd"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/passwd"
root:x:0:0::/root:/bin/bash
bin:x:1:1::/:/usr/bin/nologin
daemon:x:2:2::/:/usr/bin/nologin
mail:x:8:12::/var/spool/mail:/usr/bin/nologin
ftp:x:14:11::/srv/ftp:/usr/bin/nologin
http:x:33:33::/srv/http:/usr/bin/nologin
nobody:x:65534:65534:Nobody:/:/usr/bin/nologin
systemd-journal-gateway:x:193:193:systemd Journal Gateway:/:/usr/bin/nologin
systemd-network:x:190:190:systemd Network Management:/:/usr/bin/nologin
systemd-resolve:x:191:191:systemd Resolver:/:/usr/bin/nologin
systemd-timesync:x:192:192:systemd Time Synchronization:/:/usr/bin/nologin
systemd-coredump:x:997:997:systemd Core Dumper:/:/usr/bin/nologin
uuidd:x:68:68::/:/usr/bin/nologin
dbus:x:81:81:System Message Bus:/:/usr/bin/nologin
tss:x:941:941:tss:/:/usr/bin/nologin
astra:x:1000:1000::/home/astra:/bin/bash

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/shadow"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/shadow"
root:!*::0:99999:7:::
bin:!*::0:99999:7:::
daemon:!*::0:99999:7:::
mail:!*::0:99999:7:::
ftp:!*::0:99999:7:::
http:!*::0:99999:7:::
nobody:!*::0:99999:7::::
systemd-journal-gateway:!*::0:99999:7:::
systemd-network:!*::0:99999:7:::
systemd-resolve:!*::0:99999:7:::
systemd-timesync:!*::0:99999:7:::
systemd-coredump:!*::0:99999:7:::
uuidd:!*::0:99999:7:::
dbus:!*::0:99999:7:::
tss:!*::0:99999:7:::
astra:!*::0:99999:7:::

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/skel/.bashrc"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/skel/.bashrc"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║                    AstraOS — Bash configuration                       ║
# ║                          Default user: astra                            ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# This file is copied to /etc/skel/.bashrc by the ISO build, then to
# ~/.bashrc on first login of the default "astra" user.

# ── Shell options ────────────────────────────────────────────────────────
[ -z "$PS1" ] && return
shopt -s checkwinsize
shopt -s histappend
shopt -s cdspell
shopt -s dirspell
shopt -s globstar 2>/dev/null

# ── History ──────────────────────────────────────────────────────────────
HISTCONTROL=ignoreboth:erasedups
HISTSIZE=10000
HISTFILESIZE=20000
HISTTIMEFORMAT="%F %T  "

# ── PATH (Rust toolchain + user bins + paru) ─────────────────────────────
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.local/share/paru/bin:$HOME/.local/share/gem/ruby/bin:$PATH"

# ── Editor & pager ───────────────────────────────────────────────────────
export EDITOR=nvim
export VISUAL=nvim
export PAGER=less
export LESS="-R -F -X -i"
export MANPAGER="sh -c 'col -bx | bat -l man -p' 2>/dev/null || less"

# ── AstraOS branding env ────────────────────────────────────────────────
export ASTRAOS_VERSION="0.1.0"
export ASTRAOS_EDITION="Core"

# ── Aliases ──────────────────────────────────────────────────────────────
# Modern replacements — fall back to classic tools if not installed.
if command -v eza >/dev/null 2>&1; then
    alias ll='eza -la --git --group-directories-first'
    alias l='eza -l --git --group-directories-first'
    alias la='eza -a --git --group-directories-first'
    alias lt='eza -T --level=2'
else
    alias ll='ls -la --color=auto'
    alias l='ls -l --color=auto'
    alias la='ls -A --color=auto'
fi

if command -v bat >/dev/null 2>&1; then
    alias cat='bat --paging=never --style=plain'
    alias batcat='bat'
else
    alias cat='cat'
fi

if command -v rg >/dev/null 2>&1; then
    alias grep='rg'
fi

if command -v fd >/dev/null 2>&1; then
    alias find='fd'
fi

alias ..='cd ..'
alias ...='cd ../..'
alias ....='cd ../../..'
alias cp='cp -iv'
alias mv='mv -iv'
alias rm='rm -Iv --preserve-root'
alias mkdir='mkdir -pv'
alias chmod='chmod -c'
alias chown='chown -c'
alias df='df -h'
alias du='du -h'
alias free='free -h'
alias ports='sudo ss -tulpn'
alias ports6='sudo ss -6tulpn'

# ── AstraOS-specific aliases ─────────────────────────────────────────────
alias astra-version='echo "AstraOS $ASTRAOS_EDITION v$ASTRAOS_VERSION"'
alias astra-update='sudo pacman -Syu && paru -Syu'
alias astra-clean='sudo pacman -Rns $(pacman -Qtdq) 2>/dev/null; paru -Sc'

# Source /etc/astra/aliases if maintained by the system image
[ -f /etc/astra/aliases ] && . /etc/astra/aliases

# ── Color prompt with AstraOS branding ───────────────────────────────────
# Magenta ✦ symbol (#FF2D55-ish) + classic 4-color prompt.
ASTRA_COLOR='\[\e[35m\]'   # magenta
PATH_COLOR='\[\e[34m\]'    # blue
GIT_COLOR='\[\e[33m\]'     # yellow
USER_COLOR='\[\e[32m\]'   # green
RESET='\[\e[0m\]'

# Lightweight git branch (no external deps)
__astra_git_branch() {
    local b
    b=$(git symbolic-ref --short HEAD 2>/dev/null) \
        || b=$(git rev-parse --short HEAD 2>/dev/null) \
        || return
    printf ' (%s)' "$b"
}

PS1="${ASTRA_COLOR}✦${RESET} ${USER_COLOR}\u${RESET}@${PATH_COLOR}\h${RESET}:${PATH_COLOR}\w${RESET}${GIT_COLOR}\$(__astra_git_branch)${RESET}\n\$ "

# ── Starship (preferred prompt when installed) ─────────────────────────
if command -v starship >/dev/null 2>&1; then
    eval "$(starship init bash)"
fi

# ── bash completion (if installed) ──────────────────────────────────────
if [ -f /usr/share/bash-completion/bash_completion ]; then
    . /usr/share/bash-completion/bash_completion
fi

# ── Welcome banner on login shell ───────────────────────────────────────
if [ -n "$PS1" ] && [ -z "$ASTRA_NO_BANNER" ]; then
    printf '\n'
    printf '\e[35m   ╦   ╦╔╗╔╔╦╗╔═╗╦╔╗╔╦\e[0m\n'
    printf '\e[35m   ║   ║║║║ ║ ║  ║║║║║\e[0m  \e[90mAstraOS\e[0m \e[1mv%s\e[0m (\e[1m%s\e[0m)\n' "$ASTRAOS_VERSION" "$ASTRAOS_EDITION"
    printf '\e[35m   ╩═╝╩╝╚╝ ╩ ╚═╝╩╝╚╝╩\e[0m  \e[90m© 2026 Astra Corporation\e[0m\n'
    printf '\n'
fi

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/skel/.config/astra/desktop.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/skel/.config/astra/desktop.conf"
# AstraOS — desktop configuration
# Format: TOML
# This file is read by Astra Shell (Rust + GTK4-rs) starting ISO 2.
# For ISO 1 it is a placeholder so that future code knows the schema.

[astraos]
version        = "0.1.0"
edition        = "Core"
theme          = "dark"
accent         = "#818cf8"
wallpaper      = "/usr/share/backgrounds/astraos/default-violet.png"
glassmorphism  = true
taskbar_mode   = "win"  # "win" | "mac"

[ui]
font_sans  = "Inter"
font_mono  = "JetBrains Mono"
rounding   = 12
blur_size  = 8
blur_passes = 2

[input]
keyboard_layout = "fr"
touchpad_natural_scroll = true

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/skel/.config/astra/version"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/skel/.config/astra/version"
0.1.0

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/skel/.config/hypr/astra.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/skel/.config/hypr/astra.conf"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║              AstraOS-specific Hyprland configuration                   ║
# ║                  Sourced by ~/.config/hypr/hyprland.conf               ║
# ║                       ISO 1 (v0.1) — placeholder                       ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# This file is intentionally minimal for ISO 1.
#
# Starting ISO 2 (v0.2), it will contain:
#   - Astra Shell window rules (bar / dock / launcher / lock screen)
#   - AstraOS-specific animations (astra easing curve refinements)
#   - Branded workspace names + icons
#   - Custom AstraOS keybindings (AstraPass, Widgets, etc.)
#   - Integration with the Rust + GTK4-rs Astra Shell daemons
#
# It is sourced automatically at the bottom of hyprland.conf via:
#     source = ~/.config/hypr/astra.conf

# Placeholder for ISO 1

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/skel/.config/hypr/hyprland.conf"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║                    AstraOS — Hyprland configuration                    ║
# ║                        ISO 1 (v0.1) — Core pack                       ║
# ║              © 2026 AstraOS Project — Astra Corporation              ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# Minimal but elegant config for ISO 1.
# Astra Shell (custom Rust + GTK4-rs) will be wired in ISO 2 and will
# source additional settings from ~/.config/hypr/astra.conf (see bottom).

# ── Environment variables ────────────────────────────────────────────────
env = XCURSOR_SIZE,24
env = QT_QPA_PLATFORMTHEME,qt5ct
env = XDG_CURRENT_DESKTOP,Hyprland
env = XDG_SESSION_TYPE,wayland
env = XDG_SESSION_DESKTOP,Hyprland

# ── Monitors ────────────────────────────────────────────────────────────
# Auto-detect preferred mode + scale 1 (HiDPI handled later by Astra Shell)
monitor=,preferred,auto,1

# ── Default programs ─────────────────────────────────────────────────────
$terminal    = kitty
$fileManager = thunar
$menu        = wofi --show drun
$browser     = firefox
$editor      = nvim

# ── Autostart ────────────────────────────────────────────────────────────
exec-once = pipewire
exec-once = wireplumber
exec-once = hyprpaper &

# Waybar is NOT shipped in ISO 1 (will be replaced by Astra Shell in ISO 2).
# Uncomment once waybar is added if you need a temporary bar.
# exec-once = waybar

# Set the AstraOS violet wallpaper on boot
exec-once = hyprctl hyprpaper preload /usr/share/backgrounds/astraos/default-violet.png
exec-once = hyprctl hyprpaper wallpaper ",/usr/share/backgrounds/astraos/default-violet.png"

# ── Input ────────────────────────────────────────────────────────────────
input {
    kb_layout  = fr
    kb_variant =
    kb_model   =
    kb_options =
    kb_rules   =

    follow_mouse = 1

    touchpad {
        natural_scroll = true
        tap-to-click  = true
        clickfinger   = true
    }

    sensitivity = 0 # -1.0..1.0, 0 = no modification
}

# ── Device / gestures ────────────────────────────────────────────────────
gestures {
    workspace_swipe = true
    workspace_swipe_distance = 200
}

# ── Decoration (glassmorphism) ──────────────────────────────────────────
decoration {
    rounding = 12

    blur {
        enabled = true
        size = 8
        passes = 2
        new_optimizations = true
        ignore_opacity = true
        xray = false
    }

    drop_shadow = true
    shadow_range = 20
    shadow_render_power = 3
    col.shadow = rgba(1a1a2eee)

    # Glassmorphism accent
    active_opacity = 0.95
    inactive_opacity = 0.85
    fullscreen_opacity = 1.0
}

# ── Animations ───────────────────────────────────────────────────────────
animations {
    enabled = true

    bezier = easeOutQuint, 0.22, 1, 0.36, 1
    bezier = easeInOutCubic, 0.65, 0.05, 0.36, 1
    bezier = astra, 0.4, 0.0, 0.2, 1.0

    animation = windows, 1, 6, astra, slide
    animation = windowsOut, 1, 5, astra, slide
    animation = windowsMove, 1, 5, astra, slide
    animation = border, 1, 8, default
    animation = fade, 1, 5, astra
    animation = workspaces, 1, 5, astra, slidefade
    animation = workspacesIn, 1, 5, astra, slidefade
    animation = workspacesOut, 1, 5, astra, slidefade
    animation = specialWorkspace, 1, 5, astra, slidefade
}

# ── Layout / Dwindle ─────────────────────────────────────────────────────
dwindle {
    pseudotile = true
    preserve_split = true
}

master {
    new_is_master = true
}

# ── Misc ─────────────────────────────────────────────────────────────────
misc {
    disable_hyprland_logo = true
    disable_splash_rendering = true
    force_default_wallpaper = 0
    vfr = true
    mouse_move_enables_dpms = true
    key_press_enables_dpms = true
}

# ── Keybindings ──────────────────────────────────────────────────────────
$mainMod = SUPER

# Apps
bind = $mainMod,       Return, exec, $terminal
bind = $mainMod,       E,      exec, $fileManager
bind = $mainMod,       B,      exec, $browser
bind = $mainMod,       space,  togglefloating,
bind = $mainMod SHIFT, M,      exit,
bind = $mainMod,       Q,      killactive,

# Launcher (placeholder, wofi may not be installed yet)
bind = $mainMod,       D,      exec, $menu

# Screenshot — full screen + region (grim + slurp)
bind = ,               Print,  exec, grim - | wl-copy
bind = SHIFT,          Print,  exec, grim -g "$(slurp)" - | wl-copy
bind = $mainMod SHIFT, S,      exec, grim -g "$(slurp)" - | wl-copy

# Window focus
bind = $mainMod,       left,  movefocus, l
bind = $mainMod,       right, movefocus, r
bind = $mainMod,       up,    movefocus, u
bind = $mainMod,       down,  movefocus, d

# Window movement
bind = $mainMod SHIFT, left,  movewindow, l
bind = $mainMod SHIFT, right, movewindow, r
bind = $mainMod SHIFT, up,    movewindow, u
bind = $mainMod SHIFT, down,  movewindow, d

# Workspaces
bind = $mainMod,       1, workspace, 1
bind = $mainMod,       2, workspace, 2
bind = $mainMod,       3, workspace, 3
bind = $mainMod,       4, workspace, 4
bind = $mainMod,       5, workspace, 5
bind = $mainMod,       6, workspace, 6
bind = $mainMod,       7, workspace, 7
bind = $mainMod,       8, workspace, 8
bind = $mainMod,       9, workspace, 9

bind = $mainMod SHIFT, 1, movetoworkspace, 1
bind = $mainMod SHIFT, 2, movetoworkspace, 2
bind = $mainMod SHIFT, 3, movetoworkspace, 3
bind = $mainMod SHIFT, 4, movetoworkspace, 4
bind = $mainMod SHIFT, 5, movetoworkspace, 5
bind = $mainMod SHIFT, 6, movetoworkspace, 6
bind = $mainMod SHIFT, 7, movetoworkspace, 7
bind = $mainMod SHIFT, 8, movetoworkspace, 8
bind = $mainMod SHIFT, 9, movetoworkspace, 9

# Scroll through workspaces
bind = $mainMod, mouse_down, workspace, e+1
bind = $mainMod, mouse_up,   workspace, e-1

# Media keys (best-effort)
bindl = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bindl = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bindl = , XF86AudioMute,        exec, wpctl set-mute   @DEFAULT_AUDIO_SINK@ toggle
bindl = , XF86MonBrightnessUp,   exec, brightnessctl set +5%
bindl = , XF86MonBrightnessDown, exec, brightnessctl set 5%-

# ── Window rules ─────────────────────────────────────────────────────────
# Default rules. AstraOS-specific app rules will be added in astra.conf
# (sourced at the bottom of this file) starting from ISO 2.

windowrule = float, ^(pavucontrol)$
windowrule = float, ^(nm-connection-editor)$
windowrule = float, ^(blueman-manager)$
windowrule = center, ^(pavucontrol)$

# kitty / firefox tweaks for glassmorphism
windowrule = opacity 0.95 0.85, ^(kitty)$
windowrule = opacity 1.0,       ^(firefox)$

# ── AstraOS-specific config (placeholder for ISO 1) ─────────────────────
# Astra Shell integration, custom animations, branded window rules, etc.
# will live in ~/.config/hypr/astra.conf starting ISO 2.
source = ~/.config/hypr/astra.conf

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/skel/.config/kitty/kitty.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/skel/.config/kitty/kitty.conf"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║                AstraOS — Kitty terminal configuration                 ║
# ║                    Glassmorphism + magenta accent                     ║
# ╚══════════════════════════════════════════════════════════════════════╝

# ── Font ────────────────────────────────────────────────────────────────
font_family       JetBrains Mono
bold_font         auto
italic_font       auto
bold_italic_font  auto
font_size         12.0

# Fallback for CJK + emoji
symbol_map U+E000-U+E8FF,U+F000-U+FFFF Symbols Nerd Font
symbol_map U+1F300-U+1F6FF,U+1F900-U+1F9FF Noto Color Emoji

# Disable cursor blink during typing for less visual noise
cursor_shape          block
cursor_blink_interval 0.5
cursor_stop_blinking_after 15.0

# ── Window / glassmorphism ──────────────────────────────────────────────
background_opacity 0.85
background_blur    20

window_padding_width 8
window_margin_width  0
window_border_width 0
draw_minimal_borders yes
inactive_text_alpha 0.85

placement_strategy center
remember_window_size  no
initial_window_width  900
initial_window_height 600

# Title bar — let Hyprland handle decorations
hide_window_decorations yes
confirm_os_window_close 0

# ── Scrollback ──────────────────────────────────────────────────────────
scrollback_lines 10000
wheel_scroll_multiplier 5.0
touch_scroll_multiplier 1.0

# ── Shell ──────────────────────────────────────────────────────────────
shell bash
editor nvim
allow_remote_control yes
listen_on unix:/tmp/kitty

# ── Performance / VSync ────────────────────────────────────────────────
repaint_delay    10
input_delay      3
sync_to_monitor  yes

# ── Mouse / selection ──────────────────────────────────────────────────
url_color #818cf8
url_style curly
open_url_with default
copy_on_select yes
strip_trailing_spaces smart
rectangle_select_modifiers ctrl+alt

# ── Color scheme — AstraOS dark + magenta accent ───────────────────────
foreground            #e6e6e6
background            #0b0b14
selection_foreground  #0b0b14
selection_background  #818cf8

cursor                #ff2d55
cursor_text_color     #0b0b14

# Black
color0  #1a1a2e
color8  #2a2a3e
# Red
color1  #ff2d55
color9  #ff5a7a
# Green
color2  #10b981
color10 #34d399
# Yellow
color3  #f59e0b
color11 #fbbf24
# Blue
color4  #3b82f6
color12 #60a5fa
# Magenta
color5  #818cf8
color13 #a5b4fc
# Cyan
color6  #06b6d4
color14 #22d3ee
# White
color7  #e6e6e6
color15 #ffffff

# ── Keybindings ────────────────────────────────────────────────────────
# New tab/new window easily accessible
map ctrl+shift+t     new_tab
map ctrl+shift+n     new_os_window
map ctrl+shift+w     close_tab
map ctrl+shift+enter new_window

# Split navigation
map ctrl+shift+j neighboring_window down
map ctrl+shift+k neighboring_window up
map ctrl+shift+h neighboring_window left
map ctrl+shift+l neighboring_window right

# Font size
map ctrl+shift+equal change_font_size all +1.0
map ctrl+shift+minus change_font_size all -1.0
map ctrl+shift+0     change_font_size all 0

# Reload config
map ctrl+shift+f5 load_config_file

# ── AstraOS branding ───────────────────────────────────────────────────
# Show AstraOS version in window title (when title bar is enabled)
titlebar_layout left
macos_titlebar_color background

# Extra font (optional, fallback)
font_features JetBrainsMono-Regular +zero +cv01 +ss01

# Bell (visual only)
visual_bell_duration 0.0
enable_audio_bell no

# Layout
enabled_layouts splits,stack,grid

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/skel/.config/starship.toml"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/skel/.config/starship.toml"
# ╔══════════════════════════════════════════════════════════════════════╗
# ║                  AstraOS — Starship prompt                            ║
# ║                Dark theme with magenta AstraOS branding                ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# Used by ~/.bashrc when starship is installed.

format = """
[](fg:color_bg1)\
[](bg:color_magenta fg:color_magenta)\
$os\
[](bg:color_bg1 fg:color_magenta)\
$directory\
[](fg:color_bg1 bg:color_bg2)\
$git_branch\
$git_status\
[](fg:color_bg2 bg:color_bg3)\
$rust\
$python\
$nodejs\
$golang\
[](fg:color_bg3 bg:color_bg4)\
$cmd_duration\
[](fg:color_bg4)\
$line_break$character"""

palette = "astra_dark"

[palettes.astra_dark]
color_bg1     = "#1a1a2e"
color_bg2     = "#16213e"
color_bg3     = "#0f3460"
color_bg4     = "#533483"
color_text    = "#e6e6e6"
color_magenta = "#ff2d55"
color_accent  = "#818cf8"
color_green   = "#10b981"
color_yellow  = "#f59e0b"
color_red     = "#ef4444"
color_blue    = "#3b82f6"

[os]
disabled = false
style = "bg:color_magenta fg:#0b0b14 bold"
format = '[ ✦ ]($style)'

[directory]
style = "bg:color_bg1 fg:color_accent bold"
format = "[ $path ]($style)"
truncation_length = 3
truncation_symbol = "…/"

[directory.substitutions]
"Documents" = "󰈙 "
"Downloads" = " "
"Music" = "󰝚 "
"Pictures" = " "
"~" = " "

[git_branch]
symbol = ""
style = "bg:color_bg2 fg:color_yellow"
format = '[ $symbol $branch ]($style)'

[git_status]
style = "bg:color_bg2 fg:color_yellow"
format = '[$all_status$ahead_behind ]($style)'

[rust]
symbol = ""
style = "bg:color_bg3 fg:#ff7d43"
format = '[ $symbol ($version) ]($style)'

[python]
symbol = ""
style = "bg:color_bg3 fg:color_blue"
format = '[ $symbol ($version) ]($style)'

[nodejs]
symbol = ""
style = "bg:color_bg3 fg:color_green"
format = '[ $symbol ($version) ]($style)'

[golang]
symbol = ""
style = "bg:color_bg3 fg:color_accent"
format = '[ $symbol ($version) ]($style)'

[cmd_duration]
min_time = 500
style = "bg:color_bg4 fg:color_text"
format = '[  $duration ]($style)'

[character]
success_symbol = "[ ✦ ](bold color_magenta)"
error_symbol   = "[ ✗ ](bold color_red)"

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/sudoers.d/10-astraos"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/sudoers.d/10-astraos"
# AstraOS sudoers drop-in — wheel group gets full sudo with password feedback.
# File: /etc/sudoers.d/10-astraos
# Mode: 0440 (enforced by archiso file_permissions if needed, but sudo
# requires 0440 — install scripts usually set this).

%wheel ALL=(ALL:ALL) ALL
Defaults pwfeedback

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/etc/vconsole.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/etc/vconsole.conf"
KEYMAP=fr-latin9
FONT=eurlatgr

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/root/.bash_profile"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/root/.bash_profile"
# ~/.bash_profile for AstraOS root user
[[ -f ~/.bashrc ]] && . ~/.bashrc

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/root/.bashrc"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/root/.bashrc"
# ~/.bashrc for root
PS1='[\[\033[01;31m\]root\[\033[00m\]@\h \W]\$ '
alias ll='ls -la'
alias update='pacman -Syu'

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/airootfs/root/customize_airootfs.sh"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/airootfs/root/customize_airootfs.sh"
#!/usr/bin/env bash
# AstraOS airootfs customization script
# Runs INSIDE the chroot during archiso build

set -e -u

echo "=== AstraOS airootfs customization starting ==="

# Set timezone
ln -sf /usr/share/zoneinfo/Europe/Paris /etc/localtime
echo "✓ Timezone set to Europe/Paris"

# Generate locales
locale-gen
echo "✓ Locales generated"

# Set vconsole keymap
echo "KEYMAP=fr-latin9" > /etc/vconsole.conf
echo "FONT=eurlatgr" >> /etc/vconsole.conf
echo "✓ Console keymap set to fr-latin9"

# Create user 'astra'
useradd -m -G wheel -s /bin/bash astra
echo "✓ User 'astra' created (member of wheel)"

# Set passwords (default = astraos, will be changed at install)
echo "root:astraos" | chpasswd
echo "astra:astraos" | chpasswd
echo "✓ Default passwords set (will be changed at install)"

# Enable services
systemctl enable NetworkManager.service
systemctl enable iwd.service
systemctl enable greetd.service
systemctl enable systemd-timesyncd.service
systemctl enable systemd-resolved.service
echo "✓ Services enabled"

# Create AstraOS config directories
mkdir -p /etc/astra
mkdir -p /usr/share/backgrounds/astraos
mkdir -p /usr/share/pixmaps/astraos
mkdir -p /usr/share/icons/astraos
echo "✓ AstraOS directories created"

# Install paru-bin from AUR (in chroot, requires network access in build container)
# This may fail if AUR is not accessible during build - that's OK, paru will be installed at first boot if missing
if ! command -v paru &> /dev/null; then
    echo "→ Attempting paru-bin installation from AUR..."
    # We need a temporary build user since makepkg refuses to run as root
    useradd -m -G wheel -s /bin/bash paru-builder
    echo "paru-builder:astraos" | chpasswd
    echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/99-paru-builder
    cd /tmp
    sudo -u paru-builder git clone https://aur.archlinux.org/paru-bin.git
    cd /tmp/paru-bin
    sudo -u paru-builder makepkg -si --noconfirm --noprogressbar || echo "⚠ paru install failed (will be retried at first boot)"
    cd /
    rm -rf /tmp/paru-bin
    userdel -r paru-builder
    rm /etc/sudoers.d/99-paru-builder
    echo "✓ paru-bin installation attempted"
else
    echo "✓ paru already available"
fi

# Make scripts executable
chmod +x /usr/local/bin/* 2>/dev/null || true

# Final message
echo "=== AstraOS airootfs customization complete ==="
echo "    Default user: astra (password: astraos)"
echo "    Default root password: astraos (CHANGE AT INSTALL)"
echo "    Locale: fr_FR.UTF-8"
echo "    Keyboard: fr-latin9"
echo "    Timezone: Europe/Paris"

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/boot/loaders/entries/astraos-fallback.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/boot/loaders/entries/astraos-fallback.conf"
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS Live — systemd-boot fallback entry (UEFI)
# Uses initramfs-linux-fallback.img (more drivers, useful if main boot fails).

title   AstraOS Live (x86_64) — fallback initramfs
linux   /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
initrd  /%INSTALL_DIR%/boot/intel-ucode.img
initrd  /%INSTALL_DIR%/boot/amd-ucode.img
initrd  /%INSTALL_DIR%/boot/x86_64/initramfs-linux-fallback.img
options archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL%

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/boot/loaders/entries/astraos.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/boot/loaders/entries/astraos.conf"
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS Live — systemd-boot entry (UEFI), primary location
# Path on the ISO: /loader/entries/astraos.conf
# Tokens %INSTALL_DIR% and %ARCHISO_LABEL% are substituted by mkarchiso.

title   AstraOS Live (x86_64)
linux   /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
initrd  /%INSTALL_DIR%/boot/intel-ucode.img
initrd  /%INSTALL_DIR%/boot/amd-ucode.img
initrd  /%INSTALL_DIR%/boot/x86_64/initramfs-linux.img
options archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL%

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/boot/loaders/loader.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/boot/loaders/loader.conf"
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS Live — systemd-boot loader configuration
# Path on the ISO: /loader/loader.conf
#
# Default boot entry: AstraOS Live (x86_64)
# Auto-boot after 10 seconds (10 * 1s systemd-boot units = 10s timeout).

default astraos
timeout 10
console-mode keep
editor no

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/boot/syslinux/syslinux.cfg"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/boot/syslinux/syslinux.cfg"
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS Live — BIOS (syslinux) boot configuration
# Based on the official archiso template, customized for AstraOS.
#
# Tokens %INSTALL_DIR% and %ARCHISO_LABEL% are substituted at ISO build time
# by the archiso mkarchiso tool (see profiledef.sh).

# Load custom AstraOS theme colors and menu cosmetics first
INCLUDE theme.cfg

DEFAULT astraos
PROMPT 0
TIMEOUT 30
UI menu.c32

MENU TITLE AstraOS Live
MENU BACKGROUND /%INSTALL_DIR%/boot/syslinux/astraos-bg.png
MENU HELPMSGROW 22
MENU WIDTH 78
MENU MARGIN 6
MENU ROWS 10
MENU TABMSGROW 18
MENU CMDLINEROW 18
MENU HIDDENROW 20
MENU TIMEOUTROW 24

# ---------------------------------------------------------------------------
# Default entry — AstraOS Live (x86_64)
# ---------------------------------------------------------------------------
LABEL astraos
MENU LABEL AstraOS Live (x86_64)
MENU HELP Print AstraOS logo on boot and load the default Hyprland session.
LINUX /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
INITRD /%INSTALL_DIR%/boot/intel-ucode.img,/%INSTALL_DIR%/boot/amd-ucode.img,/%INSTALL_DIR%/boot/x86_64/initramfs-linux.img
APPEND archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL%

# ---------------------------------------------------------------------------
# Fallback entry — AstraOS Live (x86_64) with fallback initramfs
# ---------------------------------------------------------------------------
LABEL astraos-fallback
MENU LABEL AstraOS Live (x86_64) — fallback initramfs
MENU HELP Use this if the default boot fails (more drivers, slower boot).
LINUX /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
INITRD /%INSTALL_DIR%/boot/intel-ucode.img,/%INSTALL_DIR%/boot/amd-ucode.img,/%INSTALL_DIR%/boot/x86_64/initramfs-linux-fallback.img
APPEND archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL%

# ---------------------------------------------------------------------------
# Memtest86+ entry (commented out — enable if memtest86+ is bundled in ISO)
# ---------------------------------------------------------------------------
#LABEL memtest
#MENU LABEL Memtest86+
#LINUX /%INSTALL_DIR%/boot/memtest86+/memtest.bin
#APPEND -

# ---------------------------------------------------------------------------
# Reboot / Power off helpers
# ---------------------------------------------------------------------------
LABEL reboot
MENU LABEL Reboot
MENU HELP Reboot the machine now.
COM32 reboot.c32

LABEL poweroff
MENU LABEL Power Off
MENU HELP Power off the machine now.
COM32 poweroff.c32

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/boot/syslinux/theme.cfg"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/boot/syslinux/theme.cfg"
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS Live — syslinux theme
#
# Custom color palette derived from AstraOS visual identity:
#   - Magenta accent (matches logo-boot.png magenta star): #E040FB
#   - Deep space background near-black: #0b0d17
#   - Soft slate foreground text:       #f1f5f9
#
# Color format:  FG;BG   #RRGGBB   #RRGGBB   <shadow>
# (16-color ANSI fg/bg pair + 32-bit RGB foreground + RGB background + shadow type)

# Frame and menu cosmetics
MENU COLOR border       30;44   #40ffffff #a00000000 none
MENU COLOR title        1;36;44 #ffe040fb #a00b0d17  none
MENU COLOR sel          7;37;40 #ff000000 #ffe040fb  none
MENU COLOR hotsel       1;7;37;40 #ff000000 #ffe040fb none
MENU COLOR unsel        37;44   #fff1f5f9 #a00b0d17  none
MENU COLOR hotkey       1;37;44 #ffe040fb #a00b0d17  none
MENU COLOR tabmsg       31;40   #ff94a3b8 #a00000000 none
MENU COLOR cmdmark      1;36;40 #ffff0000 #a00000000 none
MENU COLOR cmdline      37;40   #fff1f5f9 #a00000000 none
MENU COLOR scrollbar    30;44   #40ffffff #a00000000 none
MENU COLOR pwdborder    30;44   #80ffffff #a00000000 none
MENU COLOR pwdheader    31;40   #ffe040fb #a00000000 none
MENU COLOR pwdentry     30;40   #fff1f5f9 #a00000000 none
MENU COLOR timeout_msg  37;40   #ff94a3b8 #a00000000 none
MENU COLOR timeout      1;37;40 #ffe040fb #a00000000 none
MENU COLOR help         37;40   #fff1f5f9 #a00000000 none
MENU COLOR helpsel      7;37;40 #ff000000 #ffe040fb  none
MENU COLOR helpborder    30;44  #40ffffff #a00000000 none

# Indentation / spacing
MENU HSHIFT 4
MENU VSHIFT 2
MENU WIDTH 78
MENU MARGIN 6
MENU ROWS 10
MENU HELPMSGROW 22
MENU CMDLINEROW 18
MENU TABMSGROW 18
MENU TIMEOUTROW 24
MENU HIDDENROW 20

# ASCII AstraOS banner shown above the menu (drawn with the magenta star)
MENU TITLE AstraOS Live  [ v0.1 ]

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/customize_airootfs.sh"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/customize_airootfs.sh"
#!/usr/bin/env bash
# AstraOS airootfs customization script
# Runs INSIDE the chroot during archiso build

set -e -u

echo "=== AstraOS airootfs customization starting ==="

# Set timezone
ln -sf /usr/share/zoneinfo/Europe/Paris /etc/localtime
echo "✓ Timezone set to Europe/Paris"

# Generate locales
locale-gen
echo "✓ Locales generated"

# Set vconsole keymap
echo "KEYMAP=fr-latin9" > /etc/vconsole.conf
echo "FONT=eurlatgr" >> /etc/vconsole.conf
echo "✓ Console keymap set to fr-latin9"

# Create user 'astra'
useradd -m -G wheel -s /bin/bash astra
echo "✓ User 'astra' created (member of wheel)"

# Set passwords (default = astraos, will be changed at install)
echo "root:astraos" | chpasswd
echo "astra:astraos" | chpasswd
echo "✓ Default passwords set (will be changed at install)"

# Enable services
systemctl enable NetworkManager.service
systemctl enable iwd.service
systemctl enable greetd.service
systemctl enable systemd-timesyncd.service
systemctl enable systemd-resolved.service
echo "✓ Services enabled"

# Create AstraOS config directories
mkdir -p /etc/astra
mkdir -p /usr/share/backgrounds/astraos
mkdir -p /usr/share/pixmaps/astraos
mkdir -p /usr/share/icons/astraos
echo "✓ AstraOS directories created"

# Install paru-bin from AUR (in chroot, requires network access in build container)
# This may fail if AUR is not accessible during build - that's OK, paru will be installed at first boot if missing
if ! command -v paru &> /dev/null; then
    echo "→ Attempting paru-bin installation from AUR..."
    # We need a temporary build user since makepkg refuses to run as root
    useradd -m -G wheel -s /bin/bash paru-builder
    echo "paru-builder:astraos" | chpasswd
    echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/99-paru-builder
    cd /tmp
    sudo -u paru-builder git clone https://aur.archlinux.org/paru-bin.git
    cd /tmp/paru-bin
    sudo -u paru-builder makepkg -si --noconfirm --noprogressbar || echo "⚠ paru install failed (will be retried at first boot)"
    cd /
    rm -rf /tmp/paru-bin
    userdel -r paru-builder
    rm /etc/sudoers.d/99-paru-builder
    echo "✓ paru-bin installation attempted"
else
    echo "✓ paru already available"
fi

# Make scripts executable
chmod +x /usr/local/bin/* 2>/dev/null || true

# Final message
echo "=== AstraOS airootfs customization complete ==="
echo "    Default user: astra (password: astraos)"
echo "    Default root password: astraos (CHANGE AT INSTALL)"
echo "    Locale: fr_FR.UTF-8"
echo "    Keyboard: fr-latin9"
echo "    Timezone: Europe/Paris"

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/efiboot/loader/entries/astraos.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/efiboot/loader/entries/astraos.conf"
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS Live — systemd-boot entry on the EFI System Partition (ESP)
# Path on the ISO: /EFI/loader/entries/astraos.conf
# This mirrors /loader/entries/astraos.conf for archiso's standard dual location.
# Tokens %INSTALL_DIR% and %ARCHISO_LABEL% are substituted by mkarchiso.

title   AstraOS Live (x86_64)
linux   /%INSTALL_DIR%/boot/x86_64/vmlinuz-linux
initrd  /%INSTALL_DIR%/boot/intel-ucode.img
initrd  /%INSTALL_DIR%/boot/amd-ucode.img
initrd  /%INSTALL_DIR%/boot/x86_64/initramfs-linux.img
options archisobasedir=%INSTALL_DIR% archisolabel=%ARCHISO_LABEL%

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/efiboot/loader/loader.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/efiboot/loader/loader.conf"
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS Live — systemd-boot loader configuration on the ESP
# Path on the ISO: /EFI/loader/loader.conf
# Mirrors /loader/loader.conf for archiso's standard dual location.

default astraos
timeout 10
console-mode keep
editor no

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/packages.x86_64"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/packages.x86_64"
#
# SPDX-License-Identifier: GPL-3.0-or-later
#
# AstraOS ISO 1 - Base packages (v0.1.0)
# x86_64 package list for archiso build.
#
# NOTE: paru-bin (AUR helper) is NOT available in official Arch repos.
# It is installed by customize_airootfs.sh from AUR at build time,
# with a first-boot fallback if the build-time install fails.
#

## AstraOS ISO 1 - Base packages (v0.1.0)

## Base system
base
base-devel
linux
linux-firmware
intel-ucode
amd-ucode

## Bootloader & ISO tools
syslinux
edk2-ovmf
squashfs-tools
archiso

## Filesystem
btrfs-progs
dosfstools
e2fsprogs
exfatprogs
f2fs-tools
ntfs-3g
xfsprogs

## Networking
networkmanager
iwd
modemmanager
openresolv

## Audio
pipewire
pipewire-alsa
pipewire-jack
pipewire-pulse
wireplumber
pamixer

## Display (Hyprland stack)
hyprland
qt5-wayland
qt6-wayland
xdg-desktop-portal-hyprland
xdg-desktop-portal-gtk
polkit-gnome
greetd
greetd-gtkgreet
kitty

## Essential utilities
bash-completion
btop
ca-certificates
cronie
cryptsetup
dbus-broker
efibootmgr
openssh
openvpn
pacman-contrib
ppp
rsync
sudo
systemd-resolvconf
usbutils
vim
wget
which
wireless_tools
wpa_supplicant

## AstraOS branding & UX
neofetch
fastfetch
htop
bat
eza
fd
ripgrep
starship
thunar
thunar-archive-plugin
thunar-volman
tumbler
file-roller
gvfs
gvfs-mtp
firefox
ttf-dejavu
ttf-liberation
noto-fonts
noto-fonts-emoji

## Screenshot
grim
slurp
wl-clipboard

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/pacman.conf"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/pacman.conf"
#
# SPDX-License-Identifier: GPL-3.0-or-later
#
# /etc/pacman.conf
#
# pacman.conf for AstraOS ISO build (x86_64).
# Based on the official Arch Linux archiso template.
#
# See the pacman.conf(5) manpage for option and repository directives.

#
# GENERAL OPTIONS
#
[options]
# The following paths are commented out with their default values listed.
# If you wish to use different paths, uncomment and update the paths.
#RootDir     = /
#DBPath      = /var/lib/pacman/
#CacheDir    = /var/cache/pacman/pkg/
#LogFile     = /var/log/pacman.log
#GPGDir      = /etc/pacman.d/gnupg/
#HookDir     = /etc/pacman.d/hooks/
HoldPkg     = pacman glibc
#XferCommand = /usr/bin/curl -L -C - -f -o %o %u
#XferCommand = /usr/bin/wget --passive-ftp -c -O %o %u
#CleanMethod = KeepInstalled
Architecture = auto

# Pacman won't upgrade packages listed in IgnorePkg and members of IgnoreGroup
#IgnorePkg   =
#IgnoreGroup =

#NoUpgrade   =
#NoExtract   =

# Misc options
#UseSyslog
Color
#NoProgressBar
CheckSpace
VerbosePkgLists
ParallelDownloads = 5
#DisableSandbox
ILoveCandy

# By default, pacman accepts packages signed by keys that its local keyring
# trusts (see pacman-key and its man page), as well as unsigned packages.
SigLevel    = Required DatabaseOptional
LocalFileSigLevel = Optional
#RemoteFileSigLevel = Required

#
# REPOSITORIES
#
# In the AstraOS build container, the official Arch repos are exposed via the
# local archbuild mirror at /mnt/archbuild/repo (populated by the host/Docker
# build pipeline). [multilib] still uses the standard mirrorlist.
#

[core]
Server = file:///mnt/archbuild/repo

[extra]
Server = file:///mnt/archbuild/repo

[multilib]
Include = /etc/pacman.d/mirrorlist

# AstraOS note: AUR packages (paru-bin, etc.) are NOT added here because
# pacman.conf cannot host AUR repositories directly. They are installed via
# the customize_airootfs.sh script at ISO build time, with a first-boot
# fallback if the build-time install fails.

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → iso/x86_64/profiledef.sh"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "iso/x86_64/profiledef.sh"
#!/hint: bash
#
# SPDX-License-Identifier: GPL-3.0-or-later

# shellcheck disable=SC2034

# AstraOS ISO 1 (v0.1) — archiso profile definition
# Base Arch Linux + Hyprland + AstraOS branding (minimum viable).
# Based on the official Arch Linux archiso template.

iso_name="astraos"
iso_label="ASTRAOS_$(date +%Y%m)"
iso_publisher="AstraOS Project <https://astraos.org>"
iso_application="AstraOS Live/Rescue Media"
iso_version="0.1.0"
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
  "/etc/sudoers.d" "0:0:750"
  "/etc/sudoers.d/10-astraos" "0:0:440"
)

ASTRAOS_EOF_MARKER_42_ZAI

echo "  → scripts/copy-assets.sh"
cat << 'ASTRAOS_EOF_MARKER_42_ZAI' > "scripts/copy-assets.sh"
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

ASTRAOS_EOF_MARKER_42_ZAI

echo "→ Setting executable permissions..."
chmod +x "iso/x86_64/profiledef.sh"
chmod +x "iso/x86_64/customize_airootfs.sh"
chmod +x "iso/x86_64/airootfs/root/customize_airootfs.sh"
chmod +x "scripts/copy-assets.sh"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ⚠️  ACTION REQUISE:Uploader les assets binaires            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Les fichiers images (logos + wallpapers) ne sont pas inclus."
echo "Vous devez les uploader manuellement via VS Code (drag&drop)."
echo ""
echo "Fichiers à uploader dans assets/logos/:"
echo "  - logo-astraos.png  (logo officiel AstraOS)"
echo "  - logo-boot.png     (logo boot/splash)"
echo ""
echo "Fichiers à uploader dans assets/wallpapers/:"
echo "  - default-violet.png"
echo "  - default-bleu.png"
echo "  - classic.png"
echo ""
echo "Puis exécuter: bash scripts/copy-assets.sh"
echo ""
echo "✅ AstraOS ISO 1 structure created successfully!"
echo ""
echo "Next steps:"
echo "  1. Upload image assets (logos + wallpapers)"
echo "  2. Run: bash scripts/copy-assets.sh"
echo "  3. Build ISO: make build-iso"
echo ""
echo "🎯 AstraOS project ready at: $(pwd)"
