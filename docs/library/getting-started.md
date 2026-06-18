# Getting Started

!!! abstract "Cargo Docs"
    [docs.rs/finance-query](https://docs.rs/finance-query/latest/finance_query/)

## Installation

Add `finance-query` to your `Cargo.toml`:

```toml
[dependencies]
finance-query = "2.0"
tokio = { version = "1", features = ["full"] }
```

### Optional Features

```toml
[dependencies]
finance-query = { version = "2.0", features = ["dataframe", "backtesting"] }
```

| Feature | Description |
|---------|-------------|
| `polygon` | Polygon.io API (5 req/sec free) |
| `fmp` | Financial Modeling Prep API (250 req/day free) |
| `alphavantage` | Alpha Vantage API (25 req/day free) |
| `crypto` | CoinGecko cryptocurrency data (keyless, 30 req/min) |
| `fred` | FRED macro-economic data (120 req/min, free API key) |
| `dataframe` | Polars DataFrame integration for data analysis |
| `backtesting` | Strategy backtesting engine (includes `indicators`) |
| `indicators` | 52+ technical indicators (auto-enabled with `backtesting`) |
| `risk` | Risk analytics: VaR, Sharpe/Sortino/Calmar, beta, drawdown (includes `indicators`) |
| `rss` | RSS/Atom news feed aggregation |
| `sentiment` | Offline VADER sentiment scoring for news titles and transcripts (keyless) |
| `translation` | Translate human-readable response fields (built-in dictionary + pluggable backend) |
| `translation-offline` | Local opus-mt machine-translation backend (needs `cmake` + a C++ toolchain) |

## Quick Example

```rust
use finance_query::{Ticker, Interval, TimeRange, Raw};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default: Yahoo Finance (no API key required)
    let ticker = Ticker::builder("AAPL").logo().build().await?;

    // Get quote
    let quote = ticker.quote::<Raw>().await?;
    println!("{}: ${:.2}", quote.symbol,
        quote.regular_market_price.unwrap_or(0.0));

    // Get chart
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    println!("Candles: {}", chart.candles.len());

    Ok(())
}
```

## Multi-Provider Data Sources

Finance Query supports multiple data providers through feature flags:

```bash
export POLYGON_API_KEY="your-key"
export FMP_API_KEY="your-key"
```

```rust
use finance_query::{Capability, Fetch, Provider, Providers};

// Route quote to Polygon, fall back to Yahoo (routing lives on Providers::builder)
let providers = Providers::builder()
    .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build()
    .await?;
let ticker = providers.ticker("AAPL").build().await?;
```

→ [Multi-Provider Architecture](providers/index.md) for all providers and strategies

## Key Features

### 📊 Stock Data & Analysis

```rust
use finance_query::format::Raw;

// Quotes, financials, options, news
let ticker = Ticker::builder("MSFT").logo().build().await?;
let quote = ticker.quote::<Raw>().await?; // fetch quote with logo if available
let financials = ticker.financial_data().await?;
let options = ticker.options(None).await?;
```

→ [Ticker API](ticker.md) for complete reference

### 📦 Batch Operations

```rust
// Fetch multiple symbols efficiently
let tickers = Tickers::builder(vec!["AAPL", "MSFT", "GOOGL"]).logo().build().await?;
let quotes = tickers.quotes().await?; // fetch quotes with logos if available
let sparks = tickers.spark(Interval::OneDay, TimeRange::FiveDays).await?;
```

→ [Batch Tickers](tickers.md) for multi-symbol operations

### 🔍 Market Discovery

```rust
use finance_query::{finance, Screener, SearchOptions};

// Search, screeners, trending stocks
let results = finance::search("Tesla", &SearchOptions::default()).await?;
let actives = finance::screener(Screener::MostActives, 25).await?;
let trending = finance::trending(None).await?;
```

→ [Finance Module](finance.md) for market-wide data

### 📊 DataFrame Support

```rust
// Convert to Polars DataFrames
let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let df = chart.to_dataframe()?;
```

→ [DataFrame Support](dataframe.md) for data analysis

### 📈 Technical Indicators

```rust
// 52+ indicators: RSI, MACD, Bollinger Bands, etc.
let indicators = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;

if let Some(rsi) = indicators.rsi_14 {
    println!("RSI: {:.2}", rsi);
}
```

→ [Technical Indicators](indicators.md) for all available indicators

### 🔬 Backtesting

```rust
use finance_query::backtesting::SmaCrossover;

// Test strategies against historical data
let result = ticker.backtest(
    SmaCrossover::new(10, 20),
    Interval::OneDay,
    TimeRange::OneYear,
    None,
).await?;

println!("Return: {:.2}%", result.metrics.total_return_pct);
```

→ [Backtesting](backtesting.md) for strategy building

### 📡 Real-time Streaming

```rust
use finance_query::streaming::PriceStream;
use futures::StreamExt;

// Subscribe to real-time price updates via WebSocket
let mut stream = PriceStream::subscribe(&["AAPL", "NVDA", "TSLA"]).await?;

while let Some(price) = stream.next().await {
    println!("{}: ${:.2} ({:+.2}%)",
        price.id,
        price.price,
        price.change_percent
    );
}
```

→ [Real-time Streaming](streaming.md) for WebSocket details

### 📁 SEC EDGAR Filings

```rust
use finance_query::edgar;

// Init once per process (SEC requires contact email)
edgar::init("user@example.com")?;

// Resolve ticker to CIK number
let cik = edgar::resolve_cik("AAPL").await?;  // 320193

// Fetch all SEC filings metadata
let submissions = edgar::submissions(cik).await?;
if let Some(recent) = submissions.filings.as_ref().and_then(|f| f.recent.as_ref()) {
    println!("Recent filings: {}", recent.form.len());
}

// Fetch structured XBRL financial data
let facts = edgar::company_facts(cik).await?;
```

→ [EDGAR Module](providers/edgar.md) for SEC filing data

### ⚠️ Risk Analytics

```rust
// VaR, Sharpe/Sortino/Calmar ratio, Beta, max drawdown
let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, Some("SPY")).await?;

println!("VaR 95%:      {:.2}%", summary.var_95 * 100.0);
println!("Sharpe:       {:.2}", summary.sharpe.unwrap_or(0.0));
println!("Max Drawdown: {:.2}%", summary.max_drawdown * 100.0);
println!("Beta vs SPY:  {:.2}", summary.beta.unwrap_or(0.0));
```

→ [Risk Analytics](risk.md) for portfolio risk metrics

## Next Steps

**Start Here:**

- [Ticker API](ticker.md) - Single symbol operations
- [Multi-Provider Architecture](providers/index.md) - Configure and combine data providers
- [Technical Indicators](indicators.md) - RSI, MACD, Bollinger Bands, and more
- [Backtesting](backtesting.md) - Test trading strategies

**Advanced:**

- [Batch Tickers](tickers.md) - Multi-symbol efficiency
- [Finance Module](finance.md) - Market-wide searches
- [DataFrame Support](dataframe.md) - Data analysis with Polars
- [Configuration](configuration.md) - Regional settings and customization
- [Real-time Streaming](streaming.md) - WebSocket price feeds
- [SEC EDGAR](providers/edgar.md) - SEC filings and XBRL data
- [Risk Analytics](risk.md) - Portfolio risk metrics
