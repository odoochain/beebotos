#!/bin/bash
set -e

echo "🏗️  Building BeeBotOS..."

# Check Rust installation
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust not found. Please install Rust: https://rustup.rs"
    exit 1
fi

# Install nightly toolchain if needed
if ! rustup toolchain list | grep -q nightly; then
    echo "📥 Installing Rust nightly..."
    rustup install nightly
fi

rustup default nightly

# Add WASM target
rustup target add wasm32-unknown-unknown

echo "🔨 Building workspace..."
cargo build --release --workspace

echo "🧪 Running tests..."
cargo test --workspace

echo "📦 Creating release package..."
mkdir -p dist
cp target/release/gateway dist/
cp target/release/beebot dist/
cp target/release/beehub dist/
cp -r contracts dist/

echo "✅ Build complete! Binaries in ./dist/"
echo ""
echo "Available binaries:"
echo "  - dist/gateway    : API Gateway server"
echo "  - dist/beebot     : CLI tool"
echo "  - dist/beehub     : Agent marketplace"
