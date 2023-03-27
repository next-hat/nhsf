# stage 1 - Setup cargo-chef
FROM rust:1.68.0-alpine3.17 as planner

WORKDIR /app
RUN apk add gcc g++ make
RUN cargo install cargo-chef --locked
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo chef prepare --recipe-path recipe.json

# state 2 - Cook our dependencies
FROM rust:1.68.0-alpine3.17 as cacher

WORKDIR /app
RUN apk add musl-dev openssl-dev
COPY --from=planner /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef
COPY --from=planner /app .
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo chef cook --release --target=x86_64-unknown-linux-musl --recipe-path recipe.json

# stage 3 - Build our project
FROM rust:1.67.0-alpine3.17 as builder

## Build our metrs daemon binary
WORKDIR /app
RUN apk add musl-dev openssl-dev upx
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY --from=cacher /app .
COPY ./src ./src
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target=x86_64-unknown-linux-musl

## Strip and compress the binary
RUN strip /app/target/x86_64-unknown-linux-musl/release/nhsf
RUN upx /app/target/x86_64-unknown-linux-musl/release/nhsf

# stage 4 - Create runtime image
FROM scratch

LABEL org.opencontainers.image.source https://github.com/nxthat/nhsf

## Copy the binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/nhsf /usr/local/bin/nhsf

## Set entrypoint
ENTRYPOINT ["/usr/local/bin/nhsf"]
