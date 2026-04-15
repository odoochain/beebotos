#!/bin/bash
set -e

echo "Running BeeBotOS Unit Tests..."

# Test configuration
TEST_THREADS=${TEST_THREADS:-$(nproc)}
RUST_BACKTRACE=${RUST_BACKTRACE:-1}

echo "Test threads: $TEST_THREADS"

# Run all unit tests
echo "Running Rust unit tests..."
cargo test --workspace \
    --lib \
    --bins \
    --tests \
    -- --test-threads=$TEST_THREADS

# Run with nextest if available
if command -v cargo-nextest &> /dev/null; then
    echo "Running tests with nextest..."
    cargo nextest run --workspace
fi

# Generate test report
echo "Generating test report..."
cargo test --workspace -- --format=json > target/test-report.json 2>/dev/null || true

echo "Unit tests complete!"
