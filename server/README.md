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
RATE_LIMIT_PER_MINUTE=60         # Optional, default 60
EDGAR_EMAIL=you@example.com      # Required for EDGAR endpoints
```

## Endpoints

All endpoints are prefixed with `/v2`.

### Single Symbol

| Route | Description |
|-------|-------------|
| `GET /v2/quote/{symbol}` | Current quote |
| `GET /v2/quote-type/{symbol}` | Quote type metadata |
| `GET /v2/chart/{symbol}` | Historical OHLCV data |
| `GET /v2/options/{symbol}` | Options chain |
| `GET /v2/recommendations/{symbol}` | Similar stocks + analyst ratings |
| `GET /v2/news/{symbol}` | Company news |
| `GET /v2/holders/{symbol}/{holder_type}` | Major/institutional/insider holders |
| `GET /v2/analysis/{symbol}/{analysis_type}` | Recommendations, earnings, or upgrades |
| `GET /v2/financials/{symbol}/{statement}` | Income/balance/cashflow statements |
| `GET /v2/dividends/{symbol}` | Dividend history |
| `GET /v2/splits/{symbol}` | Stock split history |
| `GET /v2/capital-gains/{symbol}` | Capital gains history |
| `GET /v2/indicators/{symbol}` | Technical indicators |
| `GET /v2/transcripts/{symbol}` | Latest earnings call transcript |
| `GET /v2/transcripts/{symbol}/all` | All earnings call transcripts |

### Batch (Multiple Symbols)

| Route | Description |
|-------|-------------|
| `GET /v2/quotes` | Quotes for multiple symbols |
| `GET /v2/charts` | OHLCV data for multiple symbols |
| `GET /v2/spark` | Sparkline data for multiple symbols |
| `GET /v2/dividends` | Dividend history for multiple symbols |
| `GET /v2/splits` | Stock split history for multiple symbols |
| `GET /v2/capital-gains` | Capital gains for multiple symbols |
| `GET /v2/financials` | Financial statements for multiple symbols |
| `GET /v2/recommendations` | Recommendations for multiple symbols |
| `GET /v2/options` | Options chains for multiple symbols |
| `GET /v2/indicators` | Technical indicators for multiple symbols |

### Market-Wide

| Route | Description |
|-------|-------------|
| `GET /v2/search` | Search stocks, news, and research reports |
| `GET /v2/lookup` | Type-filtered ticker discovery |
| `GET /v2/screeners/{screener_type}` | Pre-built screeners (gainers, losers, etc.) |
| `POST /v2/screeners/custom` | Custom screener query |
| `GET /v2/trending` | Trending tickers |
| `GET /v2/market-summary` | Market overview with sparklines |
| `GET /v2/news` | General financial news |
| `GET /v2/currencies` | Currency and commodity data |
| `GET /v2/exchanges` | Supported exchanges |
| `GET /v2/hours` | Market hours and status |
| `GET /v2/indices` | World market indices |
| `GET /v2/sectors/{sector_type}` | Sector data (11 GICS sectors) |
| `GET /v2/industries/{industry_key}` | Industry data by slug |

### EDGAR (SEC Filings)

Requires `EDGAR_EMAIL` environment variable.

| Route | Description |
|-------|-------------|
| `GET /v2/edgar/cik/{symbol}` | Resolve ticker to CIK number |
| `GET /v2/edgar/submissions/{symbol}` | Filing history and company metadata |
| `GET /v2/edgar/facts/{symbol}` | Structured XBRL financial data |
| `GET /v2/edgar/search` | Full-text search of filings |

### WebSocket

| Route | Description |
|-------|-------------|
| `WS /v2/stream` | Real-time price streaming |

### Infrastructure

| Route | Description |
|-------|-------------|
| `GET /v2/health` | Health check |
| `GET /v2/ping` | Ping |
| `GET /v2/metrics` | Prometheus metrics (text format) |

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
