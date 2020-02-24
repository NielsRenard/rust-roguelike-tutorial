ROOT = $(dir $(realpath $(firstword $(MAKEFILE_LIST))))

help: ##Show this help.
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'

build: ##Build
	@cargo build

format: ##Format all rust code with rustfmt
	@cd $(ROOT) && \
	find src -name '*.rs' -exec rustfmt {} \;

lint: ##Show lint warnings using clippy
	@cd $(ROOT) && \
	cargo clippy

docs: ##Generate documentation (opens in browser)
	@cargo doc --open

wasm-build:
	@cargo build --target wasm32-unknown-unknown --release

##web: Run the game compiled to webassembly (and serve)
web: wasm-build
	@wasm-bindgen ./target/wasm32-unknown-unknown/release/hellorust.wasm --out-dir web --no-modules --no-typescript && \
	cd ./web && python3 -m http.server 9999

run: ##Run the game natively
	@cargo run

run-debug: ##Run the game natively with map fully revealed
	@MAP_HACK=true cargo run

