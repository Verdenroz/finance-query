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
    let price = quote.regular_market_price.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
    println!("{} Price: ${:.2}", symbol, price);
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

## Spark Data

Fetch lightweight sparkline data for all symbols in a single batch request. Spark provides only timestamps and close prices, optimized for rendering sparklines in dashboards and watchlists.

```rust
use finance_query::{Interval, TimeRange};

// Fetch spark data for all symbols
let response = tickers.spark(Interval::OneDay, TimeRange::FiveDays).await?;

// Process successful sparks
for (symbol, spark) in &response.sparks {
    println!("{}: {} data points", symbol, spark.len());

    if let Some(change) = spark.percent_change() {
        println!("  Change: {:+.2}%", change);
    }

    if let Some(min) = spark.min_close() {
        println!("  Low: ${:.2}", min);
    }

    if let Some(max) = spark.max_close() {
        println!("  High: ${:.2}", max);
    }
}

// Handle errors
for (symbol, error) in &response.errors {
    eprintln!("Failed to fetch spark for {}: {}", symbol, error);
}
```

### Spark Structure

Each `Spark` contains:

- `symbol`: Stock symbol
- `meta`: Chart metadata (currency, exchange, timezone)
- `timestamps`: Vec of Unix timestamps
- `closes`: Vec of close prices
- `interval`: Time interval (e.g., "1d", "1h")
- `range`: Time range (e.g., "5d", "1mo")

### Available Methods

- `.len()` - Number of data points
- `.is_empty()` - Check if empty
- `.price_change()` - Absolute price change (last - first)
- `.percent_change()` - Percentage change
- `.min_close()` - Minimum close price
- `.max_close()` - Maximum close price

### Response Structure

`BatchSparksResponse` contains:

- `sparks`: `HashMap<String, Spark>` - Successfully fetched sparks grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Dividends

Fetch dividend history for all symbols. Dividends are filtered by the specified time range.

```rust
use finance_query::TimeRange;

// Fetch dividends for all symbols
let response = tickers.dividends(TimeRange::OneYear).await?;

// Process successful dividends
for (symbol, dividends) in &response.dividends {
    println!("{}: {} dividends", symbol, dividends.len());
    for div in dividends {
        println!("  Timestamp: {}, Amount: ${:.2}", div.timestamp, div.amount);
    }
}

// Handle errors
for (symbol, error) in &response.errors {
    eprintln!("Failed to fetch dividends for {}: {}", symbol, error);
}
```

### Response Structure

`BatchDividendsResponse` contains:

- `dividends`: `HashMap<String, Vec<Dividend>>` - Dividend history grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Splits

Fetch stock split history for all symbols. Particularly useful for tracking symbols like NVDA, TSLA, and AAPL which have had recent splits.

```rust
use finance_query::TimeRange;

// Fetch splits for symbols known to have splits
let tickers = Tickers::new(vec!["NVDA", "TSLA", "AAPL"]).await?;
let response = tickers.splits(TimeRange::FiveYears).await?;

// Process splits
for (symbol, splits) in &response.splits {
    if !splits.is_empty() {
        println!("{}: {} splits", symbol, splits.len());
        for split in splits {
            println!("  Timestamp: {}, Ratio: {}", split.timestamp, split.ratio);
        }
    }
}
```

### Response Structure

`BatchSplitsResponse` contains:

- `splits`: `HashMap<String, Vec<Split>>` - Split history grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Capital Gains

Fetch capital gains distribution history for all symbols. This is primarily used for mutual funds and ETFs.

```rust
use finance_query::TimeRange;

// Fetch capital gains for ETFs
let etfs = Tickers::new(vec!["SPY", "VOO", "VTI"]).await?;
let response = etfs.capital_gains(TimeRange::TwoYears).await?;

// Process capital gains
for (symbol, gains) in &response.capital_gains {
    if !gains.is_empty() {
        println!("{}: {} capital gains distributions", symbol, gains.len());
        for gain in gains {
            println!("  Timestamp: {}, Amount: ${:.2}", gain.timestamp, gain.amount);
        }
    }
}
```

### Response Structure

`BatchCapitalGainsResponse` contains:

- `capital_gains`: `HashMap<String, Vec<CapitalGain>>` - Capital gains grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Financials

Fetch financial statements for all symbols concurrently.

```rust
use finance_query::{StatementType, Frequency};

// Fetch quarterly income statements
let response = tickers.financials(StatementType::Income, Frequency::Quarterly).await?;

// Process financial statements
for (symbol, statement) in &response.financials {
    println!("{}: {} metrics", symbol, statement.statement.len());

    // Access specific metrics
    if let Some(revenue_data) = statement.statement.get("TotalRevenue") {
        println!("  Revenue data points: {}", revenue_data.len());

        // Get most recent revenue
        if let Some((date, value)) = revenue_data.iter().next() {
            println!("  Latest Revenue ({}): ${}", date, value);
        }
    }

    if let Some(income_data) = statement.statement.get("NetIncome") {
        if let Some((date, value)) = income_data.iter().next() {
            println!("  Latest Net Income ({}): ${}", date, value);
        }
    }
}
```

### Statement Types

- `StatementType::Income` - Income statements (revenue, expenses, net income)
- `StatementType::Balance` - Balance sheets (assets, liabilities, equity)
- `StatementType::CashFlow` - Cash flow statements (operating, investing, financing)

### Response Structure

`BatchFinancialsResponse` contains:

- `financials`: `HashMap<String, FinancialStatement>` - Financial statements grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch News

Fetch recent news articles for all symbols concurrently.

```rust
// Fetch news for all symbols
let response = tickers.news().await?;

// Process news
for (symbol, articles) in &response.news {
    println!("{}: {} news articles", symbol, articles.len());
    for article in articles.iter().take(3) {
        println!("  Title: {}", article.title);
        if let Some(summary) = &article.summary {
            println!("  Summary: {}", summary);
        }
    }
}
```

### Response Structure

`BatchNewsResponse` contains:

- `news`: `HashMap<String, Vec<News>>` - News articles grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Recommendations

Fetch similar stock recommendations for all symbols concurrently.

```rust
// Fetch recommendations with limit
let response = tickers.recommendations(5).await?;

// Process recommendations
for (symbol, rec) in &response.recommendations {
    println!("{}: {} recommendations", symbol, rec.recommendations.len());
    for r in &rec.recommendations {
        println!("  {} ({})", r.symbol, r.score);
    }
}
```

### Response Structure

`BatchRecommendationsResponse` contains:

- `recommendations`: `HashMap<String, Recommendation>` - Recommendations grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Options

Fetch options chains for all symbols concurrently.

```rust
// Fetch options for all symbols (nearest expiration)
let response = tickers.options(None).await?;

// Process options
for (symbol, options) in &response.options {
    let exp_dates = options.expiration_dates();
    println!("{}: {} expirations", symbol, exp_dates.len());

    // Show calls and puts for nearest expiration
    if let Some(chain) = &options.calls {
        println!("  Calls: {} contracts", chain.len());
    }
    if let Some(chain) = &options.puts {
        println!("  Puts: {} contracts", chain.len());
    }
}

// Fetch for specific expiration date (Unix timestamp)
let specific_date = 1735689600; // 2025-01-01
let response = tickers.options(Some(specific_date)).await?;
```

### Response Structure

`BatchOptionsResponse` contains:

- `options`: `HashMap<String, Options>` - Options chains grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Indicators

!!! note "Feature Flag Required"
    This feature requires the `indicators` feature flag to be enabled:

    ```toml
    [dependencies]
    finance-query = { version = "2.1", features = ["indicators"] }
    ```

Fetch technical indicators for all symbols concurrently.

```rust
#[cfg(feature = "indicators")]
{
    use finance_query::{Interval, TimeRange};

    // Fetch indicators for all symbols
    let response = tickers.indicators(Interval::OneDay, TimeRange::OneMonth).await?;

    // Process indicators
    for (symbol, indicators) in &response.indicators {
        println!("{} Indicators:", symbol);

        if let Some(rsi) = indicators.rsi_14 {
            println!("  RSI(14): {:.2}", rsi);
        }

        if let Some(sma_20) = indicators.sma_20.as_ref().and_then(|v| v.last()) {
            println!("  SMA(20): {:.2}", sma_20);
        }

        if let Some(macd) = &indicators.macd {
            println!("  MACD: {:.2}", macd.macd.last().unwrap_or(&0.0));
        }
    }
}
```

### Response Structure

`BatchIndicatorsResponse` contains:

- `indicators`: `HashMap<String, IndicatorsSummary>` - Indicators grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Dynamic Symbol Management

Add or remove symbols from a `Tickers` instance after creation. This is useful for managing watchlists or portfolios dynamically.

```rust
// Start with initial symbols
let mut tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;
println!("Initial symbols: {:?}", tickers.symbols());

// Add more symbols
tickers.add_symbols(&["GOOGL", "TSLA", "NVDA"]);
println!("After adding: {:?}", tickers.symbols());

// Remove symbols (also clears their cached data)
tickers.remove_symbols(&["MSFT", "TSLA"]).await;
println!("After removing: {:?}", tickers.symbols());

// Fetch quotes for current symbols
let response = tickers.quotes(false).await?;
// Response will only include AAPL, GOOGL, NVDA
```

!!! warning "Cache Clearing"
    When you remove symbols using `remove_symbols()`, all cached data for those symbols is also cleared. This ensures memory efficiency when managing large symbol lists.

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
    for (symbol, quote) in &quotes_response.quotes {
        let price = quote.regular_market_price.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
        println!("{}: ${:.2}", symbol, price);
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

## Next Steps

- [Ticker API](ticker.md) - Detailed operations for single symbols (financials, options, news)
- [DataFrame Support](dataframe.md) - Convert batch responses to Polars DataFrames for analysis
- [Configuration](configuration.md) - Customize regional settings and network options
- [Models Reference](models.md) - Understanding response types
