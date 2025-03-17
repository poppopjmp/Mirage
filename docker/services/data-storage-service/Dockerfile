FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
RUN USER=root cargo new --bin data-storage-service
WORKDIR /data-storage-service

# Copy manifests
COPY ./services/data-storage-service/Cargo.toml ./Cargo.toml
COPY ./common ./common

# Build dependencies - this is the caching Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./services/data-storage-service/src ./src
COPY ./config/data-storage-service /app/config/data-storage-service

# Build for release
RUN rm ./target/release/deps/mirage_data_storage_service*
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /data-storage-service/target/release/mirage-data-storage-service /usr/local/bin/
COPY --from=builder /app/config /app/config

WORKDIR /app
ENV RUN_ENV=production
ENV RUST_LOG=info

# Expose the port
EXPOSE 8006

# Run the binary
CMD ["mirage-data-storage-service"]
