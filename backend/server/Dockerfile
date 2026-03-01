# syntax=docker/dockerfile:1
#
# Multi-stage build using cargo-chef so that dependencies are cached in a
# separate layer. Only application code changes trigger a full recompile;
# Cargo.lock changes are the only thing that invalidates the deps layer.
#
# Build context: ./backend  (the Cargo workspace root)

# ── Stage 1: Install cargo-chef ───────────────────────────────────────────
FROM rust:1.88-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# ── Stage 2: Generate the dependency recipe ───────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: Cook (cache) dependencies ───────────────────────────────────
# This layer is invalidated only when Cargo.lock changes.
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ── Stage 4: Build the server binary ─────────────────────────────────────
# Requires .sqlx/ query metadata committed in the repo (cargo sqlx prepare).
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin server

# ── Stage 5: Minimal runtime image ───────────────────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server /usr/local/bin/server
EXPOSE 3000
ENTRYPOINT ["server"]
