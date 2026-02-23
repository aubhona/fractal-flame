# Build stage
FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Trunk
RUN cargo install trunk

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build frontend (API_BASE="" for same-origin when behind ingress)
ENV API_BASE=""
RUN cd crates/fractal-flame-frontend && trunk build --release

# Runtime stage
FROM nginx:alpine

COPY --from=builder /app/crates/fractal-flame-frontend/dist /usr/share/nginx/html

# SPA fallback: serve index.html for unknown routes
# API routing (/api/*) — через Ingress в K8s
RUN echo 'server { \
    listen 80; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
