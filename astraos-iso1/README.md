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
