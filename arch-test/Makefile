SHELL := /usr/bin/env bash
.DEFAULT_GOAL := build-iso

build-iso:
	docker run --rm --privileged -v $$(PWD)/out:/out -v $$(PWD):/astraos-build astraos-builder:latest sh -c "mkarchiso -v -w /work -o /out /astraos-build/iso/x86_64"

clean:
	rm -rf work out
