BINARY_NAME := browser-debloat
DIST_DIR := dist

AARCH64_APPLE_DARWIN := aarch64-apple-darwin
X86_64_APPLE_DARWIN := x86_64-apple-darwin
X86_64_PC_WINDOWS_GNU := x86_64-pc-windows-gnu
X86_64_UNKNOWN_LINUX_GNU := x86_64-unknown-linux-gnu
AARCH64_UNKNOWN_LINUX_GNU := aarch64-unknown-linux-gnu

.PHONY: all
all: build-all

.PHONY: setup
setup:
	rustup toolchain install nightly
	rustup target add --toolchain nightly $(AARCH64_APPLE_DARWIN)
	rustup target add --toolchain nightly $(X86_64_APPLE_DARWIN)
	cargo install cross --git https://github.com/cross-rs/cross

$(DIST_DIR):
	mkdir -p $(DIST_DIR)

# Build natively for ARM Mac and x86 Mac
.PHONY: aarch64-apple-darwin
aarch64-apple-darwin: $(DIST_DIR)
	cargo +nightly build --release --target $(AARCH64_APPLE_DARWIN)
	cp target/$(AARCH64_APPLE_DARWIN)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-darwin-arm64

.PHONY: x86_64-apple-darwin
x86_64-apple-darwin: $(DIST_DIR)
	cargo +nightly build --release --target $(X86_64_APPLE_DARWIN)
	cp target/$(X86_64_APPLE_DARWIN)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-darwin-x86_64

.PHONY: x86_64-pc-windows-gnu
x86_64-pc-windows-gnu: $(DIST_DIR)
	cross +nightly build --release --target $(X86_64_PC_WINDOWS_GNU)
	cp target/$(X86_64_PC_WINDOWS_GNU)/release/$(BINARY_NAME).exe $(DIST_DIR)/$(BINARY_NAME)-windows-x86_64.exe

.PHONY: x86_64-unknown-linux-gnu
x86_64-unknown-linux-gnu: $(DIST_DIR)
	cross +nightly build --release --target $(X86_64_UNKNOWN_LINUX_GNU)
	cp target/$(X86_64_UNKNOWN_LINUX_GNU)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-linux-x86_64

.PHONY: aarch64-unknown-linux-gnu
aarch64-unknown-linux-gnu: $(DIST_DIR)
	cross +nightly build --release --target $(AARCH64_UNKNOWN_LINUX_GNU)
	cp target/$(AARCH64_UNKNOWN_LINUX_GNU)/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-linux-arm64

.PHONY: build-all
build-all: aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-gnu x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

.PHONY: clean
clean:
	cargo clean
	rm -rf $(DIST_DIR)

.PHONY: run
run:
	cargo +nightly run --release