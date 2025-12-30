# Getting Started

## Installation

Add `finance-query` to your `Cargo.toml`:

```toml
[dependencies]
finance-query = "2.0"
tokio = { version = "1", features = ["full"] }
```

### Optional Features

Enable features like `dataframe` for Polars integration:

```toml
[dependencies]
finance-query = { version = "2.0", features = ["dataframe"] }
```

## Basic Usage

### Ticker API

The `Ticker` struct is your entry point for symbol-specific data (quotes, charts, financials).

```rust
use finance_query::Ticker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a ticker
    let ticker = Ticker::new("AAPL").await?;

    // 2. Fetch data (lazy loaded)
    let quote = ticker.quote(true).await?;
    println!("{} price: ${:?}", quote.symbol, quote.regular_market_price);

    // 3. Get historical data
    let chart = ticker.chart(
        finance_query::Interval::OneDay, 
        finance_query::TimeRange::OneMonth
    ).await?;
    println!("Retrieved {} candles", chart.candles.len());

    Ok(())
}
```

### General Functions

Use the `finance` module for market-wide queries like search, screeners, and news.

```rust
use finance_query::{finance, SearchOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Search for a company and get news
    let options = SearchOptions::new().news_count(5);
    let results = finance::search("NVIDIA", &options).await?;

    println!("Found {} quotes", results.quotes.len());
    
    for news in results.news {
        println!("News: {} ({})", news.title, news.publisher);
    }

    Ok(())
}
```
