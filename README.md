<div align="center">

# ✦ AstraOS

### Système d'exploitation Linux moderne, performant et indépendant

**Arch Linux-based · Wayland · Hyprland · Rust · GTK4 · Glassmorphism**

</div>

---

<div align="center">

![Status](https://img.shields.io/badge/Status-Work_in_Progress-818cf8?style=for-the-badge)
![Version](https://img.shields.io/badge/Version-0.1.0_(ISO_1)-8b5cf6?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)
![Base](https://img.shields.io/badge/Base-Arch_Linux-1793d1?style=for-the-badge)
![Shell](https://img.shields.io/badge/Shell-Rust_+GTK4-rs-dea584?style=for-the-badge)

</div>

---

## 🌟 À propos d'AstraOS

**AstraOS** est un système d'exploitation Linux basé sur Arch Linux, conçu pour le grand public avec une philosophie claire : **performance, élégance, indépendance**.

Inspiré par la stratégie Apple Silicon (verticalisation de la stack), AstraOS possède sa propre couche d'interface construite en Rust + GTK4-rs, tout en restant compatible avec l'écosystème Arch Linux.

### 🎯 Objectifs

- 🚀 **Performance maximale** — ~250-300 Mo de RAM au boot (vs 4-6 Go pour Windows 11)
- 🎨 **Glassmorphism natif** — Transparence, blur, animations fluides
- 🛡️ **Indépendance totale** — UI AstraOS propriétaire + backends agnostiques
- 🇫🇷 **Grand public francophone** — Locale FR par défaut, multi-langue à venir
- 🔓 **100% open source** — Licence MIT, code transparent

---

## ⚠️ État du projet

> 🚧 **AstraOS est en cours de développement actif.**
>
> La version actuelle est **0.1.0 (ISO 1)** — base système uniquement.
>
> Le code Rust (Astra Shell + apps custom) arrive à partir de l'ISO 2.

### 📍 Composants actuels et à venir

| Composant | Langage | Statut | ISO |
|---|---|---|---|
| Base Arch Linux + Hyprland | C/C++ (existant) | ✅ OK | ISO 1 |
| Boot loaders + branding | Bash + configs | ✅ OK | ISO 1 |
| **Astra Shell** (barre, dock, launcher, lock screen) | **Rust + GTK4-rs** | 🔜 En cours | ISO 2 |
| Welcome screen (typing animation) | Rust + GTK4-rs | 🔜 En cours | ISO 2 |
| Apps système (Terminal, Files, Firefox, VLC) | (préinstallés) | 🔜 | ISO 3 |
| **AstraPass** (gestionnaire mots de passe) | Rust + GTK4-rs | 🔜 | ISO 4 |
| Widgets desktop | Rust + GTK4-rs | 🔜 | ISO 4 |
| Sticky Notes | Rust + GTK4-rs | 🔜 | ISO 4 |
| Jeux (Démineur, Morpion, Snake) | Rust + GTK4-rs | 🔜 | ISO 4 |
| Astra Defender (antivirus) | Rust + GTK4-rs | 🔜 | ISO 5 |
| Task Manager | Rust + GTK4-rs | 🔜 | ISO 5 |
| Astra Assistant (chatbot IA opt-in) | Rust + GTK4-rs | 🔜 | ISO 6 |
| Astra Gaming Launcher (unifié) | Rust + GTK4-rs | 🔜 | ISO 7 |
| Calamares installer (branding) | C++ (existant) | 🔜 | ISO 7 |
| Multi-langue (FR/EN/ES/DE) | i18n | 🔜 | ISO 7 |

---

## 🛠️ Stack technique

### Couches du système

```
┌──────────────────────────────────────────────────────┐
│                  AstraOS                              │
├──────────────────────────────────────────────────────┤
│                                                      │
│  INTERFACE UTILISATEUR (UI)                          │
│  ┌────────────────────────────────────────────────┐  │
│  │ 🦀 Rust + GTK4-rs                              │  │
│  │ → Astra Shell (barre, dock, launcher)         │  │
│  │ → Apps custom (AstraPass, Widgets, etc.)      │  │
│  │ → Glassmorphism via GTK4 CSS                  │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  DAEMONS CRITIQUES (performance + safety)            │
│  ┌────────────────────────────────────────────────┐  │
│  │ 🦀 Rust pur                                   │  │
│  │ → Astra Update (A/B partition + signatures)   │  │
│  │ → Astra WinBox daemon (gaming VM, phase 4+)   │  │
│  │ → Astra Defender backend (ClamAV wrapper)     │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  COMPOSITOR WAYLAND                                  │
│  ┌────────────────────────────────────────────────┐  │
│  │ 🔧 Hyprland (C++, existant)                   │  │
│  │ → Blur natif (glassmorphism global)            │  │
│  │ → Tiling + floating + workspaces              │  │
│  │ → Multi-monitor                               │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  BASE ARCH LINUX (non modifiée)                      │
│  ┌────────────────────────────────────────────────┐  │
│  │ 🔧 C/C++                                      │  │
│  │ → Linux kernel + systemd + pacman             │  │
│  │ → PipeWire (audio) + NetworkManager           │  │
│  │ → glibc + Bash + Wayland                      │  │
│  │ → Mesa drivers (AMD/Intel/NVIDIA)             │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### Pourquoi Rust ?

| Critère | Rust | C pur | TypeScript |
|---|---|---|---|
| Performance native | ✅ 100% | ✅ 100% | ❌ 60-70% |
| Safety mémoire | ✅ Borrow checker | ❌ Segfaults | ✅ GC |
| Pas de runtime | ✅ Binaire natif | ✅ Binaire natif | ❌ V8 ~80 Mo |
| Pas de GC | ✅ Ownership | ✅ Manuel | ❌ GC pauses |
| Glassmorphism | ✅ GTK4-rs | ⚠️ GTK4 C bindings | ✅ AGS v2 |
| Écosystème Linux | ✅ Mature | ✅ Mature | ❌ Limité desktop |
| Comme Windows/macOS | ✅ (Swift/C# equiv) | ❌ | ❌ |

**Comparaison RAM au boot** :
- Windows 11 : 4-6 Go
- Ubuntu (GNOME/JS) : 1.2-1.5 Go
- macOS Sonoma (Swift/ARC) : 2-3 Go
- **AstraOS (Rust)** : **~250-300 Mo** ✅ le plus économe

---

## 📦 Packs ISO disponibles

AstraOS proposera plusieurs ISO packs selon l'usage :

| Pack | Cible | Apps préinstallées |
|---|---|---|
| **Core** ⭐ | Base minimale (power users) | Hyprland + Terminal + Files + Settings + apps custom AstraOS |
| **Developer** | Développeurs | Core + VSCodium + Claude Code + Docker + Git + Rust toolchain |
| **Bureautique** | Grand public pro | Core + Brave + LibreOffice (rien d'autre) |
| **Gaming** | Gamers (phase 3+) | Core + Astra Gaming Launcher + Steam + Proton + RetroArch |

Build command : `make build-iso pack=core` (ou `developer`, `bureautique`, `gaming`)

---

## 🎮 Stratégie Gaming (post-v1.0)

AstraOS devient **l'agrégateur gaming** unifié :

```
┌─────────────────────────────────────────────────────┐
│              Astra Gaming Launcher                   │
│         (UI AstraOS glassmorphism, Rust)             │
├─────────────────────────────────────────────────────┤
│  Niveau 1 : Natif Linux (CS2, Apex, OW2, Dota 2)   │
│  Niveau 2 : Astra Bridge (Wine/Proton)             │
│             Cyberpunk, Elden Ring, tous les Steam   │
│  Niveau 3 : RetroArch (SNES, PS2, GC, Switch)      │
│  Niveau 4 : Waydroid (apps Android)                │
│  Niveau 5 : Astra WinBox (VM gaming, phase 4+)     │
│             Valorant, R6, Fortnite (ban risk)      │
└─────────────────────────────────────────────────────┘
```

**Indépendance philosophy** : AstraOS possède la couche UI, les plateformes (Steam, Epic, Ubisoft) restent des backends invisibles.

---

## 🏗️ Architecture du repository

```
astraos/
├── iso/                          # archiso profiles
│   ├── x86_64/                   # Intel/AMD 64-bit
│   │   ├── profiledef.sh
│   │   ├── packages.x86_64
│   │   ├── pacman.conf
│   │   ├── customize_airootfs.sh
│   │   ├── airootfs/             # System files overlay
│   │   │   ├── etc/              # System configs
│   │   │   │   ├── skel/         # Dotfiles du user (/home/astra/)
│   │   │   │   ├── systemd/      # Services
│   │   │   │   └── greetd/       # Display manager config
│   │   │   └── root/
│   │   ├── syslinux/             # BIOS boot
│   │   ├── boot/                 # UEFI boot
│   │   └── efiboot/              # EFI System Partition mirror
│   └── aarch64/                  # ARM profile (phase 6+, future NVIDIA RTX Spark)
│
├── astra-shell/                  # 🦀 Astra Shell (Rust + GTK4-rs) — ISO 2+
│   ├── Cargo.toml
│   └── src/
│       ├── bar.rs                # Taskbar
│       ├── dock.rs               # Dock (Mac mode)
│       ├── launcher.rs           # App launcher
│       ├── lock_screen.rs        # Lock screen
│       └── widgets/              # Desktop widgets
│
├── astra-apps/                   # 🦀 Custom AstraOS apps
│   ├── astrapass/                # Password manager (AES-GCM 256)
│   ├── widgets/                  # Desktop widgets
│   ├── sticky-notes/             # Notes
│   ├── minesweeper/              # Game
│   ├── tictactoe/                # Game
│   ├── snake/                    # Game
│   ├── astra-defender/           # Antivirus (ClamAV wrapper)
│   ├── task-manager/             # Task manager
│   └── astra-assistant/          # AI chatbot (opt-in, Qwen)
│
├── astra-core/                   # System core configs
│   ├── hyprland.conf             # Hyprland config (glassmorphism)
│   ├── astra-update/             # 🦀 Update daemon (A/B partition + crypto signatures)
│   └── astra-winbox/             # 🦀 Gaming VM daemon (phase 4+)
│
├── assets/                       # Logos, wallpapers, fonts, icons
│   ├── logos/                    # AstraOS official logos
│   ├── wallpapers/               # Default wallpapers
│   └── fonts/                    # Outfit, JetBrains Mono
│
├── docker/                       # Build pipeline
│   ├── Dockerfile.build          # ISO builder container
│   └── Dockerfile.dev            # Dev container (Rust toolchain)
│
├── .devcontainer/                # VS Code DevContainer config
├── scripts/                      # Helper scripts
│   └── copy-assets.sh            # Stage logos + wallpapers into airootfs
│
├── docs/                         # Documentation
│   ├── ARCHITECTURE.md
│   ├── BUILD.md
│   └── ROADMAP.md
│
├── Makefile                      # Build orchestration
├── LICENSE                       # MIT License
└── README.md                     # This file
```

---

## 🚀 Installation & Build

### Prérequis

- [Docker](https://www.docker.com/) installé (sur Windows/Mac/Linux)
- 5 Go d'espace disque libre
- Connexion internet

### Build de l'ISO

```bash
# 1. Cloner le repo
git clone https://github.com/naoum40/astraos-iso-test.git
cd astraos-iso-test

# 2. Build l'ISO Core (par défaut)
make build-iso

# 3. L'ISO est prête
ls -lh out/
# → astraos-0.1.0-x86_64.iso (~1-2 Go)
```

### Autres commandes Makefile

```bash
make build-iso-dev          # Pack Developer
make build-iso-bureautique  # Pack Bureautique
make copy-assets            # Stager les assets (logos + wallpapers)
make clean                  # Nettoyer work/ et out/
make dev                    # Ouvrir le dev container (Rust toolchain)
make test-qemu              # Tester l'ISO dans QEMU
make help                   # Voir toutes les commandes
```

### Test de l'ISO

**Avec VirtualBox** (recommandé) :
1. Téléchargez l'ISO dans `out/`
2. Créez une nouvelle VM VirtualBox (Linux 64-bit, 4 Go RAM, 30 Go disque)
3. Montez l'ISO dans le lecteur CD virtuel
4. Boot → AstraOS démarre en live

**Avec QEMU** :
```bash
make test-qemu
```

---

## 🗺️ Roadmap

### Phase 1 — v1.0 (7 ISOs itératifs)

| ISO | Version | Contenu | Statut |
|---|---|---|---|
| **ISO 1** | 0.1 | Base Arch + Hyprland + branding AstraOS | ✅ **Build OK** |
| ISO 2 | 0.2 | + Astra Shell + welcome typing animation | 🔜 En cours |
| ISO 3 | 0.3 | + Apps système (Terminal, Files, Firefox, VLC) | ⏳ À venir |
| ISO 4 | 0.4 | + AstraPass, Widgets, Sticky Notes, 3 jeux | ⏳ À venir |
| ISO 5 | 0.5 | + Astra Defender, Task Manager, Performance Mode | ⏳ À venir |
| ISO 6 | 0.6 | + Astra Assistant, Screen recorder, Screenshot | ⏳ À venir |
| ISO 7 | 1.0 | + Astra Gaming Launcher + Calamares + i18n | ⏳ À venir |

### Phases post-v1.0

| Phase | Features |
|---|---|
| **Phase 2** | Astra Bridge (Wine/Proton) + Astra Mail + Astra Sync + Astra Store + Astra Photos |
| **Phase 3** | Astra Gaming Launcher (Steam/Epic/Ubisoft unifié) + Astra Phone Sync |
| **Phase 4** | Astra WinBox beta (VM gaming, power users) |
| **Phase 5** | Astra WinBox stable + Sunshine streaming + Astra Mobile |
| **Phase 6** | AstraOS ARM (RTX Spark target) + Box64/FEX-Emu |
| **Phase 7** | Waydroid (apps Android native) |

Voir [`docs/ROADMAP.md`](docs/ROADMAP.md) pour les détails.

---

## 🎨 Identité visuelle

### Palette de couleurs

| Couleur | Hex | Usage |
|---|---|---|
| Indigo primary | `#818cf8` | Accent principal |
| Violet | `#8b5cf6` | Gradient |
| Magenta néon | `#E040FB` | Highlights, glow |
| Dark deep | `#0b0d17` | Background |
| Panel glass | `rgba(22, 25, 40, 0.7)` | Glassmorphism |

### Typography

- **Outfit** — Display (titres, wordmark)
- **Inter** — Body text
- **JetBrains Mono** — Terminal, code

### Assets

- Logo officiel : `assets/logos/logo-astraos.png`
- Logo boot : `assets/logos/logo-boot.png`
- Wallpaper violet (défaut) : `assets/wallpapers/default-violet.png`
- Wallpaper bleu : `assets/wallpapers/default-bleu.png`
- Wallpaper classique : `assets/wallpapers/classic.png`

## ⚠️ Utilisation des logos

Les logos, icônes et éléments graphiques présents dans ce projet sont protégés et restent la propriété de leurs auteurs ou détenteurs respectifs.

**Toute utilisation des logos à des fins commerciales ou lucratives est interdite sans autorisation préalable.**

Il est également interdit de :

* vendre ou revendre les logos ;
* intégrer les logos dans un produit ou service commercial sans autorisation ;
* modifier les logos dans le but de les exploiter commercialement ;
* utiliser les logos pour donner l’impression d’une affiliation ou d’un partenariat officiel.

L’utilisation à titre personnel, non commercial et à des fins de présentation du projet est autorisée

Pour toute utilisation commerciale, veuillez demander une autorisation préalable au propriétaire des logos.

---

## 🤝 Philosophie du projet

### Indépendance (stratégie Apple Silicon)

AstraOS verticalise la stack : UI AstraOS propriétaire + backends agnostiques.

```
Astra Mail     = UI AstraOS + IMAP/Gmail/Outlook (backends existants)
Astra Sync     = UI AstraOS + Drive/Dropbox/Nextcloud (backends existants)
Astra Gaming   = UI AstraOS + Steam/Epic/Ubisoft (backends existants)
Astra Bridge   = UI AstraOS + Wine/Proton/DXVK (backends existants)
```

### Open source et transparence

- 100% code source ouvert (MIT)
- Build reproductible via Docker
- Aucune télémétrie native
- Aucune donnée personnelle collectée

---

## 📜 Licence

Ce projet est sous licence **MIT** — voir le fichier [`LICENSE`](LICENSE).

```
MIT License

Copyright (c) 2026 AstraOS Project (Astra Corporation by mmtstudio)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files...
```

---

## 👥 Crédits

**AstraOS Project** — *Astra Corporation by (mmtstudio)*

- **Concept & Direction** : naou_m (@naoum40)
- **Développer** : naou_m et GLM 5.2 (Assistant)
- **Base technique** : Arch Linux
- **Logo & branding** : MMT Studio

---

## 🔗 Liens utiles

- 📦 **Repo GitHub** : [github.com/naoum40/astraos-iso-test](https://github.com/naoum40/astraos-iso-test)
- 🐛 **Signaler un bug** : [Issues GitHub](https://github.com/naoum40/astraos-iso-test/issues)
- 📖 **Documentation** : [`docs/`](docs/)
- 🗺️ **Roadmap** : [`docs/ROADMAP.md`](docs/ROADMAP.md)
- 🏗️ **Architecture** : [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- 🚀 **Build guide** : [`docs/BUILD.md`](docs/BUILD.md)

---

<div align="center">

### ✦ AstraOS *

*Made with 🧡 by Astra Corporation (mmtstudio)*

</div>
