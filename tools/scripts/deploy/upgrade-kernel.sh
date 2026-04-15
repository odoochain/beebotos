#!/bin/bash
set -e

echo "Upgrading BeeBotOS Kernel..."

# Configuration
VERSION=${1}
NAMESPACE=${2:-beebotos-production}

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version> [namespace]"
    exit 1
fi

echo "Upgrading to version: $VERSION"
echo "Namespace: $NAMESPACE"

# Pre-upgrade checks
echo "Running pre-upgrade checks..."

# Check current version
CURRENT_VERSION=$(kubectl get configmap beebotos-config -n $NAMESPACE -o jsonpath='{.data.VERSION}')
echo "Current version: $CURRENT_VERSION"

# Check cluster health
echo "Checking cluster health..."
kubectl get nodes
kubectl top nodes

# Backup current state
echo "Creating backup..."
kubectl get configmap beebotos-config -n $NAMESPACE -o yaml > /tmp/config-backup-$(date +%Y%m%d).yaml

# Rolling upgrade
echo "Performing rolling upgrade..."

# Update brain nodes first
echo "Updating brain nodes..."
kubectl set image deployment/brain \
    brain=beebotos/brain:$VERSION \
    -n $NAMESPACE
kubectl rollout status deployment/brain -n $NAMESPACE

# Update agent runtimes
echo "Updating agent runtimes..."
kubectl set image deployment/agent-runtime \
    agent=beebotos/agent:$VERSION \
    -n $NAMESPACE
kubectl rollout status deployment/agent-runtime -n $NAMESPACE

# Update gateway last
echo "Updating gateway..."
kubectl set image deployment/gateway \
    gateway=beebotos/gateway:$VERSION \
    -n $NAMESPACE
kubectl rollout status deployment/gateway -n $NAMESPACE

# Update version in configmap
echo "Updating version config..."
kubectl patch configmap beebotos-config -n $NAMESPACE \
    --type merge \
    -p '{"data":{"VERSION":"'$VERSION'"}}'

# Post-upgrade verification
echo "Running post-upgrade verification..."
sleep 10

# Health checks
echo "Health checks:"
for svc in gateway agent-runtime brain; do
    kubectl get pods -n $NAMESPACE -l app=$svc
done

# Run smoke tests
echo "Running smoke tests..."
curl -f http://$(kubectl get svc gateway -n $NAMESPACE -o jsonpath='{.status.loadBalancer.ingress[0].ip}')/health

echo "Kernel upgrade complete!"
echo "New version: $VERSION"
