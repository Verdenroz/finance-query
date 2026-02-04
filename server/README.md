# finance-query-server

HTTP REST API and WebSocket server for financial data. Built on the [finance-query](https://crates.io/crates/finance-query) library.

A hosted version is available at [finance-query.com](https://finance-query.com).

## Running

```bash
# From repository root
make serve

# Or directly
cargo run -p finance-query-server
```

## Configuration

Copy `.env.template` to `.env`:

```bash
PORT=8000
RUST_LOG=info
REDIS_URL=redis://localhost:6379  # Optional
```

## Endpoints

All endpoints are prefixed with `/v2`.

### Quotes & Market Data
- `GET /v2/quote/:symbols` - Get quotes (comma-separated)
- `GET /v2/simple-quote/:symbols` - Minimal quote data
- `GET /v2/spark` - Sparkline data for multiple symbols
- `GET /v2/chart/:symbol` - Historical OHLCV data
- `GET /v2/market` - Market summary
- `GET /v2/trending` - Trending symbols
- `GET /v2/indices` - World indices
- `GET /v2/sector` - Sector performance

### Company Data
- `GET /v2/info/:symbol` - Full company info
- `GET /v2/financials/:symbol` - Financial statements
- `GET /v2/earnings/:symbol` - Earnings data
- `GET /v2/options/:symbol` - Options chain
- `GET /v2/news/:symbol` - Company news
- `GET /v2/recommendations/:symbol` - Analyst recommendations

### WebSocket
- `WS /v2/stream` - Real-time price streaming

### Utilities
- `GET /v2/lookup` - Symbol search
- `GET /v2/hours/:exchange` - Market hours
- `GET /v2/health` - Health check
- `GET /v2/metrics` - Prometheus metrics (text format)

## Features

- **Redis caching** (enabled by default with market-hours-aware TTLs) - Disable with `--no-default-features`
- **Rate limiting** - Governor-based rate limiting (60 requests/minute by default, configurable via `RATE_LIMIT_PER_MINUTE`)
- **Graceful shutdown** - Handles SIGTERM/SIGINT for clean WebSocket closure
- **CORS** - Configured for cross-origin requests
- **Compression** - gzip/brotli response compression

## Docker

```bash
docker build -t finance-query-server -f server/Dockerfile .
docker run -p 8000:8000 finance-query-server
```

## License

MIT
