# Astra Photos

A native image viewer for AstraOS built with Rust + GTK4-rs.

## Features

- **Open** — pick any image via `gtk4::FileChooserDialog`
  - Supported types: PNG, JPEG, WebP, GIF
- **GtkPicture** renderer with aspect-ratio preservation
- **Zoom In / Zoom Out / Reset Zoom**
  - Zoom step: 1.25×
  - Range: 0.5× → 4.0×
- **Previous / Next** — scan the parent folder and step through all
  supported images (alphabetically sorted, wraps around)
- Live status bar showing loaded file path and current zoom factor
- Dark glassmorphism theme matching the AstraOS visual identity

## Requirements

- GTK 4.10+ (gtk4-rs 0.9)
- `gdk-pixbuf` loaders for png / jpeg / webp / gif (usually shipped with GTK)

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run
# or open a file directly via the desktop file:
astra-photos /path/to/image.png
```

## Tech Stack

- Language: Rust 2021
- UI toolkit: gtk4-rs 0.9
- File dialog: gtk4::FileChooserDialog
- Image rendering: gtk4::Picture + gdk::Texture
- Logging: env_logger
- App ID: `com.astraos.Photos`
