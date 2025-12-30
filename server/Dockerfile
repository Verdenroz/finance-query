# ==============================================================================
# Build Stage: Compile Rust application
# ==============================================================================
FROM rust:slim AS builder

# Install build dependencies
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

# Create dummy sources to cache dependencies
RUN mkdir -p src server/src finance-query-derive/src && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "" > finance-query-derive/src/lib.rs && \
    cargo build --release -p finance-query-server && \
    rm -rf src server/src finance-query-derive/src

# Copy actual source code
COPY src ./src
COPY server/src ./server/src
COPY finance-query-derive/src ./finance-query-derive/src

# Build the actual application
RUN touch src/lib.rs server/src/main.rs && \
    cargo build --release -p finance-query-server

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
