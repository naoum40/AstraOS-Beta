# Astra Widgets

Desktop widget host for **AstraOS ISO 4** (v0.4). Floating glassmorphism
panels that live on the desktop background: a clock, a calendar, the
current weather, and a battery gauge.

Built with **Rust + GTK4-rs 0.9** (the AstraOS modern-app stack).

## Widgets

| Name       | Source                       | Refresh | Notes                                                       |
| ---------- | ---------------------------- | ------- | ---------------------------------------------------------- |
| `clock`    | `src/widgets/clock.rs`       | 1 s     | 48 pt Outfit time + long-form date                         |
| `calendar` | `src/widgets/calendar.rs`    | on load | `gtk4::Calendar` with prev/next month nav + today highlight |
| `weather`  | `src/widgets/weather.rs`     | 30 min  | `wttr.in/{city}?format=j1` over `reqwest::blocking`         |
| `battery`  | `src/widgets/battery.rs`     | 30 s    | reads `/sys/class/power_supply/BAT0/{capacity,status}`      |

## Configuration

`~/.config/astra/widgets.toml`:

```toml
enabled_widgets = ["clock", "calendar", "weather", "battery"]

[positions]
clock = [64, 64]
calendar = [360, 64]
weather = [64, 240]
battery = [360, 360]
```

On first launch a default config is seeded automatically. Drag any
widget to a new spot — its position is written back to the file on
drag-end so the layout survives a reboot.

## Theme

- Glassmorphism dark background `rgba(22, 25, 40, 0.7)` + 20 px backdrop
  blur + 1 px hairline border + soft drop shadow.
- Accent color `#818cf8` (AstraOS violet).
- Outfit font for time / temperature / month headings.
- System stylesheet: `/usr/share/astraos/themes/widgets.css`.
- Embedded fallback: `assets/css/widgets.css` (compiled in via
  `include_str!` so the binary is self-contained).

## Build

```bash
cargo build --release
# Run
./target/release/astra-widgets
```

The `.desktop` entry (`astra-widgets.desktop`) is autostart-friendly
(`StartupNotify=false` so the WM doesn't show a busy cursor).

## Layout notes

- All windows are `decorated = false` (no titlebar) — they float on the
  desktop.
- Each window is wired with a `gtk4::GestureDrag` (`widgets::attach_drag`)
  so the user can drag them around; the new (x, y) is persisted on
  drag-end.
- Wayland compositors (Hyprland) may ignore client-side window
  positioning — the save still happens, and an X11 session or a future
  layer-shell port will restore the layout faithfully.

## License

MIT — AstraOS Project.
