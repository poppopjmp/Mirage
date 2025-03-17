FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new empty project
RUN USER=root cargo new --bin auth-service
WORKDIR /auth-service

# Copy manifests
COPY ./services/auth-service/Cargo.toml ./Cargo.toml
COPY ./common ./common
COPY ./config/auth-service /app/config/auth-service
# Build dependencies - this is the caching Docker layer
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./services/auth-service/src ./src
COPY ./config/auth-service /app/config/auth-service

# Build for release
RUN rm ./target/release/deps/mirage_auth_service*
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /auth-service/target/release/mirage-auth-service /usr/local/bin/
COPY --from=builder /app/config/auth-service /app/config

WORKDIR /app
ENV RUN_ENV=production
ENV RUST_LOG=info

# Expose the port
EXPOSE 8001

# Run the binary
CMD ["mirage-auth-service"]
