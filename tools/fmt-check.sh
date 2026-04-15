#!/bin/bash
# Format checking script

set -e

echo "🔍 Checking code formatting..."

# Check Rust formatting
echo "Checking Rust files..."
if ! cargo fmt --all -- --check; then
    echo "❌ Rust formatting issues found. Run 'cargo fmt --all' to fix."
    exit 1
fi

# Check Solidity formatting (if forge fmt is available)
if command -v forge &> /dev/null; then
    echo "Checking Solidity files..."
    cd contracts/solidity
    if ! forge fmt --check; then
        echo "❌ Solidity formatting issues found. Run 'forge fmt' to fix."
        exit 1
    fi
    cd ../..
fi

echo "✅ All formatting checks passed!"
