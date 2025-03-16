# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the source code into the container
COPY . .

# Build the application
RUN cargo build --release

# Use a minimal base image for the final container
FROM debian:buster-slim

# Set the working directory
WORKDIR /usr/src/app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/auth-service .

# Expose the necessary port
EXPOSE 8080

# Set the entrypoint to run the service
ENTRYPOINT ["./auth-service"]
