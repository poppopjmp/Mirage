FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
RUN USER=root cargo new --bin module-registry-service
WORKDIR /module-registry-service

# Copy manifests
COPY ./services/module-registry-service/Cargo.toml ./Cargo.toml
COPY ./common ./common

# Build dependencies - this is the caching Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./services/module-registry-service/src ./src

# Build for release
RUN rm ./target/release/mirage-module-registry/mirage_module_registry*
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /module-registry-service/target/release/mirage-module-registry /usr/local/bin/

# Create modules directory
RUN mkdir -p /app/modules

WORKDIR /app
ENV RUN_ENV=production
ENV RUST_LOG=info

# Expose the port
EXPOSE 8004

# Run the binary
CMD ["mirage-module-registry"]
