FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
RUN USER=root cargo new --bin integration-service
WORKDIR /integration-service

# Copy manifests
COPY ./services/integration-service/Cargo.toml ./Cargo.toml
COPY ./common ./common

# Build dependencies - this is the caching Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./services/integration-service/src ./src
COPY ./config/integration-service /app/config/integration-service

# Build for release
RUN rm ./target/release/deps/mirage_integration_service*
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /integration-service/target/release/mirage-integration-service /usr/local/bin/
COPY --from=builder /app/config/integration-service /app/config

# Create plugins directory
RUN mkdir -p /app/plugins

WORKDIR /app
ENV RUN_ENV=production
ENV RUST_LOG=info

# Expose the port
EXPOSE 8011

# Run the binary
CMD ["mirage-integration-service"]
