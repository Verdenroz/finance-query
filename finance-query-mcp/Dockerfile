# syntax=docker/dockerfile:1
# ==============================================================================
# Build Stage: Compile fq-mcp MCP server
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM rust:1-slim-bookworm@sha256:e18a79fc84dfcfc3ab5ba72290398a644c135c97eaa881447fddc354ee4701a3 AS builder

# translation-offline for published images; dev overrides to translation
# (dictionary tier only) to skip the from-source CTranslate2 C++ build.
ARG CARGO_FEATURES=translation-offline

# release: opt-level=3, fat LTO, single codegen unit. Dev overrides to `dev`
# (docker-compose.override.yml) for much faster builds.
ARG CARGO_PROFILE=release

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
# (scan a pulled image later with `cargo audit bin /app/fq-mcp`).
RUN cargo install --locked cargo-auditable

WORKDIR /build

# Copy workspace and all member manifests first (for layer caching)
COPY Cargo.toml Cargo.lock* ./
COPY server/Cargo.toml ./server/
COPY finance-query-derive/Cargo.toml ./finance-query-derive/
COPY finance-query-cli/Cargo.toml ./finance-query-cli/
COPY finance-query-mcp/Cargo.toml ./finance-query-mcp/

# Create dummy sources to cache dependencies.
# benches/ stubs are required because [[bench]] targets are declared in Cargo.toml;
# Cargo validates their source paths during workspace resolution even when not building them.
RUN mkdir -p src server/src finance-query-derive/src finance-query-cli/src finance-query-mcp/src benches && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "pub fn dummy() {}" > server/src/lib.rs && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > finance-query-derive/src/lib.rs && \
    echo "fn main() {}" > finance-query-cli/src/main.rs && \
    echo "fn main() {}" > finance-query-mcp/src/main.rs && \
    for b in indicators backtesting ticker tickers finance stream translation risk providers regression serde dataframe feeds sentiment; do \
        echo "fn main() {}" > benches/$b.rs; \
    done

# Cargo registry + target dir are cache mounts (not COPY'd layers), so a
# trivial Cargo.lock churn (e.g. an internal workspace-member version bump)
# can't invalidate previously-compiled deps — including translation-offline's
# from-source CTranslate2 C++ build, by far the most expensive part of this.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/build/target,sharing=locked \
    cargo build --profile ${CARGO_PROFILE} -p finance-query-mcp --features ${CARGO_FEATURES} && \
    rm -rf src server/src finance-query-derive/src finance-query-cli/src finance-query-mcp/src benches

# Copy actual source code
COPY src ./src
COPY server/src ./server/src
COPY finance-query-derive/src ./finance-query-derive/src
COPY finance-query-mcp/src ./finance-query-mcp/src
COPY benches ./benches

# Build the actual application. The compiled binary is copied out of the
# target cache mount before it unmounts — mount contents aren't part of the
# image layer, so a later `COPY --from=builder` can't reach into them.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/build/target,sharing=locked \
    touch src/lib.rs server/src/lib.rs finance-query-mcp/src/main.rs && \
    cargo auditable build --profile ${CARGO_PROFILE} -p finance-query-mcp --features ${CARGO_FEATURES} && \
    OUT_DIR=$([ "${CARGO_PROFILE}" = "dev" ] && echo debug || echo "${CARGO_PROFILE}") && \
    cp target/${OUT_DIR}/fq-mcp /build/fq-mcp

# ==============================================================================
# Runtime Stage: Minimal runtime image
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM debian:bookworm-slim@sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818

# libstdc++6/libgomp1 are required by the CTranslate2 translation backend
RUN apt-get update && apt-get upgrade -y && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    libstdc++6 \
    libgomp1 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app

COPY --from=builder /build/fq-mcp /app/fq-mcp

# Pre-create the Hugging Face cache dir with appuser ownership so a named
# volume mounted at HF_HOME inherits it. Small per-language opus-mt models
# (~80-210 MB each) are downloaded under $HF_HOME/argos on first use; mount a
# volume to persist them across deploys.
RUN chown -R appuser:appuser /app \
    && mkdir -p /home/appuser/.cache/huggingface \
    && chown -R appuser:appuser /home/appuser/.cache

USER appuser

ENV HF_HOME=/home/appuser/.cache/huggingface
ENV MCP_TRANSPORT=http
ENV MCP_ADDR=0.0.0.0:3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["/app/fq-mcp"]
