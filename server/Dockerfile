# ==============================================================================
# Build Stage: Compile Rust application
# ==============================================================================
FROM rust:1.83-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy workspace and library manifests first (for layer caching)
COPY Cargo.toml Cargo.lock* ./
COPY server/Cargo.toml ./server/

# Create dummy sources to cache dependencies
RUN mkdir -p src server/src && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > server/src/main.rs && \
    cargo build --release -p finance-query-server && \
    rm -rf src server/src

# Copy actual source code (library + server)
COPY src ./src
COPY server/src ./server/src

# Build the actual application
# Touch files to force rebuild after dummy build
RUN touch src/lib.rs server/src/main.rs && \
    cargo build --release -p finance-query-server

# ==============================================================================
# Runtime Stage: Minimal runtime image
# ==============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 -s /bin/bash appuser

# Set working directory
WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /build/target/release/finance-query-server /app/finance-query-server

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Build arguments for configuration with defaults
ARG LOG_LEVEL=INFO
ARG LOG_FORMAT=json
ARG PERFORMANCE_THRESHOLD_MS=2000
ARG DISABLE_LOGO_FETCHING=false
ARG LOGO_TIMEOUT_SECONDS=1
ARG APP_RATE_LIMIT_PER_DAY=8000

# Set environment variables with defaults
# Logging configuration
ENV LOG_LEVEL=${LOG_LEVEL}
ENV LOG_FORMAT=${LOG_FORMAT}
ENV PERFORMANCE_THRESHOLD_MS=${PERFORMANCE_THRESHOLD_MS}

# Logo fetching configuration
ENV DISABLE_LOGO_FETCHING=${DISABLE_LOGO_FETCHING}
ENV LOGO_TIMEOUT_SECONDS=${LOGO_TIMEOUT_SECONDS}

# Rate limiting configuration
ENV APP_RATE_LIMIT_PER_DAY=${APP_RATE_LIMIT_PER_DAY}

# Server configuration
ENV PORT=8000

# Expose the port the application will run on
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/finance-query-server", "--version"] || exit 1

# Run the application
CMD ["/app/finance-query-server"]
