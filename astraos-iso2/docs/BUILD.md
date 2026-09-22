# AstraOS — Build Guide

This document describes how to build the AstraOS ISO from scratch, on
Windows with Docker Desktop, on macOS, on Linux, or in GitHub Codespaces.

> ISO 1 (v0.1) is the **Core** pack: base Arch Linux + Hyprland minimal +
> AstraOS branding (wallpaper, logo, bash prompt). Subsequent ISOs add the
> Astra Shell and apps.

---

## 1. Requirements

| Tool | Version | Notes |
| --- | --- | --- |
| Docker Desktop | 4.30+ | Windows / macOS host |
| Git | any | to clone the repo |
| QEMU **or** VirtualBox | any | to boot the resulting ISO |
| Free disk | ~5 GiB | for the build cache + ISO |
| Internet | required | Arch mirrors are pulled at build time |

You do **not** need Rust, archiso, or any other toolchain installed on the
host — everything runs inside the Docker container.

---

## 2. Clone & build

```bash
git clone https://github.com/AstraCorp/astraos.git
cd astraos

# Stage AstraOS assets (wallpapers + logos) into the airootfs.
# The Docker build also runs this step automatically, but you can run it
# locally for debugging:
make copy-assets

# Build the ISO (default pack = core)
make build-iso
```

The resulting ISO appears at:

```
out/astraos-core.iso
```

### Build a different pack

```bash
make build-iso-dev          # astraos-developer.iso
make build-iso-bureautique  # astraos-bureautique.iso
```

> Pack selection switches `packages.x86_64` and `profiledef.sh` (added in a
> later ISO). For ISO 1 only the Core pack is meaningful.

---

## 3. What happens under the hood

The Makefile invokes two Docker stages:

1. **Builder image** (`docker/Dockerfile.build`)
   - `FROM archlinux:latest`
   - Installs `archiso`, `squashfs-tools`, `dosfstools`, `e2fsprogs`,
     `xorriso`, `grub`, `efibootmgr`, `mtools`.
   - Copies the repo into `/astraos-build`.
   - Runs `scripts/copy-assets.sh` to stage AstraOS wallpapers + logos into
     `/astraos-build/iso/x86_64/airootfs/usr/share/{backgrounds,pixmaps}/astraos/`.
   - On `CMD`, runs:
     ```
     mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64
     ```

2. **Dev image** (`docker/Dockerfile.dev`)
   - Used by `make dev` and VS Code DevContainers.
   - Arch Linux + Rust + GTK4 + gtk4-rs + neovim + starship + paru.
   - Default user `astra` with `wheel` NOPASSWD via sudoers.

---

## 4. Test the ISO

### Option A — VirtualBox (recommended on Windows)

1. Open VirtualBox → **New**.
2. **Name:** `AstraOS-test`, **Type:** Linux, **Version:** Arch Linux (64-bit).
3. **RAM:** 4096 MB, **CPU:** 4 cores (Settings → System → Processor).
4. **Display → Graphics Controller:** `VMSVGA`, **Video Memory:** 128 MB.
5. **Storage:** add `out/astraos-core.iso` to the empty optical drive.
6. **Boot** the VM.

### Option B — QEMU

```bash
make test-qemu
```

The Makefile auto-detects KVM if available and falls back to a software
renderer otherwise. You can override RAM / CPU / disk via env vars:

```bash
QEMU_MEM=8192 QEMU_CPUS=8 make test-qemu
```

### Option C — GitHub Codespaces

1. Create a Codespace from this repo (4-core / 16 GiB recommended).
2. `make build-iso`
3. Download `out/astraos-core.iso` from the Codespace file explorer.
4. Boot it on your local machine in VirtualBox or QEMU.

> Codespaces runs in a container, so nested virtualization is **not**
> available — you must download the ISO and boot it on your host.

---

## 5. Common issues

| Symptom | Fix |
| --- | --- |
| `docker: permission denied` | Run Docker Desktop as your user, not root. |
| `mkarchiso: profiledef.sh: not found` | Run `make copy-assets` first. |
| ISO boots to GRUB but kernel panics | Disable Hyper-V on Windows host, or try `VBoxSVGA` instead of `VMSVGA`. |
| `pacman -Syu` fails inside container | Check your DNS / proxy. Set `pacman.conf` mirrors manually. |
| Boot hangs on `Reached target Graphical Interface` | ISO 1 ships no graphical target — switch to a TTY (`Ctrl+Alt+F2`) and run `Hyprland` manually. |

---

## 6. Cleaning up

```bash
make clean        # removes work/ and out/
docker system prune   # reclaim Docker disk
```

---

## 7. Next steps

- ISO 2 will add the Astra Shell (Rust + GTK4-rs) with bar, dock, launcher,
  lock screen, and Welcome screen. See [`ROADMAP.md`](ROADMAP.md).
- Once the DevContainer is wired into VS Code, you can `make dev` and start
  writing Rust + GTK4-rs code without installing anything on the host.
