FROM golang:1.21-alpine AS builder

WORKDIR /app

# Copy dependencies first
COPY services/discovery-service/go.mod services/discovery-service/go.sum ./
RUN go mod download

# Copy the source code
COPY services/discovery-service .

# Build the application
RUN CGO_ENABLED=0 GOOS=linux go build -o discovery-service ./cmd/server

# Use a minimal image for the final container
FROM alpine:3.18

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/discovery-service .
COPY --from=builder /app/config ./config

# Create non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

EXPOSE 8093
CMD ["./discovery-service"]
