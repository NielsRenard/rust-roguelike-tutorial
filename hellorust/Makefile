ROOT = $(dir $(realpath $(firstword $(MAKEFILE_LIST))))

run:
	@cargo run

build:
	@cargo build

format:
	@cd $(ROOT) && \
	find src -name '*.rs' -exec rustfmt {} \;
