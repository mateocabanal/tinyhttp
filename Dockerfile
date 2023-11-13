FROM rust:latest as builder

# Make a fake Rust app to keep a cached layer of compiled crates
RUN USER=root cargo new app
WORKDIR /usr/src/app
COPY . .
# Needs at least a main.rs file with a main function

# Will build all dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target 

# Copy the rest
# Build (install) the actual binaries
RUN cargo build --release --features openssl_vendor --example heroku-deploy
RUN mkdir /usr/local/cargo/bin -p ; mv ./target/release/examples/heroku-deploy /usr/local/cargo/bin/heroku-deploy

# Runtime image
FROM debian:bookworm-slim

RUN apt update -y && apt install wget unzip libssl-dev libssl3 -y

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/local/cargo/bin/heroku-deploy /app/heroku-deploy

# No CMD or ENTRYPOINT, see fly.toml with `cmd` override.
