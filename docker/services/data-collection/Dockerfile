FROM rust:1.70-slim AS builder

# Create app directory
WORKDIR /app

# Copy the entire project for building
COPY . .

# Build the data collection service
WORKDIR /data-collection
COPY --from=0 /app/services/data-collection /data-collection
RUN cargo build --release

FROM debian:bullseye-slim

# Install required dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /data-collection/target/release/mirage-data-collection /usr/local/bin/
COPY --from=builder /app/config /app/config

WORKDIR /app
ENV RUN_ENV=production

# Expose the port
EXPOSE 8004

# Run the binary
CMD ["mirage-data-collection"]
