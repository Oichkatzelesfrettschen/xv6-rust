#!/bin/bash

# Update package index and install necessary build tools along with
# documentation generators and the `cloc` utility for code statistics
sudo apt-get update
sudo apt-get install -y gcc qemu qemu-system-x86 build-essential curl \
    doxygen graphviz python3-sphinx python3-sphinx-rtd-theme \
    python3-sphinxcontrib.jquery cloc

# Install rustup if it is not already installed
if ! command -v rustup >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source "$HOME/.cargo/env"
fi

# Install nightly Rust toolchain and required components
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src rustfmt

# Install cargo-xbuild used by the Makefile
cargo install cargo-xbuild --locked
