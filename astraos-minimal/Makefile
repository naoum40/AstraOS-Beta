# AstraOS Minimal Test ISO — Makefile
SHELL := /usr/bin/env bash
.DEFAULT_GOAL := build-iso

PACK     ?= core
IMAGE    := astraos-builder
PWD      := $(shell pwd)
ISO_NAME ?= astraos-minimal
OUT_DIR  := out
WORK_DIR := work

.PHONY: help build-iso clean

help:
   @echo "Targets:"
   @echo "  build-iso  - Build the minimal test ISO (default)"
   @echo "  clean      - Remove work/ and out/"

build-iso:
   @printf "AstraOS Minimal - building test ISO\n"
   @rm -rf $(WORK_DIR) $(OUT_DIR)
   docker build \
       --build-arg PACK=$(PACK) \
       -f ../astraos-iso4/docker/Dockerfile.build \
       -t $(IMAGE):latest ../astraos-iso4
   @mkdir -p $(OUT_DIR)
   docker run --rm \
       --privileged \
       -e PACK=$(PACK) \
       -v $(PWD)/$(OUT_DIR):/out \
       -v $(PWD):/astraos-build \
       $(IMAGE):latest \
       sh -c "exec mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64"
   @printf "ISO ready -> $(OUT_DIR)/$(ISO_NAME).iso\n"

clean:
   @printf "Cleaning...\n"
   rm -rf $(WORK_DIR) $(OUT_DIR)
