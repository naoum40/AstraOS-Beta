# Astra Sticky Notes

A floating-notes desktop app for **AstraOS** (ISO 4 / v0.4), built with
Rust + GTK4-rs.

## Features

- Each note is a borderless, draggable `ApplicationWindow` on the desktop.
- Six pastel color backgrounds:
  - yellow `#fef08a`
  - red `#fca5a5`
  - blue `#a5b4fc`
  - green `#86efac`
  - purple `#d8b4fe`
  - cyan `#67e8f9`
- Per-note editable title (`Entry`) + body (`TextView`, word-wrapped).
- 6-color dot picker to recolor a note instantly.
- "+" new-note button (spawns an offset window), "x" close button.
- Drag-to-move via a `GestureDrag` on the header row which calls
  `gdk::Toplevel::begin_move` on the underlying surface (the
  GTK4-native way to begin a window move).
- All state is persisted to `~/.config/astra/sticky-notes.json` on every
  change (title / body / color / resize / close).
- On startup, every saved note gets its own window restored.

## File layout

```
astra-sticky-notes/
├── Cargo.toml
├── src/main.rs
├── assets/css/sticky-notes.css
├── astra-sticky-notes.desktop
└── README.md
```

## Build

```sh
cargo build --release
```

## Run

```sh
astra-sticky-notes
```

Notes persist to `~/.config/astra/sticky-notes.json`.
