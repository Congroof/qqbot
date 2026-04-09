FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY crates/ crates/

RUN cargo build --release --bin napcat-bot

# ---- runtime ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/napcat-bot /usr/local/bin/napcat-bot

ENV RUST_LOG=info

ENTRYPOINT ["napcat-bot"]
