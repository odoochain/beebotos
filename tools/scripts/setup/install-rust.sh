#!/bin/bash
set -e

echo "Installing Rust toolchain for BeeBotOS..."

# Detect OS
OS=$(uname -s)
ARCH=$(uname -m)

echo "Detected: $OS $ARCH"

# Install rustup if not present
if ! command -v rustup &> /dev/null; then
    echo "Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Install required Rust version
RUST_VERSION=$(cat rust-toolchain.toml | grep channel | cut -d'"' -f2)
echo "Installing Rust $RUST_VERSION..."
rustup install $RUST_VERSION
rustup default $RUST_VERSION

# Add required components
echo "Adding required components..."
rustup component add rustfmt clippy rust-analyzer

# Install additional targets
rustup target add wasm32-unknown-unknown
rustup target add wasm32-wasi

# Install cargo tools
echo "Installing cargo tools..."
cargo install cargo-audit --features=fix
cargo install cargo-outdated
cargo install cargo-deny
cargo install cargo-nextest
cargo install just
cargo install wasm-pack

# Verify installation
echo "Verifying installation..."
rustc --version
cargo --version

echo "Rust installation complete!"
