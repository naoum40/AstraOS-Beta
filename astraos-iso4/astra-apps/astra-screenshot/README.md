# Astra Screenshot

A small screenshot capture tool for **AstraOS** (ISO 3 / v0.3), built with
Rust + GTK4-rs.

## Features

- Three capture modes:
  - **Full Screen** — `grim -t png ~/Pictures/Screenshot_<ts>.png`
  - **Region** — `slurp | grim -g - ~/Pictures/Screenshot_<ts>.png`
  - **Window** — `hyprctl -j activewindow` + `jq` + `grim -l <addr> ...`
- Timestamp format: `YYYYMMDD_HHMMSS` (chrono).
- Non-blocking: shell pipelines run on worker threads via
  `std::process::Command`; the GTK UI never stalls.
- GIO desktop notification once a file is saved to `~/Pictures/`.
- 400x300, undecorated, glassmorphism-dark window with the
  `astra-screenshot` CSS class (theme: accent `#818cf8`).

## Build

```sh
cargo build --release
```

Runtime deps (AstraOS ships them all): `grim`, `slurp`, `hyprctl`, `jq`.

## Run

```sh
astra-screenshot
```

Output is written to `~/Pictures/Screenshot_<timestamp>.png`.
