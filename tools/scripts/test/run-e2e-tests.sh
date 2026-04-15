#!/bin/bash
# Run end-to-end tests

set -e

echo "Running BeeBotOS E2E tests..."

# Start local environment
echo "Starting local devnet..."
docker-compose -f docker/docker-compose.yml up -d

# Wait for services
echo "Waiting for services..."
sleep 30

# Run tests
cd tests/e2e

cargo test --test agent_lifecycle -- --nocapture
cargo test --test dao_governance -- --nocapture

# Cleanup
echo "Cleaning up..."
docker-compose -f ../../docker/docker-compose.yml down

echo "E2E tests complete!"
