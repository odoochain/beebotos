#!/bin/bash
set -e

echo "Setting up BeeBotOS DAO testnet..."

# Configuration
TESTNET_RPC="https://testnet-rpc.monad.xyz"
CHAIN_ID=10143
CONTRACTS_DIR="contracts/solidity"
DEPLOYMENT_DIR="deployments/testnet"

# Create deployment directory
mkdir -p $DEPLOYMENT_DIR

echo "Step 1: Checking Foundry installation..."
if ! command -v forge &> /dev/null; then
    echo "Foundry not found. Please run setup-monad-devnet.sh first."
    exit 1
fi

echo "Step 2: Installing dependencies..."
cd $CONTRACTS_DIR
forge install

echo "Step 3: Building contracts..."
forge build

echo "Step 4: Deploying to testnet..."

# Deploy core contracts
echo "Deploying AgentRegistry..."
REGISTRY_ADDRESS=$(forge create src/core/AgentRegistry.sol:AgentRegistry \
    --rpc-url $TESTNET_RPC \
    --private-key $PRIVATE_KEY \
    | grep "Deployed to:" | awk '{print $3}')

echo "Deploying BeeToken..."
TOKEN_ADDRESS=$(forge create src/dao/token/BeeToken.sol:BeeToken \
    --rpc-url $TESTNET_RPC \
    --private-key $PRIVATE_KEY \
    | grep "Deployed to:" | awk '{print $3}')

echo "Deploying VeBeeToken..."
VE_TOKEN_ADDRESS=$(forge create src/dao/token/VeBeeToken.sol:VeBeeToken \
    --constructor-args $TOKEN_ADDRESS \
    --rpc-url $TESTNET_RPC \
    --private-key $PRIVATE_KEY \
    | grep "Deployed to:" | awk '{print $3}')

echo "Deploying AgentDAO..."
DAO_ADDRESS=$(forge create src/dao/core/AgentDAO.sol:AgentDAO \
    --constructor-args $VE_TOKEN_ADDRESS \
    --rpc-url $TESTNET_RPC \
    --private-key $PRIVATE_KEY \
    | grep "Deployed to:" | awk '{print $3}')

echo "Deploying TreasuryManager..."
TREASURY_ADDRESS=$(forge create src/dao/treasury/TreasuryManager.sol:TreasuryManager \
    --rpc-url $TESTNET_RPC \
    --private-key $PRIVATE_KEY \
    | grep "Deployed to:" | awk '{print $3}')

# Save deployment info
echo "Step 5: Saving deployment info..."
cat > ../../$DEPLOYMENT_DIR/contracts.json << EOF
{
  "network": "monad-testnet",
  "chainId": $CHAIN_ID,
  "deployedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "contracts": {
    "AgentRegistry": "$REGISTRY_ADDRESS",
    "BeeToken": "$TOKEN_ADDRESS",
    "VeBeeToken": "$VE_TOKEN_ADDRESS",
    "AgentDAO": "$DAO_ADDRESS",
    "TreasuryManager": "$TREASURY_ADDRESS"
  }
}
EOF

echo "Step 6: Verifying deployments..."
forge verify-contract $REGISTRY_ADDRESS src/core/AgentRegistry.sol:AgentRegistry \
    --chain-id $CHAIN_ID \
    --watch

echo ""
echo "=========================================="
echo "DAO Testnet Deployment Complete!"
echo "=========================================="
echo ""
echo "Contract Addresses:"
echo "  AgentRegistry: $REGISTRY_ADDRESS"
echo "  BeeToken: $TOKEN_ADDRESS"
echo "  VeBeeToken: $VE_TOKEN_ADDRESS"
echo "  AgentDAO: $DAO_ADDRESS"
echo "  TreasuryManager: $TREASURY_ADDRESS"
echo ""
echo "Deployment info saved to: $DEPLOYMENT_DIR/contracts.json"
