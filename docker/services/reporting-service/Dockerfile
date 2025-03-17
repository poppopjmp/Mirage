FROM rust:1.70-slim AS builder

# Create app directory
WORKDIR /app

# Copy the entire project for building
COPY . .

# Build the reporting service
WORKDIR /reporting
COPY --from=0 /app/services/reporting /reporting
RUN cargo build --release

FROM debian:bullseye-slim

# Install required dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /reporting/target/release/mirage-reporting /usr/local/bin/
COPY --from=builder /app/config /app/config
COPY --from=builder /reporting/templates /app/templates

WORKDIR /app
ENV RUN_ENV=production

# Create reports directory
RUN mkdir -p /app/reports

# Expose the port
EXPOSE 8008

# Run the binary
CMD ["mirage-reporting"]
