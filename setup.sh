#!/bin/bash
# -----------------------------------------------------------------------------
# Provision development environment: build tools, virtualization, documentation,
# debugging, and code-analysis utilities, plus Rust nightly toolchain and Cargo
# extensions.
# -----------------------------------------------------------------------------
# Additional dependencies:
# - tmux (headless server)
# - sphinxcontrib-spelling (Sphinx spelling checker)

set -euo pipefail

# If not running as root, prefix commands with sudo
if ((EUID != 0)); then
	SUDO='sudo'
else
	SUDO=''
fi
#
#/**
# * is_installed - check if apt package is installed
# * @param pkg name of the package
# */
is_installed() {
	dpkg -s "$1" >/dev/null 2>&1
}

# 1. Update package index
$SUDO apt-get update -qq

# 2. Install core build & QA tools
$SUDO apt-get install -y \
	build-essential \
	gcc \
	clang \
	clang-format \
	clang-tidy \
	lldb \
	gdb \
	valgrind \
	lcov \
	strace \
	ltrace \
	curl \
	cloc

# 3. Install QEMU & utilities for virtualization/emulation
$SUDO apt-get install -y \
        qemu-system \
        qemu-system-x86 \
        qemu-utils \
        qemu-user-static

# Fall back to language package managers if the qemu command remains missing.
if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
        echo "QEMU not found via OS package manager. Attempting fallback installation..."
        if command -v pip3 >/dev/null 2>&1; then
            echo "Attempting QEMU installation via pip3..."
            $SUDO pip3 install --no-binary :all: qemu || true
        else
            echo "pip3 not found, skipping pip3 QEMU installation."
        fi

        if ! command -v qemu-system-x86_64 >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
            echo "Attempting QEMU installation via npm..."
            npm install -g qemu || true
        elif ! command -v npm >/dev/null 2>&1; then
            echo "npm not found, skipping npm QEMU installation."
        fi

        if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
            echo "QEMU still not found in PATH after fallback installation attempts."
            if [ -f "$HOME/.local/bin/qemu-system-x86_64" ]; then
                echo "QEMU found in $HOME/.local/bin."
                echo "Please add $HOME/.local/bin to your PATH environment variable."
                echo "For example, you can add 'export PATH=\"\$HOME/.local/bin:\$PATH\"' to your .bashrc or .zshrc and restart your shell."
            else
                echo "Could not find QEMU in common user-local binary directories."
                echo "Please install QEMU manually or ensure it's in your PATH."
            fi
        fi
fi

# 4. Install documentation generators & graphviz
$SUDO apt-get install -y \
	doxygen \
	graphviz \
	python3-sphinx \
	python3-sphinx-rtd-theme \
	python3-sphinxcontrib.jquery \
        python3-breathe

if ! python3 -c 'import sphinxcontrib.spelling' 2>/dev/null; then
        $SUDO pip3 install sphinxcontrib-spelling
fi

if ! is_installed tmux; then
	$SUDO apt-get install -y tmux
fi

# 5. Install Rust toolchain via rustup if missing
if ! command -v rustup &>/dev/null; then
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	source "$HOME/.cargo/env"
fi

# 6. Configure Rust nightly toolchain and components
rustup toolchain install nightly
rustup default nightly
rustup component add \
	rust-src \
	rustfmt \
	clippy \
	miri \
	rust-analyzer-preview || true

# 7. Install Cargo extensions for cross-building, fuzzing, auditing, coverage
cargo install cargo-xbuild --locked
cargo install cargo-fuzz --locked
cargo install cargo-audit --locked
cargo install grcov --locked

echo "✅ Development environment provisioning complete."
