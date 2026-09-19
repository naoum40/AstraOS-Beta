# Astra Tic-Tac-Toe

Classic 3x3 Tic-Tac-Toe for **AstraOS** (ISO 4 / v0.4), built with
Rust + GTK4-rs.

## Features

- 3x3 grid of 80x80px buttons.
- X always starts; turns alternate.
- Win detection covers all 8 winning lines:
  - 3 rows
  - 3 columns
  - 2 diagonals
- Draw detection when every cell is filled with no winner.
- Reset button clears the board but **keeps scores** across rounds.
- Score tracking in the header: X wins, O wins, draws.
- X marks are blue (`#0b6ad8`), O marks are red (`#ff2d55`).
- Winning line is highlighted with the accent color (`#818cf8`).
- Status text under the grid shows whose turn it is or the result.

## File layout

```
astra-tictactoe/
├── Cargo.toml
├── src/main.rs
├── assets/css/tictactoe.css
├── astra-tictactoe.desktop
└── README.md
```

## Build

```sh
cargo build --release
```

## Run

```sh
astra-tictactoe
```
