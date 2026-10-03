SHELL := /usr/bin/env bash
.DEFAULT_GOAL := build-iso

IMAGE := astraos-builder:latest
DOCKERFILE := ../../../astraos-iso4/docker/Dockerfile.build
IMAGE_CONTEXT := ../../../astraos-iso4

build-iso:
	@if ! docker image inspect $(IMAGE) >/dev/null 2>&1; then \
		echo "Docker image $(IMAGE) missing; building it..."; \
		docker build -f $(DOCKERFILE) -t $(IMAGE) $(IMAGE_CONTEXT); \
	fi
	docker run --rm --privileged -v $(CURDIR)/out:/out -v $(CURDIR):/astraos-build $(IMAGE) sh -c "mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64"

clean:
	rm -rf work out
