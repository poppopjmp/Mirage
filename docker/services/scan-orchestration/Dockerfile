FROM rust:1.70-slim AS builder

# Create app directory
WORKDIR /app

# Copy the entire project for building
COPY . .

# Build the scan orchestration service
WORKDIR /scan-orchestration
COPY --from=0 /app/services/scan-orchestration /scan-orchestration
RUN cargo build --release

FROM debian:bullseye-slim

# Install required dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /scan-orchestration/target/release/mirage-scan-orchestration /usr/local/bin/
COPY --from=builder /app/config /app/config
COPY --from=builder /scan-orchestration/migrations /app/migrations

WORKDIR /app
ENV RUN_ENV=production

# Expose the port
EXPOSE 8003

# Run the binary
CMD ["mirage-scan-orchestration"]
