#!/bin/bash

# Settings
SOURCE_DIR="services"
DOCKER_DIR="docker/services"
GITHUB_REPO="van1sh/Mirage"
REGISTRY="ghcr.io"

# Command-line arguments
BUILD_ALL=false
PUSH=false
TAG="latest"
SERVICE=""

# Process arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      BUILD_ALL=true
      shift
      ;;
    --push)
      PUSH=true
      shift
      ;;
    --tag)
      TAG="$2"
      shift 2
      ;;
    --service)
      SERVICE="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [--all] [--push] [--tag TAG] [--service SERVICE]"
      echo "Build Docker images for Mirage services"
      echo ""
      echo "Options:"
      echo "  --all       Build all services"
      echo "  --push      Push images to registry"
      echo "  --tag TAG   Set image tag (default: latest)"
      echo "  --service   Specify a single service to build"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Determine which services to build
if [ "$BUILD_ALL" = true ]; then
  services=(
    "api-gateway"
    "auth-service"
    "user-management-service"
    "scan-orchestration-service" 
    "module-registry-service"
    "data-collection-service"
    "data-storage-service"
    "correlation-engine-service"
    "visualization-service"
    "reporting-service"
    "notification-service"
    "integration-service"
    "configuration-service"
    "discovery-service"
    "web-ui"
  )
elif [ -n "$SERVICE" ]; then
  services=("$SERVICE")
else
  echo "Please specify --all to build all services or --service NAME to build a specific service"
  exit 1
fi

# Build each service
for service in "${services[@]}"; do
  echo "Building $service..."
  
  # Check if service directory exists
  if [ ! -d "$DOCKER_DIR/$service" ]; then
    echo "Error: Docker directory for service $service not found at $DOCKER_DIR/$service"
    continue
  fi
  
  # Build the image
  docker build \
    -t "$REGISTRY/$GITHUB_REPO/$service:$TAG" \
    -f "$DOCKER_DIR/$service/Dockerfile" \
    .
  
  if [ $? -ne 0 ]; then
    echo "Error building $service"
    continue
  fi
  
  echo "Successfully built $service"
  
  # Push the image if requested
  if [ "$PUSH" = true ]; then
    echo "Pushing $service to registry..."
    docker push "$REGISTRY/$GITHUB_REPO/$service:$TAG"
    if [ $? -ne 0 ]; then
      echo "Error pushing $service"
    else
      echo "Successfully pushed $service"
    fi
  fi
done

echo "Build process completed"
