FROM golang:1.20-alpine as builder

WORKDIR /app

# Copy Go module manifests
COPY ./services/configuration-service/go.mod ./
COPY ./services/configuration-service/go.sum ./

# Cache dependencies
RUN go mod download

# Copy source code
COPY ./services/configuration-service ./

# Build the binary
RUN CGO_ENABLED=0 go build -o configuration-service .

# Use a small alpine image for the final container
FROM alpine:3.18

# Install certificates for HTTPS
RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/configuration-service /app/
COPY --from=builder /app/config /app/config

# Create config directory
RUN mkdir -p /app/config

ENV RUN_ENV=production
ENV LOG_LEVEL=info

# Expose the service port
EXPOSE 8012

# Run the binary
CMD ["/app/configuration-service"]
