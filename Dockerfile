ARG RUST_VERSION=1.86

# --- Build Stage ---
FROM rust:${RUST_VERSION}-slim-bookworm AS build

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libdbus-1-dev \
    build-essential \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release && strip ./target/release/jired

# --- Runtime Stage ---
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libdbus-1-3 \
    bash \
    && rm -rf /var/lib/apt/lists/*

RUN addgroup --system --gid 1001 jired
RUN adduser --system --uid 1001 jired
USER jired

COPY --from=build --chown=jired:jired /app/target/release/jired /usr/local/bin/jired

WORKDIR /home/jired

CMD ["jired"]
