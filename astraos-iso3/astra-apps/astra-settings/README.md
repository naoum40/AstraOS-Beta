# Astra Settings

System configuration panel for **AstraOS** (ISO 3 — v0.3).

Astra Settings is the central place where the user tweaks account,
personalization, system and about info. It is a native GTK4 + libadwaita
application written in Rust, sharing the same glassmorphism dark theme
as the rest of the AstraOS desktop.

## Features

- **Account** — avatar, username, change-password button.
- **Personalization** — theme toggle, 8-color accent picker, wallpaper
  gallery, taskbar mode (Windows / Mac), glassmorphism toggle.
- **System** — performance mode, battery saver, storage readout, Astra
  Defender status.
- **About** — AstraOS branding, version, license, GitHub link.

Every change is persisted to `~/.config/astra/desktop.toml` (TOML),
which is also read by `astra-shell` so the whole desktop reacts to the
user's preferences.

## Build

```sh
cargo build --release
```

The release binary lands in `target/release/astra-settings`.

## Run

```sh
astra-settings
```

Or, during development:

```sh
cargo run --release
```

A `.desktop` entry (`astra-settings.desktop`) is shipped so the app
appears in the AstraOS launcher.

## Dependencies (system)

- **GTK4** (`gtk4` ≥ 4.10)
- **libadwaita** ≥ 1.4
- Rust toolchain (edition 2021, MSRV 1.74+)

## Theme

The stylesheet lives at `/usr/share/astraos/themes/settings.css` on an
AstraOS install. When that file is missing (e.g. during `cargo run` on
a dev workstation), the app falls back to an embedded stylesheet
compiled in via `include_str!("../assets/css/settings.css")`.

## Configuration file

`~/.config/astra/desktop.toml`:

```toml
theme = "dark"
accent_color = "#818cf8"
wallpaper = "default-violet.png"
taskbar_mode = "win"
glassmorphism = true
language = "fr"
auto_lock = true
performance_mode = "balanced"
```

## License

MIT — © AstraOS Project.
