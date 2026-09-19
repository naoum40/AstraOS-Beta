# Astra VM

AstraOS Virtual Machine Manager — a glassmorphism dark GTK4 + libadwaita
wrapper around a portable QEMU install (no admin required).

## Features

- VM list sidebar with per-OS gradient icons (AstraOS / Windows / Linux / ARM)
- VM detail view: configuration grid + live stats (CPU, RAM, Disk I/O, Uptime)
- One-click VM start / stop / capture / delete
- QEMU portable auto-detection (system install, `~/.local/share/astra-vm/qemu/`,
  or `qemu-system-x86_64` in `PATH`)
- TOML config at `~/.config/astra-vm/config.toml`
- VM images stored in `~/.local/share/astra-vm/vms/`
- ISOs stored in `~/.local/share/astra-vm/isos/`

## Build

```sh
cargo build --release
```

## Run

```sh
cargo run --release
# or, after install:
astra-vm
```

## Layout

```
┌───────────────────────────────────────────────────────────────┐
│ ✦ Astra VM                       [🐧 QEMU detected] [+ New]   │ 56px header
├─────────────────┬─────────────────────────────────────────────┤
│ Mes VMs         │ VM detail header + actions                  │
│  [icon] Name    │ Configuration grid                          │
│  ● En cours     │ Stats section (when running)               │
│  [icon] Name    │                                             │
│  ● Arrêtée      │                                             │
│ Récent          │                                             │
└─────────────────┴─────────────────────────────────────────────┘
   280px sidebar         fills the rest (min 920px)
```

## License

MIT
