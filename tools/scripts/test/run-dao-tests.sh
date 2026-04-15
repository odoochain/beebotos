#!/bin/bash
set -e

echo "Running BeeBotOS DAO Tests..."

cd contracts/solidity

# Start local Anvil node
echo "Starting local node..."
anvil --fork-url https://testnet-rpc.monad.xyz --silent &
ANVIL_PID=$!
sleep 5

# Set test RPC
export RPC_URL=http://localhost:8545

# Run DAO contract tests
echo "Running DAO contract tests..."
forge test --match-path "test/*DAO*.t.sol" -vvv
forge test --match-path "test/*Voting*.t.sol" -vvv
forge test --match-path "test/*Treasury*.t.sol" -vvv
forge test --match-path "test/*Token*.t.sol" -vvv

# Run integration tests
echo "Running DAO integration tests..."
forge test --match-path "test/integration/*.t.sol" -vvv

# Gas snapshot
echo "Generating gas snapshot..."
forge snapshot

# Coverage
echo "Generating coverage report..."
forge coverage --report lcov

# Cleanup
echo "Cleaning up..."
kill $ANVIL_PID

cd ../..

# Run Rust DAO client tests
echo "Running Rust DAO client tests..."
cargo test -p chain -- dao

echo "DAO tests complete!"
