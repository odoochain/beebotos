#!/bin/bash
# Setup Monad devnet for local development

set -e

echo "Setting up Monad devnet..."

# Check dependencies
command -v docker >/dev/null 2>&1 || { echo "Docker required"; exit 1; }

# Pull Monad devnet image
docker pull monadfoundation/devnet:latest

# Create network
docker network create monad-devnet 2>/dev/null || true

# Run devnet container
docker run -d \
  --name monad-devnet \
  --network monad-devnet \
  -p 8545:8545 \
  -p 8546:8546 \
  -v monad-data:/data \
  monadfoundation/devnet:latest

# Wait for devnet
echo "Waiting for devnet to start..."
sleep 10

# Fund test accounts
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_sendTransaction","params":[{"from":"0x...","to":"0x...","value":"0x56bc75e2d63100000"}],"id":1}'

echo "Monad devnet ready at http://localhost:8545"
