# Compile
FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev gcc
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Run
FROM busybox
COPY --from=builder /app/target/release/rustyflare /rustyflare
COPY entrypoint.sh /entrypoint.sh
USER 65534:65534
HEALTHCHECK --interval=60s --timeout=5s --start-period=30s CMD grep -qx 0 /tmp/status
ENTRYPOINT ["sh", "/entrypoint.sh"]

