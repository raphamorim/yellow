.PHONY: all clean bindings lib zig-build-examples help test

# Default target
all: bindings zig-example

# Build the Rust library (cdylib for C bindings)
lib:
	@echo "Building Rust library..."
	cargo build --release

# Generate C bindings header
bindings: lib
	@echo "C header generated at: bindings/h"

# Build Zig examples
zig-build-examples: bindings
	@echo "Building Zig examples..."
	@if ! command -v zig &> /dev/null; then \
		echo "Error: zig not found. Please install Zig from https://ziglang.org/"; \
		exit 1; \
	fi
	cd bindings/zig && zig build -Doptimize=ReleaseSafe

# Run Zig basic example
run-zig-example-basic: zig-build-examples
	@echo "Running Zig basic example..."
	@echo "Note: This requires a terminal (TTY). Run directly if make fails:"
	@echo "  On macOS:   DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/basic"
	@echo "  On Linux:   LD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/basic"
	@echo ""
	@if [ "$(shell uname)" = "Darwin" ]; then \
		DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/basic; \
	else \
		LD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/basic; \
	fi

# Run Zig colors-rgb example
run-zig-example-colors: zig-build-examples
	@echo "Running Zig colors-rgb example..."
	@echo "Note: This requires a terminal (TTY). Run directly if make fails:"
	@echo "  On macOS:   DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/colors-rgb"
	@echo "  On Linux:   LD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/colors-rgb"
	@echo ""
	@if [ "$(shell uname)" = "Darwin" ]; then \
		DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/colors-rgb; \
	else \
		LD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/colors-rgb; \
	fi

run-rust-example-colors:
	@cd examples/apps/colors-rgb && cargo run --release

# Run Rust tests
test:
	@echo "Running Rust tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf bindings/zig/zig-out
	rm -rf bindings/zig/.zig-cache
