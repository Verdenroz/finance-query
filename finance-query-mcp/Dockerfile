# ==============================================================================
# Build Stage: Compile fq-mcp MCP server
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM rust:1-slim-bookworm@sha256:c8a94a78f67ec8c4d474ec7f71e0720f21eb7e584e158daec0874cafa7c30e4d AS builder

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
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > finance-query-derive/src/lib.rs && \
    echo "fn main() {}" > finance-query-cli/src/main.rs && \
    echo "fn main() {}" > finance-query-mcp/src/main.rs && \
    for b in indicators backtesting ticker tickers finance stream translation risk providers regression serde dataframe feeds sentiment; do \
        echo "fn main() {}" > benches/$b.rs; \
    done && \
    cargo build --release -p finance-query-mcp --features translation-offline && \
    rm -rf src server/src finance-query-derive/src finance-query-cli/src finance-query-mcp/src benches

# Copy actual source code
COPY src ./src
COPY finance-query-derive/src ./finance-query-derive/src
COPY finance-query-mcp/src ./finance-query-mcp/src
COPY benches ./benches

# Build the actual application
RUN touch src/lib.rs finance-query-mcp/src/main.rs && \
    cargo auditable build --release -p finance-query-mcp --features translation-offline

# ==============================================================================
# Runtime Stage: Minimal runtime image
# ==============================================================================
# Base image pinned by digest (tag kept for readability); bumped by Dependabot.
FROM debian:bookworm-slim@sha256:96e378d7e6531ac9a15ad505478fcc2e69f371b10f5cdf87857c4b8188404716

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

COPY --from=builder /build/target/release/fq-mcp /app/fq-mcp

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
