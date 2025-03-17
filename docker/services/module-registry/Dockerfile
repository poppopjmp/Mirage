FROM rust:1.70-slim AS builder

# Create app directory
WORKDIR /app

# Copy the entire project for building
COPY . .

# Build the module registry service
WORKDIR /module-registry
COPY --from=0 /app/services/module-registry /module-registry
RUN cargo build --release

FROM debian:bullseye-slim

# Install required dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /module-registry/target/release/mirage-module-registry /usr/local/bin/
COPY --from=builder /app/config /app/config
COPY --from=builder /module-registry/migrations /app/migrations

WORKDIR /app
ENV RUN_ENV=production

# Create storage directory for modules
RUN mkdir -p /app/storage/modules

# Expose the port
EXPOSE 8002

# Run the binary
CMD ["mirage-module-registry"]
