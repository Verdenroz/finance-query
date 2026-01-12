<h1 align="center">Finance Query</h1>

<p align="center">
  <img src="assets/logo.png" alt="Finance Query" width="187">
</p>

[![Crates.io](https://img.shields.io/crates/v/finance-query.svg)](https://crates.io/crates/finance-query)
[![Documentation](https://docs.rs/finance-query/badge.svg)](https://docs.rs/finance-query)
[![Build Status](https://github.com/Verdenroz/finance-query/workflows/CI/badge.svg)](https://github.com/Verdenroz/finance-query/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Finance Query** is a Rust library, CLI, and server for financial data, inspired by the popular `yfinance` Python library. It provides access to real-time quotes, historical charts, technical analysis, and financial statements primarily from Yahoo Finance.

A free hosted API is available at **[finance-query.com](https://finance-query.com)** — no setup required!

It is designed to be used in three ways:

*   **Rust Library**: A type-safe crate for direct integration into your Rust projects.
*   **Command-Line Tool**: Interactive `fq` tool for market data, analysis, and trading.
*   **REST & WebSocket Server**: A standalone service that exposes the library's functionality over HTTP.

---

## Documentation

=== "Library"

    ### Getting Started

    For installation instructions and a quick start guide, see [Getting Started](library/getting-started.md).

    ### Reference

    *   [Ticker API](library/ticker.md)
    *   [Batch Tickers](library/tickers.md)
    *   [Finance Module](library/finance.md)
    *   [Technical Indicators](library/indicators.md)
    *   [Backtesting](library/backtesting.md)
    *   [DataFrame Support](library/dataframe.md)
    *   [Models](library/models.md)
    *   [Configuration](library/configuration.md)

=== "CLI"

    ### Getting Started

    Install the command-line tool:

    ```bash
    cargo install finance-query-cli
    ```

    Or download pre-built binaries for [Linux, macOS, and Windows](cli/installation.md).

    ### Quick Start

    ```bash
    fq quote AAPL                              # Get a quote
    fq stream AAPL TSLA NVDA                   # Live prices
    fq chart AAPL -r 6mo                       # Interactive chart
    fq indicator AAPL --indicator rsi:14 --no-tui  # Technical analysis
    fq backtest AAPL --preset swing            # Backtest strategy
    fq dashboard                               # Interactive dashboard
    ```

    ### Reference

    *   [Installation & Quick Start](cli/installation.md)
    *   [Commands Reference](cli/commands.md)
    *   [Examples](cli/examples.md)

=== "Server"

    ### Getting Started

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

    ### Reference

    *   [REST API Reference](server/api-reference.md)
    *   [WebSocket API Reference](server/websocket-api-reference.md)
    *   [OpenAPI Specification](https://github.com/Verdenroz/finance-query/blob/main/server/openapi.yaml)
    *   [AsyncAPI Specification](https://github.com/Verdenroz/finance-query/blob/main/server/asyncapi.yaml)

---

## Example Usage

Finance Query is ready to use out of the box. Here's how to get stock data:

=== "Rust Library"

    ```rust
    use finance_query::{Ticker, Interval, TimeRange};

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Get detailed quote for Apple
        let ticker = Ticker::new("AAPL").await?;
        let quote = ticker.quote(true).await?;
        println!("{} price: ${:?}", quote.symbol, quote.regular_market_price);

        // Get historical charts
        let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
        println!("Retrieved {} candles", chart.candles.len());

        Ok(())
    }
    ```

=== "CLI"

    ```bash
    # Get a stock quote
    fq quote AAPL

    # Multiple quotes
    fq quote AAPL MSFT GOOGL AMZN

    # Stream live prices
    fq stream AAPL TSLA NVDA

    # Interactive price chart
    fq chart AAPL -r 6mo

    # Technical indicators
    fq indicator AAPL --indicator rsi:14 --no-tui

    # Backtest a strategy
    fq backtest AAPL --preset swing

    # Market dashboard
    fq dashboard
    ```

=== "REST API"

    ```bash
    # Get detailed quote for Apple
    curl "https://finance-query.com/v2/quote/AAPL?logo=true"

    # Get historical chart data
    curl "https://finance-query.com/v2/chart/AAPL?interval=1d&range=1mo"

    # Search for symbols
    curl "https://finance-query.com/v2/lookup?q=Apple"

    # Get predefined screeners
    curl "https://finance-query.com/v2/screeners/most-actives"

    # Get company news
    curl "https://finance-query.com/v2/news/AAPL"
    ```

=== "WebSocket"

    ```javascript
    // Connect to WebSocket for real-time updates
    const ws = new WebSocket('wss://finance-query.com/v2/stream');

    ws.onopen = () => {
        console.log('Connected to Finance Query WebSocket');
        ws.send(JSON.stringify({
            action: 'subscribe',
            symbols: ['AAPL', 'NVDA']
        }));
    };

    ws.onmessage = (event) => {
        const update = JSON.parse(event.data);
        console.log('Real-time update:', update);
    };
    ```

---

## Legal

This library fetches data from Yahoo Finance. Use responsibly and be aware of Yahoo's rate limits and terms of service.
