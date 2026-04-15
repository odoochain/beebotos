#!/bin/bash
set -e

echo "Deploying BeeBotOS Contracts..."

# Configuration
NETWORK=${1:-testnet}
RPC_URL=${2:-https://testnet-rpc.monad.xyz}
VERIFIER_URL=${3:-https://testnet-explorer.monad.xyz/api}

# Load environment
if [ -f .env ]; then
    export $(cat .env | grep -v '#' | xargs)
fi

if [ -z "$PRIVATE_KEY" ]; then
    echo "Error: PRIVATE_KEY not set"
    exit 1
fi

cd contracts/solidity

echo "Network: $NETWORK"
echo "RPC: $RPC_URL"

# Get gas price
echo "Getting gas price..."
GAS_PRICE=$(cast gas-price --rpc-url $RPC_URL)
echo "Gas price: $GAS_PRICE"

# Deploy contracts using forge script
echo "Running deployment script..."
forge script script/DeployDAO.s.sol \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify \
    --verifier-url $VERIFIER_URL \
    --gas-price $GAS_PRICE \
    -vvvv

# Save deployment addresses
echo "Saving deployment info..."
mkdir -p ../../deployments/$NETWORK

cat > ../../deployments/$NETWORK/contracts.json << EOF
{
  "network": "$NETWORK",
  "deployedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "$(cast wallet address $PRIVATE_KEY)",
  "contracts": {}
}
EOF

echo "Deployment complete!"
echo "Deployment info saved to: deployments/$NETWORK/contracts.json"
