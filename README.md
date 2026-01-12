<h1 align="center">Finance Query</h1>

<p align="center">
  <img src=".github/assets/logo.png" alt="FinanceQuery" width="187" style="background:white; border-radius:8px; padding:8px;">
</p>

[![Crates.io](https://img.shields.io/crates/v/finance-query.svg)](https://crates.io/crates/finance-query)
[![Documentation](https://docs.rs/finance-query/badge.svg)](https://docs.rs/finance-query)
[![CI](https://github.com/Verdenroz/finance-query/actions/workflows/ci.yml/badge.svg)](https://github.com/Verdenroz/finance-query/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust library, CLI, and HTTP server for querying financial data.

## Hosted API

Free hosted version at **[finance-query.com](https://finance-query.com)**:

```bash
# Get a quote
curl "https://finance-query.com/v2/quote/AAPL"

# Real-time streaming
wscat -c "wss://finance-query.com/v2/stream"
```

## What's in This Repository

- **Library** (`finance-query`) - Rust crate for programmatic access to Yahoo Finance
- **CLI** (`finance-query-cli`) - Command-line tool for market data, technical analysis, and backtesting
- **Server** (`finance-query-server`) - HTTP REST API and WebSocket server
- **Derive Macros** (`finance-query-derive`) - Procedural macros for Polars DataFrame integration

## Quick Start

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
finance-query = "2.0"

# Or with DataFrame support (Polars integration)
finance-query = { version = "2.0", features = ["dataframe"] }
```

Basic usage:

```rust
use finance_query::Ticker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;
    let quote = ticker.quote(true).await?;
    println!("{}: ${}", quote.short_name, quote.regular_market_price);
    Ok(())
}
```

### CLI

Install `fq` (the command-line tool):

```bash
# Pre-built binary (Linux/macOS)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Verdenroz/finance-query/releases/latest/download/finance-query-cli-installer.sh | sh

# Or from crates.io
cargo install finance-query-cli
```

Quick examples:

```bash
fq quote AAPL MSFT GOOGL          # Get quotes
fq stream AAPL TSLA NVDA          # Live prices
fq chart AAPL -r 6mo              # Interactive price chart
fq indicator AAPL --indicator rsi:14  # Technical indicators
fq backtest AAPL --preset swing   # Strategy backtesting
fq dashboard                      # Market dashboard
fq alerts add AAPL price-above:200  # Price alerts with notifications
```

See [finance-query-cli/README.md](finance-query-cli/README.md) for full documentation.

### Server

Run the server locally (requires [Rust](https://rustup.rs/)):

```bash
git clone https://github.com/Verdenroz/finance-query.git
cd finance-query
make serve  # Compiles and runs v2 server
```

Or run both v1 and v2 with Docker Compose:

```bash
make docker-compose  # Starts v1 (port 8002), v2 (port 8001), Redis, and Nginx
```

The v2 server provides REST endpoints at `/v2/*` and WebSocket streaming at `/v2/stream`.

## Documentation

**Package guides:**

- [CLI](finance-query-cli/README.md) - Command-line tool with examples, installation, and features
- [Server](server/README.md) - REST API and WebSocket server setup and endpoints
- [Derive Macros](finance-query-derive/README.md) - Procedural macros for Polars DataFrame support

**Full documentation at [verdenroz.github.io/finance-query](https://verdenroz.github.io/finance-query):**

- [Library Getting Started](https://verdenroz.github.io/finance-query/library/getting-started/)
- [REST API Reference](https://verdenroz.github.io/finance-query/server/api-reference/)
- [WebSocket API](https://verdenroz.github.io/finance-query/server/websocket-api-reference/)
- [Contributing](https://verdenroz.github.io/finance-query/development/contributing/)

**API Documentation:**

- [Rust Docs](https://docs.rs/finance-query) - Library API on docs.rs
- [Crates.io](https://crates.io/crates/finance-query) - Published library
- [CLI on Crates.io](https://crates.io/crates/finance-query-cli) - Published CLI

## Legacy Python Version (v1)

The original Python implementation is available in the [`v1/`](./v1/) directory. It is no longer actively maintained but remains available for reference.

## Contributing

We welcome contributions! See the [Contributing Guide](https://verdenroz.github.io/finance-query/development/contributing/) for setup instructions and development workflow.

```bash
make install-dev  # Set up development environment
make test-fast    # Run tests
make fix          # Auto-fix formatting and linting
```

## Acknowledgements

This project relies on Yahoo Finance's publicly available data. We are grateful to Yahoo for providing this data.

Special thanks to [yfinance](https://github.com/ranaroussi/yfinance), the popular Python library that inspired this project. Many of the API patterns and data structures are adapted from yfinance's excellent work.

## License

MIT License - see [LICENSE](LICENSE) for details.
