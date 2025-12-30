<h1 align="center">Finance Query</h1>

<p align="center">
  <img src=".github/assets/logo.png" alt="FinanceQuery" width="187">
</p>

[![Crates.io](https://img.shields.io/crates/v/finance-query.svg)](https://crates.io/crates/finance-query)
[![Documentation](https://docs.rs/finance-query/badge.svg)](https://docs.rs/finance-query)
[![CI](https://github.com/Verdenroz/finance-query/actions/workflows/ci.yml/badge.svg)](https://github.com/Verdenroz/finance-query/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library and HTTP/WebSocket server for fetching financial data from Yahoo Finance.

## What's in This Repository

This repository mtaintains two services:

- **Library** (`finance-query`) - Rust crate for programmatic access to Yahoo Finance data
- **Server** (`finance-query-server`) - HTTP REST API and WebSocket server built on the library

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

**Full documentation is available at [verdenroz.github.io/finance-query](https://verdenroz.github.io/finance-query)**

- [Library Guide](https://verdenroz.github.io/finance-query/library/getting-started/) - Getting started with the Rust library
- [REST API Reference](https://verdenroz.github.io/finance-query/server/api-reference/) - Interactive OpenAPI documentation
- [WebSocket API Reference](https://verdenroz.github.io/finance-query/server/websocket-api-reference/) - Real-time streaming API
- [Contributing](https://verdenroz.github.io/finance-query/development/contributing/) - Development setup and guidelines

Additional resources:

- [Rust API Docs](https://docs.rs/finance-query) - Detailed API documentation on docs.rs
- [Crates.io](https://crates.io/crates/finance-query) - Published crate

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
