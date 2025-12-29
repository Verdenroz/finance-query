<h1 align="center">Finance Query</h1>

<p align="center">
  <img src=".github/assets/logo.png" alt="FinanceQuery" width="187">
</p>

[![Crates.io](https://img.shields.io/crates/v/finance-query.svg)](https://crates.io/crates/finance-query)
[![Documentation](https://docs.rs/finance-query/badge.svg)](https://docs.rs/finance-query)
[![CI](https://github.com/Verdenroz/finance-query/actions/workflows/ci.yml/badge.svg)](https://github.com/Verdenroz/finance-query/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for fetching financial data from Yahoo Finance, with an optional HTTP/WebSocket server.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
finance-query = "2.0"
```

### Optional Features

```toml
# Enable DataFrame conversions with Polars
finance-query = { version = "2.0", features = ["dataframe"] }
```

## Quick Start

```rust
use finance_query::Ticker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    // Quote data is lazy-loaded and cached
    let quote = ticker.quote(true).await?; // true = include logo
    println!("{}: ${}", quote.short_name, quote.regular_market_price);

    Ok(())
}
```

## Features

- **Real-time quotes** from Yahoo Finance
- **Historical data** with customizable intervals and ranges
- **Company information** including financials, holders, and analyst ratings
- **Market data** including indices, sectors, and movers
- **Technical indicators** (SMA, EMA, RSI, MACD, Bollinger Bands, etc.)
- **News and earnings transcripts**
- **WebSocket streaming** for real-time price updates
- **Optional DataFrame support** with Polars integration

## Examples

### Historical Data

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("AAPL").await?;
let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

for bar in &chart.bars {
    println!("{}: Open={:.2}, Close={:.2}", bar.date, bar.open, bar.close);
}
```

### Technical Indicators

```rust
let indicators = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;

println!("SMA 20: {:?}", indicators.sma20);
println!("RSI 14: {:?}", indicators.rsi14);
println!("MACD: {:?}", indicators.macd);
```

### Real-time Streaming

```rust
use finance_query::streaming::PriceStream;
use futures::StreamExt;

let mut stream = PriceStream::subscribe(&["AAPL", "NVDA", "TSLA"]).await?;

while let Some(update) = stream.next().await {
    println!("{}: ${:.2} ({:+.2}%)", update.id, update.price, update.change_percent);
}
```

### Multiple Symbols

```rust
use finance_query::Tickers;

let tickers = Tickers::new(vec!["AAPL", "GOOGL", "MSFT"]).await?;
let quotes = tickers.quotes(false).await?;

for (symbol, result) in quotes.results {
    match result {
        Ok(quote) => println!("{}: ${}", symbol, quote.regular_market_price),
        Err(e) => println!("{}: Error - {}", symbol, e),
    }
}
```

## Server Binary

A convenience HTTP/WebSocket server is included for those who want a REST API:

```bash
# Clone and run
git clone https://github.com/Verdenroz/finance-query.git
cd finance-query
make serve

# Or with Docker
docker build -f server/Dockerfile -t financequery:v2 .
docker run -p 8000:8000 financequery:v2
```

### Server Endpoints

| Endpoint | Description |
|----------|-------------|
| `/v2/quote/{symbol}` | Detailed quote |
| `/v2/quotes?symbols=...` | Batch quotes |
| `/v2/chart/{symbol}` | Historical OHLCV data |
| `/v2/indicators/{symbol}` | Technical indicators |
| `/v2/news/{symbol}` | Symbol news |
| `/v2/stream` | WebSocket streaming |

See the [server documentation](docs/server/overview.md) for full API reference.

## Python Version (v1)

The original Python implementation is available in the [`v1/`](./v1/) directory with its own documentation.

## Documentation

- [Library API Docs](https://docs.rs/finance-query)
- [Server API Reference](https://financequery.apidocumentation.com/)

## Development

```bash
make help          # Show all commands
make test          # Run tests
make lint          # Run linter
make docs-serve    # Serve documentation
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- Issues: [GitHub Issues](https://github.com/Verdenroz/finance-query/issues)
- Email: harveytseng2@gmail.com
