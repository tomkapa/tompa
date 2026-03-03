# syntax=docker/dockerfile:1
#
# Multi-stage build for the container agent.
# The agent embeds a React setup UI (served on :3001 for initial configuration).
# That UI is compiled first, then the Rust binary embeds the resulting assets.
#
# Build context: ./backend  (the Cargo workspace root)

# ── Stage 1: Build the embedded setup UI ─────────────────────────────────
FROM oven/bun:1-slim AS ui-builder
WORKDIR /ui
# Install deps before copying the rest to maximise layer cache hits.
COPY agent/setup-ui/package.json agent/setup-ui/bun.lock* ./
RUN bun install --frozen-lockfile
COPY agent/setup-ui/ .
RUN bun run build

# ── Stage 2: Install cargo-chef ───────────────────────────────────────────
FROM rust:1.93-slim AS chef
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# ── Stage 3: Generate the dependency recipe ───────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 4: Cook (cache) dependencies ───────────────────────────────────
# This layer is invalidated only when Cargo.lock changes.
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# ── Stage 5: Build the agent binary ──────────────────────────────────────
# Embed the pre-built setup UI assets so the binary serves them directly.
# Requires .sqlx/ query metadata committed in the repo (cargo sqlx prepare).
COPY . .
COPY --from=ui-builder /ui/dist agent/setup-ui/dist
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin agent

# ── Stage 6: Minimal runtime image ───────────────────────────────────────
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/agent /usr/local/bin/agent
EXPOSE 3001
ENTRYPOINT ["agent"]
