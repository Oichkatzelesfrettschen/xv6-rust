#!/bin/bash

# Update package index and install necessary build tools
sudo apt-get update
sudo apt-get install -y gcc qemu qemu-system-x86 build-essential curl

# Install rustup if it is not already installed
if ! command -v rustup >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source "$HOME/.cargo/env"
fi

# Install nightly Rust toolchain and required components
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src

# Install cargo-xbuild used by the Makefile
cargo install cargo-xbuild
