#!/usr/bin/env bash
# Installer for prismfetch — builds a release binary and installs it
# system-wide to /usr/local/bin so it's on every user's $PATH.
set -euo pipefail

if ! command -v cargo &> /dev/null; then
    echo "error: cargo/rust is not installed. Run:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "Building prismfetch (release, optimized)..."
cargo build --release

echo "Installing to /usr/local/bin/prismfetch (requires sudo)..."
sudo install -Dm755 target/release/prismfetch /usr/local/bin/prismfetch

echo "Done. Run it with: prismfetch"
