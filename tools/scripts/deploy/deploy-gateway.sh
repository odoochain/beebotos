#!/bin/bash
set -e

echo "Deploying BeeBotOS Gateway..."

# Configuration
ENVIRONMENT=${1:-staging}
VERSION=${2:-latest}

# Kubernetes namespace
NAMESPACE="beebotos-$ENVIRONMENT"

echo "Environment: $ENVIRONMENT"
echo "Version: $VERSION"

# Build Docker image
echo "Building Docker image..."
docker build -f docker/Dockerfile.gateway \
    -t beebotos/gateway:$VERSION \
    -t beebotos/gateway:$ENVIRONMENT .

# Push to registry
echo "Pushing to registry..."
docker push beebotos/gateway:$VERSION
docker push beebotos/gateway:$ENVIRONMENT

# Update Kubernetes deployment
echo "Updating Kubernetes deployment..."
kubectl set image deployment/gateway \
    gateway=beebotos/gateway:$VERSION \
    -n $NAMESPACE

# Wait for rollout
echo "Waiting for rollout..."
kubectl rollout status deployment/gateway -n $NAMESPACE

# Verify deployment
echo "Verifying deployment..."
kubectl get pods -n $NAMESPACE -l app=gateway

# Run health check
echo "Running health check..."
GATEWAY_URL=$(kubectl get svc gateway -n $NAMESPACE -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
curl -f http://$GATEWAY_URL/health || echo "Health check failed"

echo "Gateway deployment complete!"
