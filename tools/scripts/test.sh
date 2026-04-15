#!/bin/bash
set -e

echo "🧪 Running BeeBotOS test suite..."

# Format check
echo "📋 Checking formatting..."
cargo fmt --all -- --check

# Clippy
echo "🔍 Running clippy..."
cargo clippy --workspace -- -D warnings

# Unit tests
echo "🧪 Running unit tests..."
cargo test --workspace --verbose

# Doc tests
echo "📚 Running doc tests..."
cargo test --doc --workspace

# Integration tests (if any)
if [ -d "tests" ]; then
    echo "🔗 Running integration tests..."
    cargo test --test '*'
fi

echo "✅ All tests passed!"
