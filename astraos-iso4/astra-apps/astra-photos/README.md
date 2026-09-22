# Astra Photos

A lightweight image viewer for **AstraOS** (ISO 3 / v0.3), built with
Rust + GTK4-rs.

## Features

- **Open** — file chooser filtered to PNG / JPEG / WebP / GIF.
- **Zoom In / Out** — scale × 1.25 per step, clamped to 0.5× .. 4×.
- **Reset Zoom** — back to 1×.
- **Previous / Next** — scan the parent folder for sibling images and
  walk through them (wraps around at the ends).
- Image is rendered in a `GtkPicture` inside a `GtkScrolledWindow`,
  with shared state held in `Rc<RefCell<AppState>>`.
- 900x600 glassmorphism-dark window, CSS class `astra-photos`,
  accent `#818cf8`.

## Build

```sh
cargo build --release
```

## Run

```sh
astra-photos
```

Open any image file from the toolbar — the parent folder is scanned
automatically so Previous / Next work immediately.
