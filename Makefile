# Yellow Library Bindings Makefile
# Automates building Rust library, generating C headers, and compiling Zig bindings

.PHONY: all clean bindings lib zig-example help test

# Default target
all: bindings zig-example

# Build the Rust library (cdylib for C bindings)
lib:
	@echo "Building Yellow Rust library..."
	cargo build --release

# Generate C bindings header
bindings: lib
	@echo "C header generated at: bindings/yellow.h"

# Build Zig examples
zig-example: bindings zig-basic zig-mosaic

zig-basic: bindings
	@echo "Building Zig basic example..."
	@if ! command -v zig &> /dev/null; then \
		echo "Error: zig not found. Please install Zig from https://ziglang.org/"; \
		exit 1; \
	fi
	cd bindings/zig && zig build -Doptimize=ReleaseSafe

zig-mosaic: zig-basic
	@echo "Zig mosaic example built with basic example"

# Run Zig basic example
run-zig: zig-basic
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

# Run Zig mosaic example
run-zig-mosaic: zig-mosaic
	@echo "Running Zig mosaic example..."
	@echo "Note: This requires a terminal (TTY). Run directly if make fails:"
	@echo "  On macOS:   DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/mosaic"
	@echo "  On Linux:   LD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/mosaic"
	@echo ""
	@if [ "$(shell uname)" = "Darwin" ]; then \
		DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/mosaic; \
	else \
		LD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/mosaic; \
	fi

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

# Display help
help:
	@echo "Yellow Library Bindings - Makefile targets:"
	@echo ""
	@echo "  make                - Build everything (lib + bindings + zig examples)"
	@echo "  make lib            - Build the Rust library"
	@echo "  make bindings       - Generate C header file"
	@echo "  make zig-example    - Build all Zig examples"
	@echo "  make zig-basic      - Build basic Zig example"
	@echo "  make zig-mosaic     - Build mosaic Zig example"
	@echo "  make run-zig        - Build and run basic Zig example"
	@echo "  make run-zig-mosaic - Build and run mosaic Zig example"
	@echo "  make test           - Run Rust tests"
	@echo "  make clean          - Remove all build artifacts"
	@echo "  make help           - Show this help message"
	@echo ""
	@echo "Requirements:"
	@echo "  - Rust (cargo)"
	@echo "  - Zig compiler (for building Zig examples)"
	@echo ""
