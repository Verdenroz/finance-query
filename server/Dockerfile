# ==============================================================================
# Build Stage: Compile Rust application
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM rust:1-slim-bookworm@sha256:4732ca96fd086cb9be682050c3f0176288eebaac2b80aa2bcefccfaf198e1950 AS builder

# Install build dependencies.
# cmake/g++/make compile CTranslate2 + SentencePiece for the
# translation-offline feature (opus-mt bilingual models), enabled on the cargo
# build commands below.
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
    echo "pub fn dummy() {}" > server/src/lib.rs && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > finance-query-derive/src/lib.rs && \
    echo "fn main() {}" > finance-query-cli/src/main.rs && \
    echo "fn main() {}" > finance-query-mcp/src/main.rs && \
    for b in indicators backtesting ticker tickers finance stream translation risk providers regression serde dataframe feeds sentiment; do \
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
RUN touch src/lib.rs server/src/lib.rs server/src/main.rs && \
    cargo auditable build --release -p finance-query-server --features translation-offline

# ==============================================================================
# Runtime Stage: Minimal runtime image
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM debian:bookworm-slim@sha256:60eac759739651111db372c07be67863818726f754804b8707c90979bda511df

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
