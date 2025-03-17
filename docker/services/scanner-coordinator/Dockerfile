FROM rust:1.75-slim-bookworm AS builder

# Install necessary dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy project files
COPY . .

# Build the application
RUN cargo build --release

# Create a minimal image
FROM debian:bookworm-slim

# Copy the built executable
COPY --from=builder /app/target/release/scanner-coordinator /usr/local/bin/scanner-coordinator

COPY --from=builder /app/target/release/scanner-coordinator /usr/local/bin/mirage-scanner-coordinator
COPY --from=builder /app/config /app/config

# Set environment variables
ENV RUST_LOG=info
ENV RUN_ENV=production

# Expose the port the app runs on
EXPOSE 8080

# Command to run the executable
CMD ["mirage-scanner-coordinator"]
