ROOT = $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
PACKAGE_NAME = $(shell grep name $(shell pwd)/Cargo.toml | cut -d '"' -f 2)

run:
	@cargo run

build:
	@cargo build

format:
	@cd $(ROOT) && \
	find src -name '*.rs' -exec rustfmt {} \;

lint:
	@cd $(ROOT) && \
	cargo clippy

docs:
	@cargo doc && \
	xdg-open $(ROOT)/target/doc/$(PACKAGE_NAME)/index.html

