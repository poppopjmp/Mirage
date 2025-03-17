FROM rust:1.70-slim AS builder

WORKDIR /usr/src/app
COPY services/visualization-service .
RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/visualization-service .
COPY --from=builder /usr/src/app/config ./config

EXPOSE 8088
CMD ["./visualization-service"]
