# Makefile for Lunar Lander (Rust + WebAssembly)
TARGET := wasm32-unknown-unknown
WASM_OUT := target/$(TARGET)/release/lunar_lander.wasm
WEB_DIR := web
PORT ?= 8080

.PHONY: all build test run serve clean check-loc

all: build

## Compile the Rust crate to WebAssembly and copy the binary to web/
build:
	cargo build --release --target $(TARGET)
	cp $(WASM_OUT) $(WEB_DIR)/lunar_lander.wasm

## Run all native Rust unit tests
test:
	cargo test

## Build WASM and start a local HTTP server to play the game
run: build serve

## Serve the web/ directory over HTTP
serve:
	@echo "Serving Lunar Lander at http://localhost:$(PORT)"
	python3 -m http.server $(PORT) --directory $(WEB_DIR)

## Verify every source file is under 200 lines of code
check-loc:
	@wc -l Cargo.toml Makefile src/*.rs web/*.html web/*.css web/*.js | awk '$$1 >= 200 && $$2 != "total" { print "ERROR: " $$2 " has " $$1 " lines"; exit 1 } END { print "All source files are under 200 LOC." }'

## Remove build artifacts
clean:
	cargo clean
	rm -f $(WEB_DIR)/lunar_lander.wasm
