#!/bin/bash

CURRENT_VER=$(head Cargo.toml | grep version | cut -f2 -d'=' | cut -f2 -d\")

# assuming that libmagic is installed in /opt/homebrew (eg: mac M1, M2, etc; aarch64 installation)
RUSTFLAGS=-L/opt/homebrew/lib cargo b --release --target aarch64-apple-darwin
# assuming that libmagic is installed in /usr/local/Cellar (eg: mac intel x86_64 installation)
RUSTFLAGS=-L/usr/local/Cellar/libmagic/5.45/lib cargo b --release --target x86_64-apple-darwin
# assuming that libmagic is installed in
# /opt/homebrew/Cellar/aarch64-unknown-linux-gnu/13.2.0/toolchain/aarch64-unknown-linux-gnu/sysroot/usr/lib/ (eg: linux aarch64 installation)
cargo b --release --target aarch64-unknown-linux-gnu
# assuming that libmagic is installed in
# /opt/homebrew/Cellar/x86_64-unknown-linux-gnu/13.2.0/toolchain/x86_64-unknown-linux-gnu/sysroot/usr/lib/ (eg: linux x86_64 installation)
cargo b --release --target x86_64-unknown-linux-gnu

# remove existing files
rm -rf dist
# make the folder again
mkdir -p dist

# copy files to the dist folder
# macos
cp target/aarch64-apple-darwin/release/yara_dedupe dist/yaya_dedupe_macos_aarch64_v"$CURRENT_VER"
cp target/x86_64-apple-darwin/release/yara_dedupe dist/yara_dedupe_macos_x86-64_v"$CURRENT_VER"
# linux
cp target/aarch64-unknown-linux-gnu/release/yara_dedupe dist/yara_dedupe_linux_aarch64_v"$CURRENT_VER"
cp target/x86_64-unknown-linux-gnu/release/yara_dedupe dist/yara_dedupe_linux_x86-64_v"$CURRENT_VER"
