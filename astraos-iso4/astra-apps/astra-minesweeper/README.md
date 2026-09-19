# Astra Minesweeper

Classic 9x9 / 10-mines Minesweeper for **AstraOS** (ISO 4 / v0.4),
built with Rust + GTK4-rs.

## Features

- 9x9 grid with 10 mines, randomly placed on first click (so the first
  click is always safe — its cell + neighbors are excluded from mine
  placement).
- Header bar: mine count (`💣 NNN`), smiley reset button, timer (`⏱ NNN`).
- Left-click reveals a cell. Revealing a 0-cell triggers recursive
  flood-fill across all adjacent 0-cells and their direct neighbors.
- Right-click toggles a 🚩 flag on an unrevealed cell.
- Per-number colors (1..8):
  - 1=blue `#00c6ff`
  - 2=green `#00ff00`
  - 3=red `#ff2d55`
  - 4=yellow `#ffb900`
  - 5=magenta `#ff007f`
  - 6=cyan `#00b7c3`
  - 7=white
  - 8=gray
- Win = every non-mine cell revealed. Lose = mine cell revealed.
- Reset button (😊) starts a fresh game (😎 on win, 😵 on loss).
- Timer started on first click, stopped on game end.
- RNG: tiny in-tree xorshift64 seeded from `SystemTime` (no `rand` dep).

## File layout

```
astra-minesweeper/
├── Cargo.toml
├── src/main.rs
├── assets/css/minesweeper.css
├── astra-minesweeper.desktop
└── README.md
```

## Build

```sh
cargo build --release
```

## Run

```sh
astra-minesweeper
```
