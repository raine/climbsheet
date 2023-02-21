FROM rust:1.67.1-slim-buster AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=target=/var/lib/apt/lists,type=cache \
    --mount=target=/var/cache/apt,type=cache \
    apt-get update && apt-get install -y \
    libssl-dev perl cmake gcc make \
    && rm -rf /var/lib/apt/lists/*
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --features vendored-openssl
ENTRYPOINT ["./target/release/climbsheet"]

FROM debian:buster-slim
WORKDIR /app
RUN --mount=target=/var/lib/apt/lists,type=cache \
    --mount=target=/var/cache/apt,type=cache \
    apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/climbsheet .
ENTRYPOINT ["./climbsheet"]
