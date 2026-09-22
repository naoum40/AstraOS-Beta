# ╔══════════════════════════════════════════════════════════════════════╗
# ║                     AstraOS — Top-level Makefile                       ║
# ║           Build orchestration for ISO + Dev container                  ║
# ║            © 2026 AstraOS Project — Astra Corporation                  ║
# ╚══════════════════════════════════════════════════════════════════════╝
#
# Usage:
#   make                  # build the ISO (Core pack)
#   make build-iso        # same as above
#   make build-iso-dev    # build the Developer pack
#   make build-iso-bureautique
#   make clean            # wipe work/ and out/
#   make dev              # bash in the dev container
#   make test-qemu        # boot the built ISO in QEMU
#   make copy-assets      # stage assets into airootfs (for local testing)
#   make help

SHELL := /usr/bin/env bash
.DEFAULT_GOAL := build-iso

# ── Variables ───────────────────────────────────────────────────────────
PACK     ?= core
IMAGE    := astraos-builder
DEV_IMG  := astraos-dev
PWD      := $(shell pwd)
ISO_NAME ?= astraos-$(PACK)
OUT_DIR  := out
WORK_DIR := work

# QEMU defaults — tuned for VirtualBox-ish test experience.
QEMU_MEM   ?= 4096
QEMU_CPUS  ?= 4
QEMU_DISK  ?= 32G

# Color helpers
BOLD  := \033[1m
MAG   := \033[35m
CYAN  := \033[36m
YEL   := \033[33m
RST   := \033[0m

# ── Phony targets ───────────────────────────────────────────────────────
.PHONY: help build-iso build-iso-dev build-iso-bureautique \
        clean dev devshell test-qemu copy-assets lint-assets \
        docker-shell

##@ Help
help: ## Show this help
	@printf "$(MAG)╔══════════════════════════════════════════════════════════════╗$(RST)\n"
	@printf "$(MAG)║           AstraOS — Makefile targets                         ║$(RST)\n"
	@printf "$(MAG)╚══════════════════════════════════════════════════════════════╝$(RST)\n"
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
	    | awk 'BEGIN{FS=":.*?## "}{printf "  $(CYAN)%-22s$(RST) %s\n", $$1, $$2}'

##@ Build
build-iso: ## Build the ISO with the Core pack (default)
	@printf "$(MAG)✦ AstraOS$(RST) — building $(BOLD)$(PACK)$(RST) pack ISO\n"
	docker build \
	    --build-arg PACK=$(PACK) \
	    -f docker/Dockerfile.build \
	    -t $(IMAGE):latest .
	@mkdir -p $(OUT_DIR)
	docker run --rm \
	    -e PACK=$(PACK) \
	    -v $(PWD)/$(OUT_DIR):/out \
	    $(IMAGE):latest
	@printf "$(MAG)✦$(RST) ISO ready → $(BOLD)$(OUT_DIR)/$(ISO_NAME).iso$(RST)\n"

build-iso-dev: ## Build the ISO with the Developer pack
	@$(MAKE) --no-print-directory build-iso PACK=developer ISO_NAME=astraos-developer

build-iso-bureautique: ## Build the ISO with the Bureautique pack
	@$(MAKE) --no-print-directory build-iso PACK=bureautique ISO_NAME=astraos-bureautique

##@ Local helpers
copy-assets: ## Stage assets into airootfs (host-side, for local tests)
	@printf "$(CYAN)✦$(RST) Staging AstraOS assets into airootfs…\n"
	@bash scripts/copy-assets.sh
	@printf "$(MAG)✦$(RST) Done.\n"

##@ Clean
clean: ## Remove work/ and out/ directories
	@printf "$(YEL)✦$(RST) Cleaning build artifacts…\n"
	rm -rf $(WORK_DIR) $(OUT_DIR)
	@printf "$(MAG)✦$(RST) Done.\n"

##@ Dev environment
dev: devshell ## Open a bash shell inside the dev container (Rust toolchain)

devshell: ## (alias) Open dev shell
	@printf "$(MAG)✦$(RST) Starting AstraOS dev container…\n"
	docker build -f docker/Dockerfile.dev -t $(DEV_IMG):latest .
	docker run -it --rm \
	    -v $(PWD):/workspace \
	    -w /workspace \
	    $(DEV_IMG):latest bash

##@ Test
test-qemu: ## Boot the built ISO in QEMU (requires out/*.iso)
	@if ! ls $(OUT_DIR)/*.iso >/dev/null 2>&1; then \
	    printf "$(YEL)No ISO found in $(OUT_DIR)/. Run 'make build-iso' first.$(RST)\n"; exit 1; \
	fi
	@printf "$(MAG)✦$(RST) Booting ISO in QEMU ($(QEMU_CPUS) vCPUs, $(QEMU_MEM) MiB RAM)\n"
	qemu-system-x86_64 \
	    -m $(QEMU_MEM) \
	    -smp $(QEMU_CPUS) \
	    -cdrom $$(ls -1 $(OUT_DIR)/*.iso | head -n1) \
	    -boot d \
	    -enable-kvm 2>/dev/null || \
	qemu-system-x86_64 \
	    -m $(QEMU_MEM) \
	    -smp $(QEMU_CPUS) \
	    -cdrom $$(ls -1 $(OUT_DIR)/*.iso | head -n1) \
	    -boot d \
	    -vga virtio \
	    -display gtk

##@ Misc
docker-shell: ## Open a shell inside the builder image (debug)
	docker run -it --rm -v $(PWD):/astraos-build $(IMAGE):latest bash

lint-assets:
	@bash -n scripts/copy-assets.sh && echo "scripts/copy-assets.sh: syntax OK"
