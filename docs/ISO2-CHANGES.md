# ISO 2 (v0.2) — Changelog

## Added

- **Astra Shell** — desktop shell in Rust + GTK4-rs
  - Taskbar (Windows mode) with clock, workspaces, start button
  - Dock (Mac mode) with pinned apps
  - App launcher with search
  - Hyprland IPC integration
- **Welcome screen** with typing animation
- **Lock screen** with clock + password (SHA-256 hashed)
- **Onboarding wizard** (6 steps: language, timezone, identity, security, privacy, customization)
- **Glassmorphism theme** CSS
- **greetd** configured to launch Astra Shell on boot
- **Wayland session entry** for display managers

## Build

```bash
make build-iso-2
```

Or use the default:
```bash
make build-iso
```

## Test

Test in VirtualBox with 3D acceleration enabled (required for Hyprland glassmorphism).

## Known limitations

- Astra Shell binary is built during ISO creation (requires Rust toolchain in Docker)
- If Rust build fails, the ISO falls back to plain Hyprland
- Multi-monitor support is experimental
- Astra Shell is in alpha stage (ISO 2)
