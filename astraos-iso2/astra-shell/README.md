# Astra Shell

Desktop shell for **AstraOS**, written in **Rust + GTK4-rs**.

This is the first real Rust code of AstraOS - ISO 2 (v0.2) ships this
shell on top of the Hyprland Wayland compositor from ISO 1.

## Components

| Module           | File                      | Responsibility                                                  |
|------------------|---------------------------|-----------------------------------------------------------------|
| **bar**          | `src/bar.rs`              | Windows-style taskbar (start button, workspaces 1-9, clock).     |
| **dock**         | `src/dock.rs`             | Floating macOS-style dock (pinned apps, magnification on hover). |
| **launcher**     | `src/launcher.rs`         | Search-filtered app grid, toggled by the Super key.             |
| **hyprland_ipc** | `src/hyprland_ipc.rs`     | Async listener on Hyprland's `.socket2.sock` event socket.       |
| **theme**        | `src/theme.rs`            | Loads `glassmorphism.css` onto the default `gdk::Display`.      |
| **config**       | `src/config.rs`           | Parses `~/.config/astra/desktop.toml` (TOML).                    |
| **screens**      | `src/screens/`            | Welcome / lock screen / onboarding wizard (ISO 2 boot flow).     |
| **main**         | `src/main.rs`             | Boots the GTK4 `Application`, wires everything together.         |

## Modes

The shell supports two runtime modes (commutable via `desktop.toml`):

- **`win`** (default) - Windows-style taskbar at the bottom of the screen.
- **`mac`** - Floating macOS-style dock centered at the bottom.

Only one surface is built at a time - the unused mode is skipped
entirely, so the shell never has both a bar and a dock on screen.

Switch by editing `~/.config/astra/desktop.toml`:

```toml
taskbar_mode = "mac"   # or "win"
```

## Build

```bash
cargo build --release
```

The build expects GTK 4.10+ and libadwaita 1.4+ development headers
(`libgtk-4-dev`, `libadwaita-1-dev` on Debian/Ubuntu; `gtk4`,
`libadwaita` on Arch). See the parent project's `docker/Dockerfile.dev`
for a ready-to-use build container.

## Run

```bash
# Under Hyprland (ISO 1's compositor)
astra-shell
```

Or via the GApplication ID (so it can be activated by another process,
e.g. the launcher D-Bus shortcut):

```bash
gapplication launch org.astraos.Shell
```

## Configuration

`~/.config/astra/desktop.toml`:

```toml
taskbar_mode   = "win"                                                       # "win" or "mac"
accent_color   = "#818cf8"                                                   # hex
glassmorphism  = true                                                        # bool
wallpaper      = "/usr/share/backgrounds/astraos/default-violet.png"        # path
```

If the file is missing or unparseable, the shell falls back to a
hard-coded default (Windows mode, violet accent, glassmorphism on,
default violet wallpaper) so the ISO is always bootable.

## Theme

The shell loads its CSS from `/usr/share/astraos/themes/glassmorphism.css`
at runtime. The source of truth is checked in at
`assets/css/glassmorphism.css` - the AstraOS package pipeline copies it
to the system path on install.

If the system file is missing (e.g. running outside an AstraOS install),
an embedded fallback stylesheet is used so the shell always boots.

Palette:

| Token        | Value                       | Use                          |
|--------------|-----------------------------|------------------------------|
| `astra_bg`   | `#0b0d17`                   | Almost-black violet bg       |
| `astra_panel`| `rgba(22,25,40,0.7)`        | Translucent panel surface    |
| `astra_accent` | `#818cf8`                 | Indigo accent (hover/focus)  |
| `astra_text` | `#f1f5f9`                   | Near-white body text         |

## Dependencies

- **GTK 4.10+** (gtk4-rs 0.9.x)
- **libadwaita 1.4+** (libadwaita-rs 0.7.x) - reserved for future use
- **glib / gio / gdk4** from gtk-rs 0.20
- **Tokio** 1.x (Hyprland IPC listener, hosted on a dedicated thread)
- **zbus** 4.0 (reserved for future D-Bus shortcut integration)
- **chrono** for clock formatting
- **serde + toml** for config parsing
- **sha2 + hex** for lock-screen password hashing

See `Cargo.toml` for the full list.

## What's next (task 2-b / 2-c)

- Replace the static app list with `.desktop` enumeration via
  `gio::AppInfo`.
- Wire Hyprland workspace events into the bar (broadcast channel).
- Wire the Super-key shortcut through D-Bus -> `Launcher::toggle`.
- Swap `ApplicationWindow` for `gtk4-layer-shell` surfaces so the bar
  and dock anchor properly above normal windows (no focus, no input
  region).

## License

MIT - see the parent `LICENSE` file in `astraos/`.
