# syntax=docker/dockerfile:1
# ==============================================================================
# Build Stage: Compile Rust application
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM rust:1-slim-bookworm@sha256:ff521445a372125ed4f76e1453a1f8098f2d05332d1601d30db1c1f62757e730 AS chef

# Install build dependencies.
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    cmake \
    g++ \
    make \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-auditable so the release binary embeds its dependency manifest
# (scan a pulled image later with `cargo audit bin /app/finance-query-server`).
RUN cargo install --locked cargo-auditable cargo-chef@0.1.78

WORKDIR /build

# ------------------------------------------------------------------------------
# Planner: reduce the workspace to a dependency recipe. The recipe masks
# workspace-member versions, so a version bump does not invalidate cooked deps.
# ------------------------------------------------------------------------------
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches
COPY server/Cargo.toml ./server/
COPY server/src ./server/src
COPY finance-query-derive ./finance-query-derive
COPY finance-query-mcp/Cargo.toml ./finance-query-mcp/
COPY finance-query-mcp/src ./finance-query-mcp/src
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# Builder: dependencies compile into an ordinary image layer, which registry
# and GitHub Actions layer caches persist (BuildKit cache mounts are not).
# ------------------------------------------------------------------------------
FROM chef AS builder

# translation-offline for published images; dev overrides to translation
# (dictionary tier only) to skip the from-source CTranslate2 C++ build.
ARG CARGO_FEATURES=translation-offline

# release: opt-level=3, fat LTO, single codegen unit. Dev overrides to `dev`
# (docker-compose.override.yml) for much faster builds.
ARG CARGO_PROFILE=release

COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --locked --profile ${CARGO_PROFILE} -p finance-query-server \
    --features ${CARGO_FEATURES} --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches
COPY server/Cargo.toml ./server/
COPY server/src ./server/src
COPY finance-query-derive ./finance-query-derive
COPY finance-query-mcp/Cargo.toml ./finance-query-mcp/

RUN cargo auditable build --locked --profile ${CARGO_PROFILE} -p finance-query-server --features ${CARGO_FEATURES} && \
    OUT_DIR=$([ "${CARGO_PROFILE}" = "dev" ] && echo debug || echo "${CARGO_PROFILE}") && \
    cp target/${OUT_DIR}/finance-query-server /build/finance-query-server

# ==============================================================================
# Runtime Stage: Minimal runtime image
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171

# libstdc++6/libgomp1 are required by the CTranslate2 translation backend.
RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    ca-certificates \
    libssl3 \
    libstdc++6 \
    libgomp1 \
    curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app

COPY --from=builder /build/finance-query-server /app/finance-query-server

# Pre-create the Hugging Face cache dir with appuser ownership so a named
# volume mounted at HF_HOME inherits it. Small per-language opus-mt models
# (~80-210 MB each) are downloaded under $HF_HOME/argos on first use; mount a
# volume to persist them across deploys.
RUN chown -R appuser:appuser /app \
    && mkdir -p /home/appuser/.cache/huggingface \
    && chown -R appuser:appuser /home/appuser/.cache

USER appuser

# Environment variables (matches server/.env.template)
ENV HF_HOME=/home/appuser/.cache/huggingface
ENV PORT=8000
ENV LOG_LEVEL=INFO
ENV LOG_FORMAT=json
ENV DEFAULT_INTERVAL=1d
ENV DEFAULT_RANGE=1mo
ENV RECOMMENDATIONS_LIMIT=5
ENV SEARCH_HITS=6
ENV SCREENERS_COUNT=25
# ENV REDIS_URL=redis://localhost:6379

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/v2/health || exit 1

CMD ["/app/finance-query-server"]
