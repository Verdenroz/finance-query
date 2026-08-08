# Tickers API Reference

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Tickers](https://docs.rs/finance-query/latest/finance_query/struct.Tickers.html)

The `Tickers` struct provides efficient batch operations for multiple symbols. It optimizes network usage by grouping requests where possible and executing concurrent fetches where necessary.

!!! info "Single Symbol"
    For detailed operations on a single symbol (financials, options, detailed analysis), see the [`Ticker`](ticker.md) struct.

## Creation

### Simple Construction

```rust capture-output
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT", "GOOGL"]).await?;
    Ok(())
}
```

```text soothfast-output
```

### Builder Pattern

For advanced configuration (region, timeout, proxy), use the builder:

```rust capture-output covers=finance_query::tickers::core::TickersBuilder
use finance_query::{Region, Tickers};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::builder(vec!["AAPL", "MSFT"])
        .region(Region::UnitedStates)
        .timeout(Duration::from_secs(30))
        .build()
        .await?;
    Ok(())
}
```

```text soothfast-output
```

### Builder Options

<!-- soothfast:bind finance_query::tickers::core::TickersBuilder -->

| Method | Description |
|--------|-------------|
| `.region(Region)` | Set region (automatically sets lang + region code) |
| `.lang(str)` | Set language code (e.g., `"en-US"`, `"ja-JP"`) |
| `.region_code(str)` | Set region code directly (e.g., `"US"`, `"JP"`) |
| `.timeout(Duration)` | Set HTTP request timeout |
| `.proxy(str)` | Set proxy URL |
| `.max_concurrency(n)` | Max concurrent requests for per-symbol batch ops (default: 10) |
| `.logo()` | Include company logo URLs in quote responses |
| `.cache(Duration)` | Bound response caching to a TTL (default: cached for the handle's lifetime) |
| `.no_cache()` | Disable caching — every call fetches fresh data |

<!-- /soothfast:bind -->

#### `max_concurrency`

Controls parallelism for methods that fetch per-symbol (charts, financials, news, etc.). Lower values reduce the risk of rate limiting; higher values increase throughput for large symbol lists.

```rust capture-output
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Conservative: 3 concurrent requests (large lists or strict rate limits)
    let tickers = Tickers::builder(vec!["AAPL", "MSFT", "GOOGL", "TSLA"])
        .max_concurrency(3)
        .build()
        .await?;
    Ok(())
}
```

```text soothfast-output
```

## Provider Configuration

`Tickers` supports the same multi-provider configuration as [`Ticker`](ticker.md). Provider routing is configured through `Providers::builder()` (see [Multi-Provider Architecture](providers/index.md)), then passed to `Tickers` via `providers.tickers()`:

```rust capture-output
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let tickers = providers.tickers(["AAPL", "NVDA"]).build().await?;
    let response = tickers.quotes().await?;
    println!("quotes fetched: {}", response.quotes.len());
    Ok(())
}
```

```text soothfast-output
quotes fetched: 2
```

With multiple providers enabled (e.g. `polygon` feature), route capabilities to specific providers:

```rust no_run feature=polygon
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let tickers = providers.tickers(["AAPL", "NVDA"]).build().await?;
    Ok(())
}
```

!!! note "Spark is Yahoo-only"
    `spark()` uses a Yahoo-specific batch endpoint with no equivalent in other providers. It will always use the Yahoo client regardless of the configured provider set.

See [Multi-Provider Architecture](providers/index.md) for full details on providers and fetch strategies.

## Batch Quotes

Fetch quotes for all symbols in a single API call. This is significantly more efficient than fetching quotes individually.

```rust capture-output
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch quotes for all symbols, including their logos if available
    let tickers = Tickers::builder(vec!["AAPL", "MSFT"])
        .logo()
        .build()
        .await?;
    let response = tickers.quotes().await?;

    // Process successful quotes
    for (symbol, quote) in &response.quotes {
        let price = quote
            .regular_market_price
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("{} Price: ${:.2}", symbol, price);
        if let Some(logo) = &quote.logo_url {
            println!("  Logo: {}", logo);
        }
    }

    // Handle errors
    for (symbol, error) in &response.errors {
        eprintln!("Failed to fetch {}: {}", symbol, error);
    }
    Ok(())
}
```

```text soothfast-output
AAPL Price: $333.74
  Logo: https://s.yimg.com/lb/brands/50x50_apple.png
MSFT Price: $393.82
  Logo: https://s.yimg.com/lb/brands/50x50_microsoft.png
```

### Response Structure

`BatchQuotesResponse` contains:

- `quotes`: `HashMap<String, Quote>` - Successfully fetched quotes grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Charts

Fetch historical data for all symbols concurrently. While Yahoo Finance doesn't support batch chart requests, `Tickers` handles concurrent fetching automatically.

```rust capture-output
use finance_query::{Interval, Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

    // Fetch charts concurrently
    let response = tickers
        .charts(Interval::OneDay, TimeRange::OneMonth)
        .await?;

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
    Ok(())
}
```

```text soothfast-output
MSFT: 20 candles
  Last Close: $393.82
AAPL: 20 candles
  Last Close: $333.74
```

### Response Structure

`BatchChartsResponse` contains:

- `charts`: `HashMap<String, Chart>` - Successfully fetched charts grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Spark Data

Fetch lightweight sparkline data for all symbols in a single batch request. Spark provides only timestamps and close prices, optimized for rendering sparklines in dashboards and watchlists.

```rust capture-output covers=finance_query::models::chart::spark::Spark
use finance_query::{Interval, Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

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
    Ok(())
}
```

```text soothfast-output
MSFT: 5 data points
  Change: +0.72%
  Low: $384.93
  High: $401.10
AAPL: 5 data points
  Change: +5.18%
  Low: $314.86
  High: $333.74
```

### Spark Structure

<!-- soothfast:bind finance_query::models::chart::spark::Spark -->

Each `Spark` contains:

- `symbol`: Stock symbol
- `meta`: Chart metadata (currency, exchange, timezone)
- `timestamps`: Vec of Unix timestamps
- `closes`: Vec of close prices
- `interval`: Time interval (e.g., `"1d"`, `"1h"`)
- `range`: Time range (e.g., `"5d"`, `"1mo"`)

### Available Methods

- `.len()` - Number of data points
- `.is_empty()` - Check if empty
- `.price_change()` - Absolute price change (last - first)
- `.percent_change()` - Percentage change
- `.min_close()` - Minimum close price
- `.max_close()` - Maximum close price

<!-- /soothfast:bind -->

### Response Structure

`BatchSparksResponse` contains:

- `sparks`: `HashMap<String, Spark>` - Successfully fetched sparks grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Dividends

Fetch dividend history for all symbols. Dividends are filtered by the specified time range.

```rust capture-output
use finance_query::{Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

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
    Ok(())
}
```

```text soothfast-output
AAPL: 4 dividends
  Timestamp: 1754919000, Amount: $0.26
  Timestamp: 1762785000, Amount: $0.26
  Timestamp: 1770647400, Amount: $0.26
  Timestamp: 1778506200, Amount: $0.27
MSFT: 5 dividends
  Timestamp: 1755783000, Amount: $0.83
  Timestamp: 1763649000, Amount: $0.91
  Timestamp: 1771511400, Amount: $0.91
  Timestamp: 1771511400, Amount: $0.91
  Timestamp: 1779370200, Amount: $0.91
```

### Response Structure

`BatchDividendsResponse` contains:

- `dividends`: `HashMap<String, Vec<Dividend>>` - Dividend history grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Splits

Fetch stock split history for all symbols. Particularly useful for tracking symbols like NVDA, TSLA, and AAPL which have had recent splits.

```rust capture-output
use finance_query::{Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}
```

```text soothfast-output
TSLA: 1 splits
  Timestamp: 1661434200, Ratio: 3:1
NVDA: 2 splits
  Timestamp: 1626787800, Ratio: 4:1
  Timestamp: 1718026200, Ratio: 10:1
```

### Response Structure

`BatchSplitsResponse` contains:

- `splits`: `HashMap<String, Vec<Split>>` - Split history grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Capital Gains

Fetch capital gains distribution history for all symbols. This is primarily used for mutual funds and ETFs.

```rust capture-output
use finance_query::{Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch capital gains for ETFs
    let etfs = Tickers::new(vec!["SPY", "VOO", "VTI"]).await?;
    let response = etfs.capital_gains(TimeRange::TwoYears).await?;

    // Process capital gains
    for (symbol, gains) in &response.capital_gains {
        if !gains.is_empty() {
            println!("{}: {} capital gains distributions", symbol, gains.len());
            for gain in gains {
                println!(
                    "  Timestamp: {}, Amount: ${:.2}",
                    gain.timestamp, gain.amount
                );
            }
        }
    }
    Ok(())
}
```

```text soothfast-output
```

### Response Structure

`BatchCapitalGainsResponse` contains:

- `capital_gains`: `HashMap<String, Vec<CapitalGain>>` - Capital gains grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Financials

Fetch financial statements for all symbols concurrently.

```rust capture-output
use finance_query::{Frequency, StatementType, Tickers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

    // Fetch quarterly income statements
    let response = tickers
        .financials(StatementType::Income, Frequency::Quarterly)
        .await?;

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

        if let Some(income_data) = statement.statement.get("NetIncome")
            && let Some((date, value)) = income_data.iter().next()
        {
            println!("  Latest Net Income ({}): ${}", date, value);
        }
    }
    Ok(())
}
```

```text soothfast-output
AAPL: 26 metrics
  Revenue data points: 5
  Latest Revenue (2025-06-30): $94036000000
  Latest Net Income (2025-03-31): $24780000000
MSFT: 30 metrics
  Revenue data points: 5
  Latest Revenue (2025-03-31): $70066000000
  Latest Net Income (2025-12-31): $38458000000
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

```rust capture-output
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

    // Fetch news for all symbols
    let response = tickers.news().await?;

    // Process news
    for (symbol, articles) in &response.news {
        println!("{}: {} news articles", symbol, articles.len());
        for article in articles.iter().take(3) {
            println!("  Title: {}", article.title);
            println!("  Source: {}", article.source);
            println!("  Link: {}", article.link);
        }
    }
    Ok(())
}
```

```text soothfast-output
MSFT: 23 news articles
  Title: Microsoft Investigation Initiated: Kahn Swick & Foti, LLC Investigates the Officers and Directors of Microsoft Corporation - MSFT
  Source: PRNewsWire
  Link: https://www.prnewswire.com/news-releases/microsoft-investigation-initiated-kahn-swick--foti-llc-investigates-the-officers-and-directors-of-microsoft-corporation---msft-302828889.html
  Title: Microsoft stock falls, analysts trim price targets ahead of Q4 earnings
  Source: Invezz
  Link: https://invezz.com/news/2026/07/17/microsoft-stock-falls-analysts-trim-price-targets-ahead-of-q4-earnings/
  Title: Microsoft (MSFT) CEO Blasts Anthropic’s Fable 5, Says It “Doesn’t Make Sense”
  Source: TipRanks
  Link: https://www.tipranks.com/news/microsoft-msft-ceo-blasts-anthropics-fable-5-says-it-doesnt-make-sense
AAPL: 20 news articles
  Title: Apple briefly overtakes Nvidia as world's most valuable company amid AI investment doubts
  Source: Fox Business
  Link: https://www.foxbusiness.com/markets/apple-briefly-overtakes-nvidia-worlds-most-valuable-company-amid-ai-investment-doubts
  Title: Apple and Google ordered to purge ‘nudify' apps from App Stores
  Source: TechCrunch
  Link: https://techcrunch.com/2026/07/17/apple-and-google-ordered-to-purge-nudify-apps-from-app-stores/
  Title: Berkshire's Equity Portfolio Is Rallying, but the Apple Sales Still Sting
  Source: Barrons
  Link: https://www.barrons.com/articles/berkshire-hathaway-apple-stock-portfolio-240a36cc
```

### Response Structure

`BatchNewsResponse` contains:

- `news`: `HashMap<String, Vec<News>>` - News articles grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Recommendations

Fetch similar stock recommendations for all symbols concurrently.

```rust capture-output
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

    // Fetch recommendations with limit
    let response = tickers.recommendations(5).await?;

    // Process recommendations
    for (symbol, rec) in &response.recommendations {
        println!("{}: {} recommendations", symbol, rec.recommendations.len());
        for r in &rec.recommendations {
            println!("  {} ({})", r.symbol, r.score);
        }
    }
    Ok(())
}
```

```text soothfast-output
MSFT: 5 recommendations
  AAPL (0.147947)
  AMZN (0.141103)
  GOOG (0.12302)
  NVDA (0.122673)
  META (0.122301)
AAPL: 5 recommendations
  AMZN (0.190787)
  TSLA (0.17989)
  GOOG (0.167981)
  META (0.160631)
  MSFT (0.147947)
```

### Response Structure

`BatchRecommendationsResponse` contains:

- `recommendations`: `HashMap<String, Recommendation>` - Recommendations grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Batch Options

Fetch options chains for all symbols concurrently.

```rust capture-output
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

    // Fetch options for all symbols (nearest expiration)
    let response = tickers.options(None).await?;

    // Process options
    for (symbol, options) in &response.options {
        let exp_dates = options.expiration_dates();
        println!("{}: {} expirations", symbol, exp_dates.len());

        // Show calls and puts count for nearest expiration
        let calls = options.calls();
        let puts = options.puts();
        println!("  Calls: {} contracts", calls.len());
        println!("  Puts: {} contracts", puts.len());
    }

    // Fetch for specific expiration date (Unix timestamp)
    let specific_date = 1735689600; // 2025-01-01
    let response = tickers.options(Some(specific_date)).await?;
    Ok(())
}
```

```text soothfast-output
MSFT: 21 expirations
  Calls: 63 contracts
  Puts: 44 contracts
AAPL: 23 expirations
  Calls: 48 contracts
  Puts: 39 contracts
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
    finance-query = { version = "2", features = ["indicators"] }
    ```

Fetch technical indicators for all symbols concurrently.

```rust capture-output feature=indicators
use finance_query::{Interval, Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

    // Fetch indicators for all symbols
    let response = tickers
        .indicators(Interval::OneDay, TimeRange::OneMonth)
        .await?;

    // Process indicators
    for (symbol, indicators) in &response.indicators {
        println!("{} Indicators:", symbol);

        if let Some(rsi) = indicators.rsi_14 {
            println!("  RSI(14): {:.2}", rsi);
        }

        if let Some(sma) = indicators.sma_20 {
            println!("  SMA(20): {:.2}", sma);
        }

        if let Some(macd) = &indicators.macd
            && let Some(line) = macd.macd
        {
            println!("  MACD: {:.2}", line);
        }
    }
    Ok(())
}
```

```text soothfast-output
AAPL Indicators:
  RSI(14): 75.87
  SMA(20): 305.52
MSFT Indicators:
  RSI(14): 55.97
  SMA(20): 381.16
```

### Response Structure

`BatchIndicatorsResponse` contains:

- `indicators`: `HashMap<String, IndicatorsSummary>` - Indicators grouped by symbol
- `errors`: `HashMap<String, String>` - Error messages grouped by symbol

## Event Calendar

`calendar(range)` merges every symbol's upcoming events — earnings, dividends,
and standard monthly options expirations — into one list sorted ascending by
timestamp, fetched concurrently (bounded by `max_concurrency`). With the `fred`
feature, market-wide economic releases are appended once. It is best-effort per
symbol: a symbol whose fetch fails contributes no events rather than failing the
whole call. See the [Ticker Event Calendar](ticker.md#event-calendar) section
for the `CalendarEvent` / `EventKind` shapes.

```rust capture-output
use finance_query::{Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(["AAPL", "MSFT", "TSLA"]).await?;
    let events = tickers.calendar(TimeRange::OneMonth).await?;

    for event in &events {
        println!("{} {:?} {:?}", event.date, event.symbol, event.event);
    }
    Ok(())
}
```

```text soothfast-output
2026-07-22 Some("TSLA") Earnings { eps_estimate_low: Some(0.31), eps_estimate_avg: Some(0.53673), eps_estimate_high: Some(0.74), revenue_estimate_avg: Some(26364265520), is_estimate: true }
2026-07-29 Some("MSFT") Earnings { eps_estimate_low: Some(4.07), eps_estimate_avg: Some(4.23972), eps_estimate_high: Some(4.89), revenue_estimate_avg: Some(87672487440), is_estimate: true }
2026-07-30 Some("AAPL") Earnings { eps_estimate_low: Some(1.83), eps_estimate_avg: Some(1.89396), eps_estimate_high: Some(1.99), revenue_estimate_avg: Some(108881511420), is_estimate: true }
```

## Batch Response Utility Methods

All batch response types expose three convenience methods:

```rust capture-output covers=finance_query::tickers::core::BatchQuotesResponse
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;
    let response = tickers.quotes().await?;

    println!("Successful: {}", response.success_count());
    println!("Failed:     {}", response.error_count());

    if !response.all_successful() {
        for (symbol, error) in &response.errors {
            eprintln!("Failed to fetch {}: {}", symbol, error);
        }
    }
    Ok(())
}
```

```text soothfast-output
Successful: 2
Failed:     0
```

<!-- soothfast:bind finance_query::tickers::core::BatchQuotesResponse -->

| Method | Description |
|--------|-------------|
| `success_count()` | Number of successfully fetched items |
| `error_count()` | Number of failed symbols |
| `all_successful()` | `true` if no errors occurred |

<!-- /soothfast:bind -->

## Dynamic Symbol Management

Add or remove symbols from a `Tickers` instance after creation. This is useful for managing watchlists or portfolios dynamically.

```rust capture-output
use finance_query::Tickers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start with initial symbols
    let mut tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;
    println!("Initial symbols: {:?}", tickers.symbols());

    // Add more symbols
    tickers.add_symbols(["GOOGL", "TSLA", "NVDA"]);
    println!("After adding: {:?}", tickers.symbols());

    // Remove symbols (also clears their cached data)
    tickers.remove_symbols(["MSFT", "TSLA"]).await;
    println!("After removing: {:?}", tickers.symbols());

    // Fetch quotes for current symbols
    let response = tickers.quotes().await?;
    // Response will only include AAPL, GOOGL, NVDA
    Ok(())
}
```

```text soothfast-output
Initial symbols: ["AAPL", "MSFT"]
After adding: ["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA"]
After removing: ["AAPL", "GOOGL", "NVDA"]
```

!!! warning "Cache Clearing"
    When you remove symbols using `remove_symbols()`, all cached data for those symbols is also cleared.

## Individual Access

You can also access individual symbols from the `Tickers` instance. If the data is already cached from a batch operation, it returns immediately. If not, it triggers a batch fetch (for quotes) or single fetch (for charts).

```rust capture-output
use finance_query::format::Raw;
use finance_query::{Interval, Tickers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await?;

    // Get single quote (uses cache if available)
    let aapl = tickers.quote::<Raw>("AAPL").await?;

    // Get single chart (uses cache if available)
    let msft_chart = tickers
        .chart("MSFT", Interval::OneDay, TimeRange::OneMonth)
        .await?;

    println!("{}: price={:?}", aapl.symbol, aapl.regular_market_price);
    println!(
        "{}: candles={}",
        msft_chart.symbol,
        msft_chart.candles.len()
    );
    Ok(())
}
```

```text soothfast-output
AAPL: price=Some(333.74)
MSFT: candles=20
```

## Caching

`Tickers` caching is **on by default** and lasts as long as the handle lives. Use `.cache(Duration)` to bound how long a response is reused, or `.no_cache()` to fetch fresh on every call.

```rust capture-output
use finance_query::Tickers;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bound reuse to a 30-second TTL
    let tickers = Tickers::builder(vec!["AAPL", "MSFT"])
        .cache(Duration::from_secs(30))
        .build()
        .await?;

    // First call: Network request, result cached for 30s
    let response1 = tickers.quotes().await?;

    // Second call within TTL: Returns cached data (no network request)
    let response2 = tickers.quotes().await?;

    // Clear all caches to force fresh data
    tickers.clear_cache().await;
    let response3 = tickers.quotes().await?; // Network request

    // Or clear selectively:
    tickers.clear_quote_cache().await; // Quotes only
    tickers.clear_chart_cache().await; // Charts, sparks, and events

    println!(
        "response1={} response2={} response3={}",
        response1.quotes.len(),
        response2.quotes.len(),
        response3.quotes.len(),
    );
    Ok(())
}
```

```text soothfast-output
response1=2 response2=2 response3=2
```

| Method | Clears |
|--------|--------|
| `clear_cache()` | All cached data (quotes, charts, financials, news, etc.) |
| `clear_quote_cache()` | Quote data only |
| `clear_chart_cache()` | Charts, spark data, and events (dividends/splits/capital gains) |

## Best Practices

!!! tip "Optimize Batch Operations"
    - **Group symbols** - Use `Tickers` whenever you need data for multiple symbols (e.g., a portfolio or watchlist)
    - **Handle partial failures** - Always check the `errors` map in responses. One invalid symbol shouldn't fail the entire batch
    - **Reuse instances** - Keep the `Tickers` instance alive to benefit from caching across multiple operations

    ```rust no_run
    use finance_query::{Interval, Tickers, TimeRange};

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Good: Reuse Tickers instance for multiple operations
        let tickers = Tickers::builder(vec!["AAPL", "GOOGL", "INVALID", "MSFT"]).logo().build().await?;

        // First operation - fetches data
        let quotes_response = tickers.quotes().await?;

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
        Ok(())
    }
    ```

## Next Steps

- [Ticker API](ticker.md) - Detailed operations for single symbols (financials, options, news)
- [Backtesting](backtesting.md) - Portfolio backtesting across multiple symbols with `Tickers::backtest()`
- [DataFrame Support](dataframe.md) - Convert batch responses to Polars DataFrames for analysis
- [Configuration](configuration.md) - Customize regional settings and network options
