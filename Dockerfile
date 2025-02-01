FROM rust:1.78.0-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Fixes some apt hash mismatch issue
RUN echo "Acquire::http::Pipeline-Depth 0;" > /etc/apt/apt.conf.d/99custom && \
    echo "Acquire::http::No-Cache true;" >> /etc/apt/apt.conf.d/99custom && \
    echo "Acquire::BrokenProxy    true;" >> /etc/apt/apt.conf.d/99custom

RUN apt-get update \
    && apt-get install -y \
    libssl-dev perl cmake gcc make
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --features vendored-openssl
ENTRYPOINT ["./target/release/climbsheet"]

FROM debian:bookworm-slim
WORKDIR /app

# Fixes some apt hash mismatch issue
RUN echo "Acquire::http::Pipeline-Depth 0;" > /etc/apt/apt.conf.d/99custom && \
    echo "Acquire::http::No-Cache true;" >> /etc/apt/apt.conf.d/99custom && \
    echo "Acquire::BrokenProxy    true;" >> /etc/apt/apt.conf.d/99custom

RUN apt-get update \
    && apt-get install -y \
    ca-certificates curl
COPY --from=builder /app/target/release/climbsheet .
CMD ["./climbsheet"]
