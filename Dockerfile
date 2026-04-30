# ---- chef: 安装 cargo-chef（只装一次，后续复用）----
FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# ---- planner: 生成依赖编译计划 ----
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---- cacher: 编译依赖（Cargo.toml 不变则缓存命中）----
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ---- builder: 编译项目代码 ----
FROM chef AS builder
COPY --from=cacher /app/target target
COPY . .
RUN cargo build --release --bin napcat-bot

# ---- runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/napcat-bot /usr/local/bin/napcat-bot
ENV RUST_LOG=info
ENTRYPOINT ["napcat-bot"]
