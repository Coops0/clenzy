# Makefile for cross-platform Rust builds
# Supports: Mac ARM, Mac Intel, Windows x86, Linux x86, Linux ARM

# Project configuration
BINARY_NAME := browser-debloat
SRC_DIR := src
TARGET_DIR := target
RELEASE_DIR := $(TARGET_DIR)/release
DIST_DIR := dist

# Rust toolchain commands
CARGO := cargo
RUSTUP := rustup

# Target triples
TARGET_MAC_ARM := aarch64-apple-darwin
TARGET_MAC_INTEL := x86_64-apple-darwin
TARGET_WINDOWS := x86_64-pc-windows-gnu
TARGET_LINUX_X86 := x86_64-unknown-linux-gnu
TARGET_LINUX_ARM := aarch64-unknown-linux-gnu

# Output binary names with platform-specific extensions
BINARY_MAC_ARM := $(BINARY_NAME)-mac-arm
BINARY_MAC_INTEL := $(BINARY_NAME)-mac-intel
BINARY_WINDOWS := $(BINARY_NAME)-windows.exe
BINARY_LINUX_X86 := $(BINARY_NAME)-linux-x86
BINARY_LINUX_ARM := $(BINARY_NAME)-linux-arm

# Build flags
CARGO_FLAGS := --release

# Default target
.PHONY: all
all: build-all

# Install required targets
.PHONY: setup
setup:
	$(RUSTUP) target add $(TARGET_MAC_ARM)
	$(RUSTUP) target add $(TARGET_MAC_INTEL)
	$(RUSTUP) target add $(TARGET_WINDOWS)
	$(RUSTUP) target add $(TARGET_LINUX_X86)
	$(RUSTUP) target add $(TARGET_LINUX_ARM)
	# Additional setup for Windows cross-compilation
	# May require: sudo apt-get install mingw-w64 (on Linux)

# Create distribution directory
$(DIST_DIR):
	mkdir -p $(DIST_DIR)

# Build for macOS ARM (Apple Silicon)
.PHONY: mac-arm
mac-arm: $(DIST_DIR)
	$(CARGO) build $(CARGO_FLAGS) --target $(TARGET_MAC_ARM)
	cp $(TARGET_DIR)/$(TARGET_MAC_ARM)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_MAC_ARM)

# Build for macOS Intel
.PHONY: mac-intel
mac-intel: $(DIST_DIR)
	$(CARGO) build $(CARGO_FLAGS) --target $(TARGET_MAC_INTEL)
	cp $(TARGET_DIR)/$(TARGET_MAC_INTEL)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_MAC_INTEL)

# Build for Windows x86
.PHONY: windows
windows: $(DIST_DIR)
	$(CARGO) build $(CARGO_FLAGS) --target $(TARGET_WINDOWS)
	cp $(TARGET_DIR)/$(TARGET_WINDOWS)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_WINDOWS)

# Build for Linux x86
.PHONY: linux-x86
linux-x86: $(DIST_DIR)
	$(CARGO) build $(CARGO_FLAGS) --target $(TARGET_LINUX_X86)
	cp $(TARGET_DIR)/$(TARGET_LINUX_X86)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_LINUX_X86)

# Build for Linux ARM
.PHONY: linux-arm
linux-arm: $(DIST_DIR)
	$(CARGO) build $(CARGO_FLAGS) --target $(TARGET_LINUX_ARM)
	cp $(TARGET_DIR)/$(TARGET_LINUX_ARM)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_LINUX_ARM)

# Build all targets
.PHONY: build-all
build-all: mac-arm mac-intel windows linux-x86 linux-arm

# Clean build files
.PHONY: clean
clean:
	$(CARGO) clean
	rm -rf $(DIST_DIR)

# Run native build for current platform
.PHONY: run
run:
	$(CARGO) run $(CARGO_FLAGS)