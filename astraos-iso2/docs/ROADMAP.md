# AstraOS — Roadmap

This document tracks the two parallel roadmaps for AstraOS:

1. **The 7 ISO milestones** (v0.1 → v1.0) — incremental desktop releases.
2. **The 7 phases** (post-v1.0) — gaming, mobile, ARM, Android compat.

---

## ISO milestones (v1.0 release)

Each ISO is testable end-to-end in VirtualBox / QEMU. We **do not** start the
next ISO until the previous one is validated.

| ISO | Version | Content | Test focus |
| --- | --- | --- | --- |
| **ISO 1** | 0.1 | Base Arch + Hyprland + AstraOS boot logo + violet wallpaper + branded bash prompt | Boots in VirtualBox? |
| **ISO 2** | 0.2 | + Astra Shell (bar/dock dual Win/Mac, launcher, lock screen, mini onboarding) + Welcome screen with typing animation | UI stable? WM works? |
| **ISO 3** | 0.3 | + System apps: Kitty, Thunar, Firefox, VLC, AstraOS Settings | Apps work? |
| **ISO 4** | 0.4 | + AstraPass, Sticky Notes, Widgets, 3 games (Minesweeper, Tic-Tac-Toe, Snake) | Custom apps run? |
| **ISO 5** | 0.5 | + Astra Defender, Task Manager, Performance Mode, Game Mode, System Monitor | System functional? |
| **ISO 6** | 0.6 | + Astra Assistant (Qwen iframe, opt-in), Voice control, Smart search, Screen recorder / Screenshot / Audio / Webcam | Advanced features OK? |
| **ISO 7** | 1.0 | + Astra Gaming Launcher + Calamares installer + Multi-language (FR/EN/ES/DE) + Multi-user + polish | Production-ready? |

> Current status: **ISO 1 (v0.1) — in progress**.

### ISO packs (final v1.0)

A single codebase produces multiple packs by swapping `packages.x86_64`:

| Pack | Audience | Apps |
| --- | --- | --- |
| **AstraOS Core** | Base / power users | Hyprland + Terminal + Files + Settings + AstraOS apps |
| **AstraOS Developer** | Developers | Core + VSCodium + Claude Code + Docker + Git + Rust toolchain + Kitty |
| **AstraOS Bureautique** | General public / office | Core + Brave + LibreOffice (nothing else) |
| **AstraOS Gaming** (phase 3+) | Gamers | Core + Astra Gaming Launcher + Steam + Proton + RetroArch |

Build commands:

```bash
make build-iso                # core (default)
make build-iso-dev            # developer pack
make build-iso-bureautique    # bureautique pack
```

ISOs 1-7 are built with the **Core** pack. The other packs are generated in
parallel starting from ISO 7.

---

## Post-v1.0 phases

| Phase | Features |
| --- | --- |
| **Phase 1** (v1.0) | Base desktop ISO 1-7, no gaming focus |
| **Phase 2** | Astra Bridge (Wine/Proton) + RetroArch + Astra Mail + Astra Sync + Astra Store + Astra Photos |
| **Phase 3** | Astra Gaming Launcher (Steam/Epic/Ubisoft unified) + Astra Phone Sync |
| **Phase 4** | Astra WinBox beta (gaming VM for kernel anti-cheat games, power users only) |
| **Phase 5** | Astra WinBox stable + Sunshine/Moonlight local streaming + Astra Mobile (smartphone variant) |
| **Phase 6** | Astra OS ARM (RTX Spark target) + x86→ARM emulation (Box64/FEX-Emu) |
| **Phase 7** | Waydroid (Android apps native on AstraOS) |

---

## Development priority order (user-confirmed)

In strict order — Astra Bridge and Astra WinBox come **last**:

1. ISO 1-7 (v1.0) — full desktop
2. Phase 2 — Astra Mail + Astra Sync + Astra Store + Astra Photos + RetroArch
3. Phase 3 — Astra Gaming Launcher + Astra Phone Sync
4. Phase 4 — **Astra Bridge** (Wine/Proton) — start of Windows gaming
5. Phase 5 — Astra Mobile + AstraOS ARM + Box64/FEX-Emu
6. Phase 6 — Waydroid
7. Phase 7 (last) — **Astra WinBox** (gaming VM for Valorant/R6/Fortnite) — the most complex, kept for the end

---

## Gaming strategy — "Reliable + Free"

After exhaustive reverse engineering (18 techniques analyzed), the final
verdict is:

> The ONLY 100% reliable + 100% free solution is: do **not** try to run
> kernel anti-cheat games on AstraOS natively.

The Astra Gaming Launcher auto-routes to the best path:

| Level | Game type | Solution | Reliability | Cost |
| --- | --- | --- | --- | --- |
| 1 | Native Linux game (CS2, Apex, OW2, Dota) | Direct launch | ✅ 100% | Free |
| 2 | Windows game without kernel anti-cheat (Cyberpunk, Elden Ring, indie) | Astra Bridge (Wine/Proton) | ✅ 95% | Free |
| 3 | Retro console game (SNES, PS2, GC, Switch) | RetroArch emulator | ✅ 100% | Free |
| 4 | Android mobile game | Waydroid container | ✅ 90% | Free |
| 5 | Windows game WITH kernel anti-cheat (Valorant, R6, Fortnite, PUBG) | ⚠️ Astra WinBox (phase 4+) | ⚠️ 70-80% ban risk | Free but risky |

For Level 5 games the Astra Gaming Launcher shows an explicit risk warning
and requires user opt-in. AstraOS is not responsible for bans.

---

## Explicitly rejected (v1.0)

- Astra Mobile (smartphone variant) → **reinstated** for phase 3+
- Screen reader (Orca) — accessibility too complex for scope
- Magnifier — not supported
- Disk encryption by default — kept for v1.1
- Auto-organize IA via API (cost) — reconsidered with local file access
- **Astra Chatt** — user's personal chat app, set aside for later
- **Tencent products** — official boycott (Riot 100%, Epic 40%, Ubisoft 5%).
  Valorant explicitly not supported. LoL works but is not promoted.
- **Astra Bridge** (Wine/Proton translation layer) — phase 2+
- **Astra WinBox** (gaming VM for kernel anti-cheat games) — phase 2+
  (beta for power users) → phase 4 (just works for general public)

---

## Current status

| Milestone | Status |
| --- | --- |
| Project identity locked | ✅ done |
| Stack locked (Rust + GTK4-rs + Hyprland + Arch) | ✅ done |
| Visual identity locked (violet/magenta glassmorphism) | ✅ done |
| Iterative ISO plan validated (7 ISOs) | ✅ done |
| Assets staged (`astraos/assets/`) | ✅ done |
| **ISO 1 build pipeline** | ✅ in progress |
| ISO 1 user validation in VirtualBox | ⏳ pending user GO |
| ISO 2+ (Astra Shell) | 🅾️ not started |
