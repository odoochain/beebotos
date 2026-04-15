#!/bin/bash
set -e

echo "Deploying BeeBotOS Agent Cluster..."

# Configuration
ENVIRONMENT=${1:-staging}
REPLICAS=${2:-3}
NAMESPACE="beebotos-$ENVIRONMENT"

echo "Environment: $ENVIRONMENT"
echo "Replicas: $REPLICAS"

# Scale agent deployment
echo "Scaling agent deployment..."
kubectl scale deployment agent-runtime \
    --replicas=$REPLICAS \
    -n $NAMESPACE

# Wait for pods to be ready
echo "Waiting for pods to be ready..."
kubectl wait --for=condition=ready pod \
    -l app=agent-runtime \
    -n $NAMESPACE \
    --timeout=300s

# Verify cluster status
echo "Cluster status:"
kubectl get pods -n $NAMESPACE -l app=agent-runtime

# Check HPA
echo "Checking HPA status..."
kubectl get hpa agent-runtime-hpa -n $NAMESPACE

# Run cluster health check
echo "Running cluster health check..."
for pod in $(kubectl get pods -n $NAMESPACE -l app=agent-runtime -o name); do
    echo "Checking $pod..."
    kubectl exec $pod -n $NAMESPACE -- wget -qO- http://localhost:8080/health || echo "$pod unhealthy"
done

echo "Agent cluster deployment complete!"
