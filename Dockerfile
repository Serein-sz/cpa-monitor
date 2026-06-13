# syntax=docker/dockerfile:1
FROM rust:1.94-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

RUN cargo build --release -p cpa-ingestor && \
    cp target/release/cpa-ingestor /cpa-ingestor

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /cpa-ingestor /usr/local/bin/cpa-ingestor

ENV CPA__DATABASE__URL=""
ENV CPA__REDIS__URL=""
ENV CPA__REDIS__CHANNEL="usage"

CMD ["cpa-ingestor"]
