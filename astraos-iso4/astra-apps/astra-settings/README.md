# Astra Settings

AstraOS system configuration panel — the Windows-Settings-style control
center for the desktop. Built with **Rust + GTK4-rs 0.9 + libadwaita 0.7**.

Part of AstraOS ISO 3 (v0.3).

## Features

- **Account** — avatar, username ("astra"), "Change password" stub.
- **Personalization** — Dark theme switch, 8-color accent picker
  (`#0078d7`, `#107C10`, `#D83B01`, `#8E8E93`, `#FF2D55`, `#5856D6`,
  `#00B7C3`, `#FFB900`), 3 wallpaper thumbnails (violet / blue / classic),
  taskbar mode combo (Windows / Mac), glassmorphism switch.
- **System** — performance mode combo (Éco / Équilibré / Performance),
  battery saver switch, storage readout, Astra Defender status.
- **About** — AstraOS brand symbol, version, edition, license, repo link.

## Layout

```
┌─────────────────────────────────────────────┐
│ ┌──────────┐ ┌─────────────────────────────┐ │
│ │ ✦ Astra  │ │  <content stack>            │ │
│ │          │ │  account / personalization  │ │
│ │ 👤 Acct  │ │  system / about            │ │
│ │ 🎨 Perso  │ │                            │ │
│ │ ⚙️ System │ │                            │ │
│ │ ℹ️ About  │ │                            │ │
│ └──────────┘ └─────────────────────────────┘ │
└─────────────────────────────────────────────┘
   240px                 rest of 900x600
```

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/astra-settings
# — or, once installed on an AstraOS system:
astra-settings
```

## Configuration

Reads `~/.config/astra/desktop.toml` on startup; falls back to the
AstraOS default (dark, `#818cf8` accent, violet wallpaper, Windows-mode
taskbar, glassmorphism on, French locale, auto-lock on, balanced
performance) if the file is missing or unparseable.

Persisting changes from the UI back to `desktop.toml` is wired in
`src/config.rs::SettingsConfig::save()` — UI controls call it via a
follow-up task.

## Theme

CSS lives at `assets/css/settings.css`. At runtime, the app reads
`/usr/share/astraos/themes/settings.css` first (production ISO install);
falls back to an embedded stylesheet kept in `src/main.rs::FALLBACK_CSS`
when the file is missing (dev mode).

The accent palette matches AstraOS's glassmorphism dark brand:
background `#0b0d17`, accent `#818cf8`, secondary `#8b5cf6`,
highlight `#E040FB`.

## License

MIT — © AstraOS Project (mmtstudio).
