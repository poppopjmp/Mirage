FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
RUN USER=root cargo new --bin scanner-coordinator
WORKDIR /scanner-coordinator

# Copy manifests
COPY ./services/scanner-coordinator/Cargo.toml ./Cargo.toml
COPY ./common ./common

# Build dependencies - this is the caching Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./services/scanner-coordinator/src ./src
COPY ./config/scanner-coordinator /app/config/scanner-coordinator

# Build for release
RUN rm ./target/release/mirage-scanner-coordinator/mirage_scanner_coordinator*
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /scanner-coordinator/target/release/mirage-scanner-coordinator /usr/local/bin/
COPY --from=builder /app/config/scanner-coordinator /app/config

WORKDIR /app
ENV RUN_ENV=production
ENV RUST_LOG=info

# Expose the port
EXPOSE 8000

# Run the binary
CMD ["mirage-scanner-coordinator"]
