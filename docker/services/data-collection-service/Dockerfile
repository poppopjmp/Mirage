FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
RUN USER=root cargo new --bin data-collection-service
WORKDIR /data-collection-service

# Copy manifests
COPY ./services/data-collection-service/Cargo.toml ./Cargo.toml
COPY ./common ./common

# Build dependencies - this is the caching Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./services/data-collection-service/src ./src


# Build for release
RUN rm ./target/release/mirage-data-collection/mirage_data_collection*
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /data-collection-service/target/release/mirage-data-collection /usr/local/bin/


WORKDIR /app
ENV RUN_ENV=production
ENV RUST_LOG=info

# Expose the port
EXPOSE 8005

# Run the binary
CMD ["mirage-data-collection"]
