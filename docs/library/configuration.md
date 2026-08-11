# Configuration

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — TickerBuilder](https://docs.rs/finance-query/latest/finance_query/struct.TickerBuilder.html)

This guide explains how to configure `Ticker` and `Tickers` for different regions, languages, network settings, and more.

## Regional Settings

Yahoo Finance provides different data based on regional settings. Finance Query makes it easy to configure the correct language and region for your use case.

### Using Regions (Recommended)

The easiest way to set regional settings is using the `Region` enum, which automatically pairs the correct language and region codes:

```rust no_run covers=finance_query::constants::Region
use finance_query::{Ticker, Region};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // French stock with France locale
    let ticker = Ticker::builder("MC.PA")
        .region(Region::France)
        .build()
        .await?;

    // German stock with German locale
    let ticker = Ticker::builder("SAP.DE")
        .region(Region::Germany)
        .build()
        .await?;

    // UK stock with UK locale
    let ticker = Ticker::builder("HSBA.L")
        .region(Region::UnitedKingdom)
        .build()
        .await?;
    Ok(())
}
```

**Supported Regions:**

<!-- soothfast:bind finance_query::constants::Region -->

| Region | Language Code | Region Code |
|---------|---------------|-------------|
| `Argentina` | es-AR | AR |
| `Australia` | en-AU | AU |
| `Brazil` | pt-BR | BR |
| `Canada` | en-CA | CA |
| `China` | zh-CN | CN |
| `Denmark` | da-DK | DK |
| `Finland` | fi-FI | FI |
| `France` | fr-FR | FR |
| `Germany` | de-DE | DE |
| `Greece` | el-GR | GR |
| `HongKong` | zh-Hant-HK | HK |
| `India` | en-IN | IN |
| `Israel` | he-IL | IL |
| `Italy` | it-IT | IT |
| `Japan` | ja-JP | JP |
| `Korea` | ko-KR | KR |
| `Malaysia` | ms-MY | MY |
| `Mexico` | es-MX | MX |
| `NewZealand` | en-NZ | NZ |
| `Norway` | nb-NO | NO |
| `Portugal` | pt-PT | PT |
| `Qatar` | ar-QA | QA |
| `Russia` | ru-RU | RU |
| `Singapore` | en-SG | SG |
| `Spain` | es-ES | ES |
| `Sweden` | sv-SE | SE |
| `Taiwan` | zh-TW | TW |
| `Thailand` | th-TH | TH |
| `Turkey` | tr-TR | TR |
| `UnitedKingdom` | en-GB | GB |
| `UnitedStates` | en-US | US (default) |
| `Vietnam` | vi-VN | VN |

<!-- /soothfast:bind -->

### Manual Language and Region

For custom configurations, set language and region separately:

```rust no_run
use finance_query::Ticker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::builder("AAPL")
        .lang("en-US")
        .region_code("US")
        .build()
        .await?;
    Ok(())
}
```

**Important**: Language and region should match. Using mismatched pairs (e.g., `de-DE` with `US` region) may produce inconsistent results.

## Network Settings

### Timeout

Set HTTP request timeout (default: 30 seconds):

```rust no_run
use finance_query::Ticker;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::builder("AAPL")
        .timeout(Duration::from_secs(60))  // 60 second timeout
        .build()
        .await?;
    Ok(())
}
```

### Proxy

Configure an HTTP proxy:

```rust no_run
use finance_query::Ticker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::builder("AAPL")
        .proxy("http://proxy.example.com:8080")
        .build()
        .await?;

    // With authentication
    let ticker = Ticker::builder("AAPL")
        .proxy("http://user:pass@proxy.example.com:8080")
        .build()
        .await?;
    Ok(())
}
```

Supports:

- HTTP proxies: `http://proxy.example.com:8080`
- HTTPS proxies: `https://proxy.example.com:8080`
- SOCKS5 proxies: `socks5://proxy.example.com:1080`

## Batch Operations (`Tickers`)

Configure `Tickers` for batch operations:

```rust no_run
use finance_query::{Tickers, Region};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tickers = Tickers::builder(vec!["2330.TW", "2317.TW", "2454.TW"])
        .region(Region::Taiwan)
        .timeout(Duration::from_secs(60))
        .build()
        .await?;
    Ok(())
}
```

`Tickers` supports the same builder methods as `Ticker`:

- `.region(Region)`
- `.lang(String)`
- `.region_code(String)`
- `.timeout(Duration)`
- `.proxy(String)`

## Intervals and Time Ranges

### Chart Intervals

<!-- soothfast:bind finance_query::constants::Interval -->
When fetching chart data, choose an appropriate interval. This block runs as a
real test — every variant below exists on `Interval`:
<!-- /soothfast:bind -->

```rust capture-output covers=finance_query::constants::Interval
use finance_query::Interval;

// Intraday trading
let _ = Interval::OneMinute; // 1m candles
let _ = Interval::FiveMinutes; // 5m candles
let _ = Interval::FifteenMinutes; // 15m candles
let _ = Interval::ThirtyMinutes; // 30m candles
let _ = Interval::OneHour; // 1h candles

// Daily and longer
let _ = Interval::OneDay; // Daily candles (most common)
let _ = Interval::OneWeek; // Weekly candles
let _ = Interval::OneMonth; // Monthly candles
let _ = Interval::ThreeMonths; // Quarterly candles

println!("OneMinute.as_str() = {:?}", Interval::OneMinute.as_str());
println!("OneDay.as_str()    = {:?}", Interval::OneDay.as_str());
```

```text soothfast-output
OneMinute.as_str() = "1m"
OneDay.as_str()    = "1d"
```

### Time Ranges

<!-- soothfast:bind finance_query::constants::TimeRange -->
Time ranges span from a single day to the full available history — this block
also runs as a real test against the `TimeRange` enum:
<!-- /soothfast:bind -->

```rust capture-output covers=finance_query::constants::TimeRange
use finance_query::TimeRange;

// Short term
let _ = TimeRange::OneDay; // 1 day
let _ = TimeRange::FiveDays; // 5 days
let _ = TimeRange::OneMonth; // 1 month
let _ = TimeRange::ThreeMonths; // 3 months
let _ = TimeRange::SixMonths; // 6 months

// Long term
let _ = TimeRange::OneYear; // 1 year
let _ = TimeRange::TwoYears; // 2 years
let _ = TimeRange::FiveYears; // 5 years
let _ = TimeRange::TenYears; // 10 years
let _ = TimeRange::YearToDate; // From Jan 1 of current year
let _ = TimeRange::Max; // All available history

println!("OneMonth.as_str() = {:?}", TimeRange::OneMonth.as_str());
println!("Max.as_str()      = {:?}", TimeRange::Max.as_str());
```

```text soothfast-output
OneMonth.as_str() = "1mo"
Max.as_str()      = "max"
```

### Interval and Range Compatibility

Not all interval/range combinations are valid. Yahoo Finance enforces these restrictions:

| Interval | Valid Ranges |
|----------|--------------|
| 1m, 5m | 1d, 5d (max 7 days of intraday data) |
| 15m, 30m | 1d, 5d, 1mo (max ~60 days) |
| 1h | 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y (max ~2 years) |
| 1d, 1wk, 1mo, 3mo | All ranges |

**Example:**

```rust no_run
use finance_query::{Ticker, Interval, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    // Valid
    let daily = ticker.chart(Interval::OneDay, TimeRange::OneYear).await?;
    let intraday = ticker.chart(Interval::FiveMinutes, TimeRange::OneDay).await?;

    // Invalid - will return error
    // let invalid = ticker.chart(Interval::OneMinute, TimeRange::OneMonth).await?;
    Ok(())
}
```

## Financial Statement Frequencies

When fetching financial statements:

```rust no_run
use finance_query::{Frequency, StatementType, Ticker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    // Annual statements (default)
    let income_annual = ticker.financials(
        StatementType::Income,
        Frequency::Annual
    ).await?;

    // Quarterly statements
    let income_quarterly = ticker.financials(
        StatementType::Income,
        Frequency::Quarterly
    ).await?;
    Ok(())
}
```

## Value Formatting

`quote()` is generic over the output format, so you can choose the representation you want at call sites.

```rust no_run
use finance_query::Ticker;
use finance_query::format::{Both, Pretty, Raw};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = Ticker::new("AAPL").await?;

    let raw = ticker.quote::<Raw>().await?;       // numeric fields (Option<f64>, Option<i64>)
    let pretty = ticker.quote::<Pretty>().await?; // formatted strings (Option<String>)
    let both = ticker.quote::<Both>().await?;     // raw + formatted pair
    Ok(())
}
```

For quote sub-modules (like `financial_data()` or `key_stats()`), the return type is still the Both format, so use `.raw` to access the numeric values.

## Provider Configuration

Configure which data providers to use and how they're initialized.

### Provider API Keys

API keys for each provider are read from environment variables:

| Provider | Env var | Feature flag |
|----------|---------|-------------|
| Polygon.io | `POLYGON_API_KEY` | `polygon` |
| FMP | `FMP_API_KEY` | `fmp` |
| Alpha Vantage | `ALPHAVANTAGE_API_KEY` | `alphavantage` |
| FRED | `FRED_API_KEY` | `fred` |
| CoinGecko | *(keyless)* | `crypto` |
| Yahoo Finance | *(keyless, automatic)* | *(always available)* |

No manual init calls are needed — `TickerBuilder::build()` reads keys automatically.

```bash
export POLYGON_API_KEY="your-polygon-key"
export FMP_API_KEY="your-fmp-key"
```

### Provider Selection

```rust no_run feature=polygon
use finance_query::{Capability, Fetch, Provider, Providers, Ticker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default: Yahoo Finance only
    let ticker = Ticker::new("AAPL").await?;

    // Route specific capabilities to preferred providers (routing lives on Providers::builder)
    // (requires the `polygon` and `fmp` features)
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .route(Capability::FUNDAMENTALS, [Provider::Fmp, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let ticker = providers.ticker("AAPL").build().await?;
    Ok(())
}
```

See [Multi-Provider Architecture](providers/index.md) for the complete provider reference.

## Best Practices

!!! tip "Match Symbols to Regions"
    - **Use `Region` enum when possible** - Ensures correct lang/region pairing
    - **Match symbols to regions** - Use appropriate regional settings for each symbol:
        - US stocks (`AAPL`, `MSFT`): `Region::UnitedStates`
        - Taiwan stocks (`2330.TW`): `Region::Taiwan`
        - UK stocks (`HSBA.L`): `Region::UnitedKingdom`

    ```rust no_run
    use finance_query::{Region, Ticker, format::Raw};

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // US stock
        let apple = Ticker::builder("AAPL")
            .region(Region::UnitedStates)
            .logo()
            .build()
            .await?;

        // Taiwan stock
        let tsmc = Ticker::builder("2330.TW")
            .region(Region::Taiwan)
            .logo()
            .build()
            .await?;

        // German stock
        let sap = Ticker::builder("SAP.DE")
            .region(Region::Germany)
            .logo()
            .build()
            .await?;

        // Fetch quotes in parallel
        let (apple_quote, tsmc_quote, sap_quote) = tokio::join!(
            apple.quote::<Raw>(),
            tsmc.quote::<Raw>(),
            sap.quote::<Raw>()
        );
        println!("{:?} {:?} {:?}", apple_quote?.symbol, tsmc_quote?.symbol, sap_quote?.symbol);
        Ok(())
    }
    ```

!!! tip "Configure Timeouts and Proxies"
    - **Set reasonable timeouts** - Default is 30s, increase for slow connections
    - **Share configuration** - Create one config and reuse it across tickers
    - **Choose appropriate intervals**:
        - Intraday analysis: 1m, 5m, 15m
        - Daily charts: 1d
        - Long-term trends: 1wk, 1mo

    ```rust no_run
    use finance_query::Ticker;
    use std::time::Duration;

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Configure for corporate network with proxy and longer timeout
        let ticker = Ticker::builder("AAPL")
            .proxy("http://corporate-proxy.company.com:8080")
            .timeout(Duration::from_secs(45))
            .build()
            .await?;
        Ok(())
    }
    ```

## Next Steps

- [Getting Started](getting-started.md) - Feature flags and initial setup
- [Ticker API](ticker.md) - Full single-symbol API reference
- [Batch Tickers](tickers.md) - Batch operations with shared configuration
