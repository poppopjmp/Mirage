#!/bin/bash

set -e

# Default namespace
NAMESPACE="mirage"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --namespace)
      NAMESPACE="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [--namespace NAMESPACE]"
      echo "Deploy Mirage services to Kubernetes"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

echo "Deploying Mirage to namespace: $NAMESPACE"

# Create namespace if it doesn't exist
kubectl get namespace "$NAMESPACE" >/dev/null 2>&1 || kubectl create -f kubernetes/mirage-namespace.yaml

# Apply infrastructure components
echo "Deploying infrastructure components..."
kubectl apply -f kubernetes/infrastructure/

# Create secrets
echo "Creating secrets..."
kubectl create secret generic database-secrets \
  --namespace="$NAMESPACE" \
  --from-literal=postgres-user=postgres \
  --from-literal=postgres-password=postgres \
  --from-literal=postgres-auth-uri=postgresql://postgres:postgres@postgres:5432/mirage_auth \
  --from-literal=postgres-users-uri=postgresql://postgres:postgres@postgres:5432/mirage_users \
  --dry-run=client -o yaml | kubectl apply -f -

kubectl create secret generic jwt-secret \
  --namespace="$NAMESPACE" \
  --from-literal=secret=$(openssl rand -hex 32) \
  --dry-run=client -o yaml | kubectl apply -f -

kubectl create secret generic neo4j-credentials \
  --namespace="$NAMESPACE" \
  --from-literal=password=neo4j \
  --dry-run=client -o yaml | kubectl apply -f -

# Wait for infrastructure to be ready
echo "Waiting for infrastructure to be ready..."
kubectl wait --for=condition=ready pod -l app=postgres --namespace="$NAMESPACE" --timeout=120s
kubectl wait --for=condition=ready pod -l app=mongodb --namespace="$NAMESPACE" --timeout=120s
kubectl wait --for=condition=ready pod -l app=neo4j --namespace="$NAMESPACE" --timeout=120s

# Deploy services
echo "Deploying Mirage services..."
kubectl apply -f kubernetes/deployments/

echo "Deployment completed successfully."
echo "You can access the API Gateway at: http://$(kubectl get svc api-gateway -n $NAMESPACE -o jsonpath='{.status.loadBalancer.ingress[0].ip}'):8080"
