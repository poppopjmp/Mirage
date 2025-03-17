FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
RUN USER=root cargo new --bin api-gateway
WORKDIR /api-gateway

# Copy manifests
COPY ./services/api-gateway/Cargo.toml ./Cargo.toml
COPY ./common ./common

# Build dependencies - this is the caching Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./services/api-gateway/src ./src
COPY ./config/api-gateway /app/config/api-gateway

# Build for release
RUN rm ./target/release/deps/mirage_api_gateway*
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /api-gateway/target/release/mirage-api-gateway /usr/local/bin/
COPY --from=builder /app/config /app/config

WORKDIR /app
ENV RUN_ENV=production

# Expose the port
EXPOSE 8000

# Run the binary
CMD ["mirage-api-gateway"]
