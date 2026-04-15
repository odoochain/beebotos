#!/bin/bash
set -e

echo "Running BeeBotOS Fuzz Tests..."

# Rust fuzz tests
echo "Running Rust fuzz tests..."

# Install cargo-fuzz if not present
if ! command -v cargo-fuzz &> /dev/null; then
    cargo install cargo-fuzz
fi

# Run kernel fuzz tests
echo "Running kernel fuzz tests..."
cd crates/kernel
if [ -d "fuzz" ]; then
    cargo fuzz run fuzz_scheduler -- -max_total_time=60
    cargo fuzz run fuzz_memory -- -max_total_time=60
fi
cd ../..

# Run agents fuzz tests
echo "Running agents fuzz tests..."
cd crates/agents
if [ -d "fuzz" ]; then
    cargo fuzz run fuzz_a2a -- -max_total_time=60
fi
cd ../..

# Solidity fuzz tests
echo "Running Solidity fuzz tests..."
cd contracts/solidity

# Run invariant tests
echo "Running invariant tests..."
forge test --match-test invariant -vvv

# Run fuzz tests
echo "Running contract fuzz tests..."
forge test --match-test fuzz -vvv

cd ../..

echo "Fuzz tests complete!"
