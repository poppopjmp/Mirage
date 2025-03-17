FROM rust:1.70-slim AS builder

# Create app directory
WORKDIR /app

# Copy the entire project for building
COPY . .

# Build the user management service
WORKDIR /user-management
COPY --from=0 /app/services/user-management /user-management
RUN cargo build --release

FROM debian:bullseye-slim

# Install required dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /user-management/target/release/mirage-user-management /usr/local/bin/
COPY --from=builder /app/config /app/config
COPY --from=builder /user-management/migrations /app/migrations

WORKDIR /app
ENV RUN_ENV=production

# Expose the port
EXPOSE 8001

# Run the binary
CMD ["mirage-user-management"]
