# Step 1: Install cargo-chef
FROM rust:1.67.1-slim-buster as chef
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-chef

# Step 2: Compute a recipe file
FROM chef as planner
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Step 3: Cache project dependencies
FROM chef as cacher
WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl
RUN --mount=target=/var/lib/apt/lists,type=cache \
    --mount=target=/var/cache/apt,type=cache \
    apt-get update && apt-get install -y \
    musl-tools libssl-dev perl cmake gcc make \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json --features vendored-openssl

# Step 4: Build the binary
FROM rust:1.67.1-slim-buster as builder
WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl
RUN --mount=target=/var/lib/apt/lists,type=cache \
    --mount=target=/var/cache/apt,type=cache \
    apt-get update && apt-get install -y musl-tools  \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --target x86_64-unknown-linux-musl --features vendored-openssl

# Step 5: Create the final image with binary and deps
FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/climbsheet .
ENTRYPOINT ["./climbsheet"]
