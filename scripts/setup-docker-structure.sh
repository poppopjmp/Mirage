#!/bin/bash

# Create docker directory structure
mkdir -p docker/services

# Create a Dockerfile for each service
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

for service in "${services[@]}"; do
  mkdir -p "docker/services/$service"
  
  # Create Dockerfile for each service
  cat > "docker/services/$service/Dockerfile" <<EOF
FROM rust:1.70-slim AS builder

WORKDIR /usr/src/app
COPY services/$service .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \\
    && apt-get clean \\
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/$service .
COPY --from=builder /usr/src/app/config ./config

EXPOSE 8080
CMD ["./$service"]
EOF
done

echo "Docker directory structure created!"
