#!/bin/bash
# Deploy DAO contracts

set -e

RPC_URL=${RPC_URL:-"http://localhost:8545"}
PRIVATE_KEY=${PRIVATE_KEY:-"0x..."}

echo "Deploying DAO contracts to $RPC_URL..."

cd contracts/solidity

# Deploy core
echo "Deploying AgentIdentity..."
IDENTITY=$(forge create AgentIdentity --rpc-url $RPC_URL --private-key $PRIVATE_KEY | grep "Deployed to:" | awk '{print $3}')

echo "Deploying ReputationSystem..."
REPUTATION=$(forge create ReputationSystem --rpc-url $RPC_URL --private-key $PRIVATE_KEY | grep "Deployed to:" | awk '{print $3}')

echo "Deploying BeeToken..."
TOKEN=$(forge create BeeToken --rpc-url $RPC_URL --private-key $PRIVATE_KEY | grep "Deployed to:" | awk '{print $3}')

echo "Deploying AgentDAO..."
DAO=$(forge create AgentDAO --rpc-url $RPC_URL --private-key $PRIVATE_KEY --constructor-args $TOKEN | grep "Deployed to:" | awk '{print $3}')

echo "Deploying TreasuryManager..."
TREASURY=$(forge create TreasuryManager --rpc-url $RPC_URL --private-key $PRIVATE_KEY --constructor-args $DAO $TOKEN | grep "Deployed to:" | awk '{print $3}')

# Save deployment
cat > deployment.json << EOF
{
  "network": "$RPC_URL",
  "contracts": {
    "AgentIdentity": "$IDENTITY",
    "ReputationSystem": "$REPUTATION",
    "BeeToken": "$TOKEN",
    "AgentDAO": "$DAO",
    "TreasuryManager": "$TREASURY"
  },
  "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

echo "Deployment complete! Saved to deployment.json"
