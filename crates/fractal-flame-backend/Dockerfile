# Build stage
FROM rust:1-bookworm AS builder

WORKDIR /app

# Copy workspace and lock file
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release -p fractal-flame-backend

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/fractal-flame-backend /app/fractal-flame-backend
COPY crates/fractal-flame-backend/config.json /app/config.json

ENV APP_PORT=3000

EXPOSE 3000

CMD ["/app/fractal-flame-backend"]
