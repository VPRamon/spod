# Multi-stage Dockerfile producing a slim image with the spod CLI and REST server.
# Build:  docker build -t spod:dev .
# Run:    docker run --rm -p 8080:8080 spod:dev
FROM rust:1.89-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --release --bins

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/spod /usr/local/bin/spod
COPY --from=builder /workspace/target/release/spod-rest /usr/local/bin/spod-rest
EXPOSE 8080
ENTRYPOINT ["spod-rest"]
