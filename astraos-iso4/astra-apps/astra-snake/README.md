# Astra Snake

Classic Snake game for **AstraOS** (ISO 4 / v0.4), built with
Rust + GTK4-rs + cairo.

## Features

- 20x20 grid (each cell 18px → 360x360 canvas) drawn on a
  `gtk4::DrawingArea` via a cairo draw function.
- Snake head is dark blue (`#0b6ad8`), body is lighter blue (`#4aa3ff`),
  apple is red (`#e24a4a`).
- Arrow keys (and WASD) steer the snake via an `EventControllerKey`
  attached to the canvas.
- 120ms tick interval via `glib::timeout_add_local` advances the snake
  one cell per tick.
- Score (number of apples eaten) shown in the score bar above the canvas.
- Reset button (and Spacebar after game over) restarts the game.
- Game over on wall collision or self-collision — translucent dark
  overlay with "Game Over" text drawn directly on the cairo context.
- 180° turns are blocked (you cannot reverse directly into yourself).
- RNG: in-tree xorshift64 seeded from `SystemTime` (no `rand` dep).

## File layout

```
astra-snake/
├── Cargo.toml
├── src/main.rs
├── assets/css/snake.css
├── astra-snake.desktop
└── README.md
```

## Build

```sh
cargo build --release
```

## Run

```sh
astra-snake
```

## Controls

| Key | Action |
| --- | --- |
| ↑ / W | Steer up |
| ↓ / S | Steer down |
| ← / A | Steer left |
| → / D | Steer right |
| Space | Restart after game over |
