#!/bin/bash
set -e

# Configuration
RPC_URL="${MONAD_RPC:-https://rpc.testnet.monad.xyz}"
PRIVATE_KEY="${PRIVATE_KEY:-}"

echo "🚀 Deploying BeeBotOS DAO Contracts..."
echo "RPC URL: $RPC_URL"

if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ PRIVATE_KEY environment variable not set"
    exit 1
fi

cd contracts/solidity

# Install dependencies if needed
if [ ! -d "lib" ]; then
    echo "📥 Installing dependencies..."
    forge install
fi

echo "🔨 Building contracts..."
forge build

echo "🚀 Deploying BeeToken..."
BEE_TOKEN=$(forge create src/token/BeeToken.sol:BeeToken \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --constructor-args "BeeToken" "BEE" \
    | grep "Deployed to" \
    | awk '{print $3}')

echo "✅ BeeToken deployed at: $BEE_TOKEN"

echo "🚀 Deploying TreasuryManager..."
TREASURY=$(forge create src/treasury/TreasuryManager.sol:TreasuryManager \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    | grep "Deployed to" \
    | awk '{print $3}')

echo "✅ TreasuryManager deployed at: $TREASURY"

echo "🚀 Deploying AgentDAO..."
AGENT_DAO=$(forge create src/dao/AgentDAO.sol:AgentDAO \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --constructor-args $BEE_TOKEN $TREASURY \
    | grep "Deployed to" \
    | awk '{print $3}')

echo "✅ AgentDAO deployed at: $AGENT_DAO"

echo ""
echo "📋 Deployment Summary:"
echo "====================="
echo "BeeToken:        $BEE_TOKEN"
echo "TreasuryManager: $TREASURY"
echo "AgentDAO:        $AGENT_DAO"
echo ""
echo "Save these addresses in your .env file!"
