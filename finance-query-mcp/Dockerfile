# ==============================================================================
# Build Stage: Compile fq-mcp MCP server
# ==============================================================================
FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

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
    cargo build --release -p finance-query-mcp && \
    rm -rf src server/src finance-query-derive/src finance-query-cli/src finance-query-mcp/src benches

# Copy actual source code
COPY src ./src
COPY finance-query-derive/src ./finance-query-derive/src
COPY finance-query-mcp/src ./finance-query-mcp/src

# Build the actual application
RUN touch src/lib.rs finance-query-mcp/src/main.rs && \
    cargo build --release -p finance-query-mcp

# ==============================================================================
# Runtime Stage: Minimal runtime image
# ==============================================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app

COPY --from=builder /build/target/release/fq-mcp /app/fq-mcp

RUN chown -R appuser:appuser /app

USER appuser

ENV MCP_TRANSPORT=http
ENV MCP_ADDR=0.0.0.0:3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["/app/fq-mcp"]
