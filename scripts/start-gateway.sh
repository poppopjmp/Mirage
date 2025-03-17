#!/bin/bash
# Start the API Gateway for Mirage

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 Starting Mirage API Gateway"
echo "==========================="

# Create docker network if it doesn't exist
echo "Ensuring docker network exists..."
docker network inspect mirage-network >/dev/null 2>&1 || docker network create mirage-network

# Create log directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/logs/nginx"

# Start the API Gateway using docker-compose
echo "Starting API Gateway container..."
cd "$PROJECT_ROOT"
docker-compose -f docker-compose.gateway.yml up -d

# Check if the container started successfully
if [ $? -eq 0 ]; then
    echo "✅ API Gateway started successfully."
    echo "API Gateway is running at http://localhost"
    echo "Management interface is available at port 8081"
else
    echo "❌ Failed to start API Gateway."
fi

echo "You can stop the gateway with: docker-compose -f docker-compose.gateway.yml down"
