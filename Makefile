# Makefile for cross-platform Rust builds

BINARY_NAME := browser-debloat
DIST_DIR := dist

# Target triples
AARCH64_APPLE_DARWIN := aarch64-apple-darwin
X86_64_APPLE_DARWIN := x86_64-apple-darwin
X86_64_PC_WINDOWS_GNU := x86_64-pc-windows-gnu
X86_64_UNKNOWN_LINUX_GNU := x86_64-unknown-linux-gnu
AARCH64_UNKNOWN_LINUX_GNU := aarch64-unknown-linux-gnu

.PHONY: all
all: build-all

.PHONY: setup
setup:
	rustup target add $(AARCH64_APPLE_DARWIN)
	rustup target add $(X86_64_APPLE_DARWIN)
	rustup target add $(X86_64_PC_WINDOWS_GNU)
	rustup target add $(X86_64_UNKNOWN_LINUX_GNU)
	rustup target add $(AARCH64_UNKNOWN_LINUX_GNU)

$(DIST_DIR):
	mkdir -p $(DIST_DIR)

.PHONY: aarch64-apple-darwin
aarch64-apple-darwin: $(DIST_DIR)
	cargo build --release --target $(AARCH64_APPLE_DARWIN)
	cp target/$(AARCH64_APPLE_DARWIN)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-darwin-arm64

.PHONY: x86_64-apple-darwin
x86_64-apple-darwin: $(DIST_DIR)
	cargo build --release --target $(X86_64_APPLE_DARWIN)
	cp target/$(X86_64_APPLE_DARWIN)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-darwin-x86_64

.PHONY: x86_64-pc-windows-gnu
x86_64-pc-windows-gnu: $(DIST_DIR)
	cargo build --release --target $(X86_64_PC_WINDOWS_GNU)
	cp target/$(X86_64_PC_WINDOWS_GNU)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_NAME)-windows-x86_64.exe

.PHONY: x86_64-unknown-linux-gnu
x86_64-unknown-linux-gnu: $(DIST_DIR)
	cargo build --release --target $(X86_64_UNKNOWN_LINUX_GNU)
	cp target/$(X86_64_UNKNOWN_LINUX_GNU)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-linux-x86_64

.PHONY: aarch64-unknown-linux-gnu
aarch64-unknown-linux-gnu: $(DIST_DIR)
	cargo build --release --target $(AARCH64_UNKNOWN_LINUX_GNU)
	cp target/$(AARCH64_UNKNOWN_LINUX_GNU)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-linux-arm64

.PHONY: build-all
build-all: aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-gnu x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

.PHONY: clean
clean:
	cargo clean
	rm -rf $(DIST_DIR)

.PHONY: run
run:
	cargo run --release