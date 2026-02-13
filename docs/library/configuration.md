# Configuration

This guide explains how to configure `Ticker` and `Tickers` for different regions, languages, network settings, and more.

## Regional Settings

Yahoo Finance provides different data based on regional settings. Finance Query makes it easy to configure the correct language and region for your use case.

### Using Regions (Recommended)

The easiest way to set regional settings is using the `Region` enum, which automatically pairs the correct language and region codes:

```rust
use finance_query::{Ticker, Region};

// Taiwan stock with France locale
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
```

**Supported Regions:**

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
| `Malaysia` | ms-MY | MY |
| `NewZealand` | en-NZ | NZ |
| `Norway` | nb-NO | NO |
| `Portugal` | pt-PT | PT |
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

### Manual Language and Region

For custom configurations, set language and region separately:

```rust
let ticker = Ticker::builder("AAPL")
    .lang("en-US")
    .region_code("US")
    .build()
    .await?;
```

**Important**: Language and region should match. Using mismatched pairs (e.g., `de-DE` with `US` region) may produce inconsistent results.

## Network Settings

### Timeout

Set HTTP request timeout (default: 30 seconds):

```rust
use std::time::Duration;

let ticker = Ticker::builder("AAPL")
    .timeout(Duration::from_secs(60))  // 60 second timeout
    .build()
    .await?;
```

### Proxy

Configure an HTTP proxy:

```rust
let ticker = Ticker::builder("AAPL")
    .proxy("http://proxy.example.com:8080")
    .build()
    .await?;

// With authentication
let ticker = Ticker::builder("AAPL")
    .proxy("http://user:pass@proxy.example.com:8080")
    .build()
    .await?;
```

Supports:

- HTTP proxies: `http://proxy.example.com:8080`
- HTTPS proxies: `https://proxy.example.com:8080`
- SOCKS5 proxies: `socks5://proxy.example.com:1080`

## Advanced Configuration

### Custom Client Config

For complete control, create a `ClientConfig`:

```rust
use finance_query::{Ticker, ClientConfig};
use std::time::Duration;

let config = ClientConfig {
    lang: "zh-TW".to_string(),
    region: "TW".to_string(),
    timeout: Duration::from_secs(45),
    proxy: Some("http://proxy.example.com:8080".to_string()),
};

let ticker = Ticker::builder("2330.TW")
    .config(config)
    .build()
    .await?;
```

!!! warning
    Using `.config()` overrides any previously set individual fields (`.lang()`, `.timeout()`, etc.).

### Reusing Configuration

Share configuration across multiple tickers:

```rust
use finance_query::{ClientConfig, Region};
use std::time::Duration;

// Create shared config for France stocks
let config = ClientConfig {
    lang: Region::France.lang().to_string(),
    region: Region::France.region().to_string(),
    timeout: Duration::from_secs(30),
    proxy: None,
};

// Use for multiple tickers
let lvmh = Ticker::builder("MC.PA").config(config.clone()).build().await?;
let loreal = Ticker::builder("OR.PA").config(config.clone()).build().await?;
let total = Ticker::builder("TTE.PA").config(config).build().await?;
```

## Batch Operations (`Tickers`)

Configure `Tickers` for batch operations:

```rust
use finance_query::{Tickers, Region};
use std::time::Duration;

let tickers = Tickers::builder(vec!["2330.TW", "2317.TW", "2454.TW"])
    .region(Region::Taiwan)
    .timeout(Duration::from_secs(60))
    .build()
    .await?;
```

`Tickers` supports the same builder methods as `Ticker`:

- `.region(Region)`
- `.lang(String)`
- `.region_code(String)`
- `.timeout(Duration)`
- `.proxy(String)`
- `.config(ClientConfig)`

## Intervals and Time Ranges

### Chart Intervals

When fetching chart data, choose an appropriate interval:

```rust
use finance_query::Interval;

// Intraday trading
Interval::OneMinute      // 1m candles
Interval::FiveMinutes    // 5m candles
Interval::FifteenMinutes // 15m candles
Interval::ThirtyMinutes  // 30m candles
Interval::OneHour        // 1h candles

// Daily and longer
Interval::OneDay         // Daily candles (most common)
Interval::OneWeek        // Weekly candles
Interval::OneMonth       // Monthly candles
Interval::ThreeMonths    // Quarterly candles
```

### Time Ranges

```rust
use finance_query::TimeRange;

// Short term
TimeRange::OneDay        // 1 day
TimeRange::FiveDays      // 5 days
TimeRange::OneMonth      // 1 month
TimeRange::ThreeMonths   // 3 months
TimeRange::SixMonths     // 6 months

// Long term
TimeRange::OneYear       // 1 year
TimeRange::TwoYears      // 2 years
TimeRange::FiveYears     // 5 years
TimeRange::TenYears      // 10 years
TimeRange::YearToDate    // From Jan 1 of current year
TimeRange::Max           // All available history
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

```rust
use finance_query::{Ticker, Interval, TimeRange};

let ticker = Ticker::new("AAPL").await?;

// Valid
let daily = ticker.chart(Interval::OneDay, TimeRange::OneYear).await?;
let intraday = ticker.chart(Interval::FiveMinutes, TimeRange::OneDay).await?;

// Invalid - will return error
// let invalid = ticker.chart(Interval::OneMinute, TimeRange::OneMonth).await?;
```

## Financial Statement Frequencies

When fetching financial statements:

```rust
use finance_query::{StatementType, Frequency};

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
```

## Value Formatting

Some server endpoints support value formatting. This is primarily a server feature, but the library defines the `ValueFormat` enum:

```rust
use finance_query::ValueFormat;

// For display purposes
ValueFormat::Raw     // Raw numbers (default)
ValueFormat::Pretty  // Formatted strings (e.g., "1.2M", "$45.67")
ValueFormat::Both    // Both raw and pretty values
```

**Note**: This is mainly used by the server's REST API. Library users typically work with raw values directly.

## Best Practices

!!! tip "Match Symbols to Regions"
    - **Use `Region` enum when possible** - Ensures correct lang/region pairing
    - **Match symbols to regions** - Use appropriate regional settings for each symbol:
        - US stocks (`AAPL`, `MSFT`): `Region::UnitedStates`
        - Taiwan stocks (`2330.TW`): `Region::Taiwan`
        - UK stocks (`HSBA.L`): `Region::UnitedKingdom`

    ```rust
    use finance_query::{Ticker, Region};

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
        apple.quote(),
        tsmc.quote(),
        sap.quote()
    );
    ```

!!! tip "Configure Timeouts and Proxies"
    - **Set reasonable timeouts** - Default is 30s, increase for slow connections
    - **Share configuration** - Create one config and reuse it across tickers
    - **Choose appropriate intervals**:
        - Intraday analysis: 1m, 5m, 15m
        - Daily charts: 1d
        - Long-term trends: 1wk, 1mo

    ```rust
    use finance_query::Ticker;
    use std::time::Duration;

    // Configure for corporate network with proxy and longer timeout
    let ticker = Ticker::builder("AAPL")
        .proxy("http://corporate-proxy.company.com:8080")
        .timeout(Duration::from_secs(45))
        .build()
        .await?;
    ```

## Next Steps

- [Ticker API](ticker.md) - Full API reference
- [Models](models.md) - Understanding response types
