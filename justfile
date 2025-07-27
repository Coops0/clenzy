binary_name := "clenzy"
dist_dir := "dist"

aarch64_apple_darwin := "aarch64-apple-darwin"
x86_64_apple_darwin := "x86_64-apple-darwin"
x86_64_pc_windows_gnu := "x86_64-pc-windows-gnu"
x86_64_unknown_linux_gnu := "x86_64-unknown-linux-gnu"
aarch64_unknown_linux_gnu := "aarch64-unknown-linux-gnu"

default: clean build-all

setup:
    rustup toolchain install nightly
    rustup target add --toolchain nightly {{aarch64_apple_darwin}}
    rustup target add --toolchain nightly {{x86_64_apple_darwin}}
    cargo install cross --git https://github.com/cross-rs/cross

_create-dist:
    mkdir -p {{dist_dir}}

macos-arm64: _create-dist
    cargo +nightly build --release --target {{aarch64_apple_darwin}}
    cp -f target/{{aarch64_apple_darwin}}/release/{{binary_name}} {{dist_dir}}/{{binary_name}}-macos-arm64

macos-x86_64: _create-dist
    cargo +nightly build --release --target {{x86_64_apple_darwin}}
    cp -f target/{{x86_64_apple_darwin}}/release/{{binary_name}} {{dist_dir}}/{{binary_name}}-macos-x86_64

windows-x86_64: _create-dist
    cross +nightly build --release --target {{x86_64_pc_windows_gnu}}
    cp -f target/{{x86_64_pc_windows_gnu}}/release/{{binary_name}}.exe {{dist_dir}}/{{binary_name}}-windows-x86_64.exe

linux-x86_64: _create-dist
    cross +nightly build --release --target {{x86_64_unknown_linux_gnu}}
    cp -f target/{{x86_64_unknown_linux_gnu}}/release/{{binary_name}} {{dist_dir}}/{{binary_name}}-linux-x86_64

linux-arm64: _create-dist
    cross +nightly build --release --target {{aarch64_unknown_linux_gnu}}
    cp -f target/{{aarch64_unknown_linux_gnu}}/release/{{binary_name}} {{dist_dir}}/{{binary_name}}-linux-arm64

[parallel]
build-all: macos-arm64 macos-x86_64 windows-x86_64 linux-x86_64 linux-arm64

clean:
    rm -rf {{dist_dir}}