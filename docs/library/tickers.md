# Tickers API Reference

The `Tickers` struct provides efficient batch operations for multiple symbols. It optimizes network usage by grouping requests where possible and executing concurrent fetches where necessary.

!!! info "Single Symbol"
    For detailed operations on a single symbol (financials, options, detailed analysis), see the [`Ticker`](ticker.md) struct.

## Creation

### Simple Construction

```rust
use finance_query::Tickers;

let tickers = Tickers::new(vec!["AAPL", "MSFT", "GOOGL"]).await?;
```

### Builder Pattern

For advanced configuration (region, timeout, proxy), use the builder:

```rust
use finance_query::{Tickers, Region};
use std::time::Duration;

let tickers = Tickers::builder(vec!["AAPL", "MSFT"])
    .region(Region::UnitedStates)
    .timeout(Duration::from_secs(30))
    .build()
    .await?;
```

## Batch Quotes

Fetch quotes for all symbols in a single API call. This is significantly more efficient than fetching quotes individually.

```rust
// Fetch quotes for all symbols, including their logos if available
let response = tickers.quotes(true).await?;

// Process successful quotes
for (symbol, quote) in &response.quotes {
    println!("{} Price: ${:.2}", symbol, quote.regular_market_price.unwrap_or(0.0));
    if let Some(logo) = &quote.logo_url {
        println!("  Logo: {}", logo);
    }
}

// Handle errors
for (symbol, error) in &response.errors {
    eprintln!("Failed to fetch {}: {}", symbol, error);
}
```

### Response Structure

`BatchQuotesResponse` contains:

- `quotes`: `HashMap<String, Quote>` - Successfully fetched quotes grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Charts

Fetch historical data for all symbols concurrently. While Yahoo Finance doesn't support batch chart requests, `Tickers` handles concurrent fetching automatically.

```rust
use finance_query::{Interval, TimeRange};

// Fetch charts concurrently
let response = tickers.charts(Interval::OneDay, TimeRange::OneMonth).await?;

// Process successful charts
for (symbol, chart) in &response.charts {
    println!("{}: {} candles", symbol, chart.candles.len());
    if let Some(last) = chart.candles.last() {
        println!("  Last Close: ${:.2}", last.close);
    }
}

// Handle errors
for (symbol, error) in &response.errors {
    eprintln!("Failed to fetch chart for {}: {}", symbol, error);
}
```

### Response Structure

`BatchChartsResponse` contains:

- `charts`: `HashMap<String, Chart>` - Successfully fetched charts grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Individual Access

You can also access individual symbols from the `Tickers` instance. If the data is already cached from a batch operation, it returns immediately. If not, it triggers a batch fetch (for quotes) or single fetch (for charts).

```rust
// Get single quote (uses cache if available)
let aapl = tickers.quote("AAPL", true).await?;

// Get single chart (uses cache if available)
let msft_chart = tickers.chart("MSFT", Interval::OneDay, TimeRange::OneMonth).await?;
```

## Caching

`Tickers` maintains an internal cache to prevent redundant network requests.

```rust
// First call: Network request
let response1 = tickers.quotes(false).await?;

// Second call: Returns cached data (no network request)
let response2 = tickers.quotes(false).await?;

// Clear cache to force fresh data
tickers.clear_cache().await;
let response3 = tickers.quotes(false).await?; // Network request
```

## Best Practices

!!! tip "Optimize Batch Operations"
    - **Group symbols** - Use `Tickers` whenever you need data for multiple symbols (e.g., a portfolio or watchlist)
    - **Handle partial failures** - Always check the `errors` map in responses. One invalid symbol shouldn't fail the entire batch
    - **Reuse instances** - Keep the `Tickers` instance alive to benefit from caching across multiple operations

    ```rust
    // Good: Reuse Tickers instance for multiple operations
    let tickers = Tickers::new(vec!["AAPL", "GOOGL", "INVALID", "MSFT"]).await?;

    // First operation - fetches data
    let quotes_response = tickers.quotes(true).await?;

    // Handle partial failures - check which symbols failed
    for (symbol, error) in &quotes_response.errors {
        println!("Failed to fetch {}: {}", symbol, error);
    }

    // Process successful results
    for quote in &quotes_response.quotes {
        println!("{}: ${}", quote.symbol, quote.regular_market_price);
    }

    // Second operation - uses cached data (no network request)
    let charts_response = tickers.charts(Interval::OneDay, TimeRange::OneMonth).await?;

    // Less efficient: Creating new instances each time
    // (loses caching benefits, re-authenticates with Yahoo each time)
    let tickers1 = Tickers::new(vec!["AAPL", "GOOGL"]).await?;
    let quotes = tickers1.quotes(true).await?;
    let tickers2 = Tickers::new(vec!["AAPL", "GOOGL"]).await?;
    let charts = tickers2.charts(Interval::OneDay, TimeRange::OneMonth).await?;
    ```
