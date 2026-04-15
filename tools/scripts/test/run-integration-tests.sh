#!/bin/bash
set -e

echo "Running BeeBotOS Integration Tests..."

# Start dependencies
echo "Starting test dependencies..."
docker-compose -f tests/docker-compose.test.yml up -d

# Wait for services
echo "Waiting for services..."
sleep 10

# Run database migrations
echo "Running migrations..."
cd migrations
cargo run -- up
cd ..

# Run integration tests
echo "Running integration tests..."
cargo test --test '*' -- --test-threads=1

# Specific integration tests
echo "Running agent integration tests..."
cargo test -p agents --test integration

echo "Running chain integration tests..."
cargo test -p chain --test integration

echo "Running brain integration tests..."
cargo test -p social-brain --test integration

# Cleanup
echo "Cleaning up..."
docker-compose -f tests/docker-compose.test.yml down

echo "Integration tests complete!"
