FROM rust:1.70-slim AS builder

# Create app directory
WORKDIR /app

# Copy the entire project for building
COPY . .

# Build the notification service
WORKDIR /notification
COPY --from=0 /app/services/notification /notification
RUN cargo build --release

FROM debian:bullseye-slim

# Install required dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /notification/target/release/mirage-notification /usr/local/bin/
COPY --from=builder /app/config /app/config
COPY --from=builder /notification/templates /app/templates

WORKDIR /app
ENV RUN_ENV=production

# Expose the port
EXPOSE 8007

# Run the binary
CMD ["mirage-notification"]
