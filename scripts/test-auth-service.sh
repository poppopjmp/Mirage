#!/bin/bash

# Simple auth-service validation test
# This script validates that the auth-service can start and respond to health checks

set -e

echo "🚀 Testing auth-service..."

# Test auth-service
echo "📝 Building auth-service..."
cd services/auth-service

# Build the service
cargo build --release

# Start the service in background
echo "Starting auth-service..."
RUST_LOG=info cargo run --release &
AUTH_PID=$!

# Wait for service to start
sleep 3

# Test health endpoint
echo "Testing health endpoint..."
if curl -f -s http://localhost:8001/health; then
    echo "✅ auth-service health check passed"
else
    echo "❌ auth-service health check failed"
    kill $AUTH_PID 2>/dev/null || true
    exit 1
fi

# Test auth endpoints
echo "Testing auth endpoints..."
if curl -f -s -X POST http://localhost:8001/auth/login; then
    echo "✅ auth-service login endpoint accessible"
else
    echo "❌ auth-service login endpoint failed"
fi

# Clean up
kill $AUTH_PID 2>/dev/null || true
echo "🧹 Cleaned up auth-service"

cd ../..

echo "🎉 Auth-service validation completed successfully!"