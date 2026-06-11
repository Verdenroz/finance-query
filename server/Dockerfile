# ==============================================================================
# Build Stage: Compile Rust application
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM rust:1-slim-bookworm@sha256:b5f842fac1e3b4ff718a652a8e0173b62d9403ec826ef4998880b9347db30684 AS builder

# Install build dependencies.
# cmake/g++/make compile CTranslate2 for the translation-offline feature.
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
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > finance-query-derive/src/lib.rs && \
    echo "fn main() {}" > finance-query-cli/src/main.rs && \
    echo "fn main() {}" > finance-query-mcp/src/main.rs && \
    for b in indicators backtesting ticker tickers finance stream; do \
        echo "fn main() {}" > benches/$b.rs; \
    done && \
    cargo build --release -p finance-query-server --features translation-offline && \
    rm -rf src server/src finance-query-derive/src finance-query-cli/src finance-query-mcp/src benches

# Copy actual source code
COPY src ./src
COPY server/src ./server/src
COPY finance-query-derive/src ./finance-query-derive/src
COPY benches ./benches

# Build the actual application
RUN touch src/lib.rs server/src/main.rs && \
    cargo auditable build --release -p finance-query-server --features translation-offline

# ==============================================================================
# Runtime Stage: Minimal runtime image
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM debian:bookworm-slim@sha256:0104b334637a5f19aa9c983a91b54c89887c0984081f2068983107a6f6c21eeb

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

COPY --from=builder /build/target/release/finance-query-server /app/finance-query-server

RUN chown -R appuser:appuser /app

USER appuser

# Environment variables (matches server/.env.template)
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
