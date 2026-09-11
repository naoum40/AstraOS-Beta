# Astra Screenshot

A native Wayland screenshot tool for AstraOS built with Rust + GTK4-rs.

## Features

- **Capture Full Screen** — full output capture via `grim`
- **Capture Region** — interactive region selection via `slurp | grim -g -`
- **Capture Window** — focused Hyprland window via `hyprctl -j activewindow`
- Timestamped PNG files saved to `~/Pictures/Screenshot_YYYYMMDD_HHMMSS.png`
- Desktop notification on save via GIO notifications
- Dark glassmorphism theme matching the AstraOS visual identity

## Requirements

- GTK 4.10+ (gtk4-rs 0.9)
- `grim` — Wayland image capture
- `slurp` — region selector (for region mode)
- `hyprctl` — Hyprland IPC (for window mode)
- Hyprland compositor (or compatible wlroots compositor)

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run
# or after install:
astra-screenshot
```

## Tech Stack

- Language: Rust 2021
- UI toolkit: gtk4-rs 0.9
- Time: chrono
- Logging: env_logger
- App ID: `com.astraos.Screenshot`
