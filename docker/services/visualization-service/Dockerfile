FROM rust:1.73-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy Cargo configuration files first for better caching
COPY Cargo.toml Cargo.lock* ./
COPY common/Cargo.toml ./common/
COPY services/visualization-service/Cargo.toml ./services/visualization-service/

# Create dummy source files to build dependencies
RUN mkdir -p common/src services/visualization-service/src \
    && echo "fn main() {}" > common/src/lib.rs \
    && echo "fn main() {}" > services/visualization-service/src/main.rs

# Build dependencies only (this will be cached if dependencies don't change)
RUN cargo build --release

# Now copy the actual source code
COPY common/src ./common/src
COPY services/visualization-service/src ./services/visualization-service/src

# Build the entire project with actual source code
# Touch the main files to ensure they get rebuilt with the real code
RUN touch common/src/lib.rs services/visualization-service/src/main.rs \
    && cargo build --release
    
# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /visualization-service/target/release/mirage-visualization-service /usr/local/bin/

WORKDIR /app
ENV RUN_ENV=production
ENV RUST_LOG=info

# Expose the port
EXPOSE 8008

# Run the binary
CMD ["mirage-visualization-service"]
