# Ticker API Reference

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Ticker](https://docs.rs/finance-query/latest/finance_query/struct.Ticker.html)

The `Ticker` struct is the primary interface for fetching financial data for a single symbol. It provides lazy-loaded, cached access to quotes, charts, financials, and more from your configured data providers (Yahoo Finance by default).

!!! tip "Multiple Symbols"
    Need to fetch data for multiple symbols? Use the [`Tickers`](tickers.md) struct for efficient batch operations.

## Creation

### Simple Construction

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;
    println!("{}", ticker.symbol());
    Ok(())
}
```

```text soothfast-output
AAPL
```

### Builder Pattern

For advanced configuration, use the builder:

```rust no_run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Ticker, Region};
    use std::time::Duration;

    // Using region enum (recommended - sets lang and region code correctly)
    let ticker = Ticker::builder("2330.TW")
        .region(Region::Taiwan)
        .timeout(Duration::from_secs(30))
        .build()
        .await?;

    // With logo fetching and in-memory cache (TTL: 5 minutes)
    let ticker = Ticker::builder("AAPL")
        .logo()
        .cache(Duration::from_secs(300))
        .build()
        .await?;

    // Manual language/region configuration
    let ticker = Ticker::builder("AAPL")
        .lang("en-US")
        .region_code("US")
        .timeout(Duration::from_secs(20))
        .proxy("http://proxy.example.com:8080")
        .build()
        .await?;
    Ok(())
}
```

**Builder Methods:**

- `.region(Region)` - Set region (automatically configures lang and region_code)
- `.lang(String)` - Set language code (e.g., "en-US", "de-DE", "zh-TW")
- `.region_code(String)` - Set region code (e.g., "US", "JP")
- `.timeout(Duration)` - Set HTTP request timeout
- `.proxy(String)` - Set proxy URL
- `.logo()` - Fetch company logo URLs alongside quote data
- `.cache(Duration)` - Bound in-memory caching to the given TTL (default: cached for the handle's lifetime)
- `.no_cache()` - Disable caching — every call fetches fresh data

See [Configuration](configuration.md) for details on available regions and settings.
See [Multi-Provider Architecture](providers/index.md) for provider configuration.

### Multi-Provider Configuration

Multi-provider routing (Polygon, FMP, Alpha Vantage, etc.) is configured through
[`ProvidersBuilder`](providers/index.md), not `TickerBuilder`. Pass the configured
`Providers` to `providers.ticker()`:

```rust no_run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Capability, Fetch, Provider, Providers};

    // Route quote to Polygon (fallback Yahoo), fundamentals to FMP (fallback Yahoo)
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .route(Capability::FUNDAMENTALS, [Provider::Fmp, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;

    // All tickers from this Providers share the same connections
    let aapl = providers.ticker("AAPL").build().await?;
    let msft = providers.ticker("MSFT").build().await?;
    Ok(())
}
```

For simple Yahoo-only usage, `Ticker::new` / `Ticker::builder` work unchanged —
no `Providers` setup needed.

## Quote Data

### Aggregated Quote

Get a comprehensive quote with all key metrics:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;
    use finance_query::format::Raw;

    // Enable logo fetching via builder
    let ticker = Ticker::builder("AAPL").logo().build().await?;
    let quote = ticker.quote::<Raw>().await?;

    println!("Symbol: {}", quote.symbol);
    println!("Name: {}", quote.short_name.as_deref().unwrap_or("N/A"));
    let price = quote.regular_market_price.unwrap_or(0.0);
    println!("Price: ${:.2}", price);
    let change = quote.regular_market_change.unwrap_or(0.0);
    let change_pct = quote.regular_market_change_percent.unwrap_or(0.0);
    println!("Change: {:+.2} ({:+.2}%)", change, change_pct);
    let market_cap = quote.market_cap.unwrap_or(0);
    println!("Market Cap: ${}", market_cap);
    // Logo URLs (only populated when .logo() is used on the builder)
    println!("Logo: {:?}", quote.logo_url);
    println!("Company Logo: {:?}", quote.company_logo_url);
    Ok(())
}
```

```text soothfast-output
Symbol: AAPL
Name: Apple Inc.
Price: $333.74
Change: +0.48 (+0.00%)
Market Cap: $4901757779968
Logo: Some("https://s.yimg.com/lb/brands/50x50_apple.png")
Company Logo: Some("https://s.yimg.com/lb/brands/50x50_apple.png")
```

The `Quote` struct aggregates data from multiple `quote modules` into a single structure.

### Value Formats

`quote()` is generic over the output format:

- `Raw` returns numeric fields as plain numbers (e.g., `Option<f64>`).
- `Pretty` returns formatted strings (e.g., `Option<String>` like `"1.2M"`).
- `Both` returns the raw + formatted pair (the default for sub-module accessors like `financial_data()` or `key_stats()`).

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;
    use finance_query::format::{Both, Pretty, Raw};

    let ticker = Ticker::new("AAPL").await?;
    let raw = ticker.quote::<Raw>().await?; // numeric fields
    let pretty = ticker.quote::<Pretty>().await?; // formatted strings
    let both = ticker.quote::<Both>().await?; // raw + formatted

    println!("Raw:    {:?}", raw.market_cap);
    println!("Pretty: {:?}", pretty.market_cap);
    println!(
        "Both:   raw={:?} fmt={:?}",
        both.market_cap.as_ref().and_then(|v| v.raw),
        both.market_cap.as_ref().and_then(|v| v.fmt.clone())
    );
    Ok(())
}
```

```text soothfast-output
Raw:    Some(4901757779968)
Pretty: Some("4.90T")
Both:   raw=Some(4901757779968) fmt=Some("4.90T")
```

### Quote Modules

Access specific quote modules directly. All modules are fetched together on first access and cached:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;

    // First access triggers ONE API call for ALL modules
    let price = ticker.price().await?;
    if let Some(p) = price {
        println!(
            "Market State: {}",
            p.market_state.as_deref().unwrap_or("N/A")
        );
        println!("Currency: {}", p.currency.as_deref().unwrap_or("N/A"));
    }

    // Subsequent calls use cached data (no network request)
    let financial_data = ticker.financial_data().await?;
    if let Some(fd) = financial_data {
        let revenue = fd.total_revenue.as_ref().and_then(|v| v.raw).unwrap_or(0);
        println!("Revenue: ${}", revenue);
        let profit_margins = fd
            .profit_margins
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("Profit Margin: {:.2}%", profit_margins * 100.0);
    }

    // Get EPS from DefaultKeyStatistics (not FinancialData)
    if let Some(stats) = ticker.key_stats().await? {
        let eps = stats
            .trailing_eps
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("EPS: ${:.2}", eps);
    }

    let profile = ticker.asset_profile().await?;
    if let Some(prof) = profile {
        println!("Sector: {}", prof.sector.as_deref().unwrap_or("N/A"));
        println!("Industry: {}", prof.industry.as_deref().unwrap_or("N/A"));
        println!("Website: {}", prof.website.as_deref().unwrap_or("N/A"));
        println!(
            "Description: {}",
            prof.long_business_summary.as_deref().unwrap_or("N/A")
        );
    }
    Ok(())
}
```

```text soothfast-output
Market State: CLOSED
Currency: USD
Revenue: $451442016256
Profit Margin: 27.15%
EPS: $8.24
Sector: Technology
Industry: Consumer Electronics
Website: https://www.apple.com
Description: Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide. The company offers iPhone, a line of smartphones; Mac, a line of personal computers; iPad, a line of multi-purpose tablets; and wearables, home, and accessories comprising AirPods, Apple Vision Pro, Apple TV, Apple Watch, Beats products, and HomePod, as well as Apple branded and third-party accessories. It also provides AppleCare support and cloud services; and operates various platforms, including the App Store that allows customers to discover and download applications and digital content, such as books, music, video, games, and podcasts, as well as advertising services include third-party licensing arrangements and its own advertising platforms. In addition, the company offers various subscription-based services, such as Apple Arcade, a game subscription service; Apple Fitness+, a personalized fitness service; Apple Music, which offers users a curated listening experience with on-demand radio stations; Apple News+, a subscription news and magazine service; Apple TV, which offers original content and live sports; Apple Card, a co-branded credit card; and Apple Pay, a cashless payment service, as well as licenses its intellectual property. The company serves consumers, and small and mid-sized businesses; and the education, enterprise, and government markets. It distributes third-party applications for its products through the App Store. The company also sells its products through its retail and online stores, and direct sales force; and third-party cellular network carriers and resellers. The company was formerly known as Apple Computer, Inc. and changed its name to Apple Inc. in January 2007. Apple Inc. was founded in 1976 and is headquartered in Cupertino, California.
```

**Available Quote Modules:**

| Method | Returns | Description |
|--------|---------|-------------|
| `.price()` | `Price` | Current price, market state, currency |
| `.summary_detail()` | `SummaryDetail` | Market cap, P/E, dividend, 52-week range |
| `.financial_data()` | `FinancialData` | Revenue, margins, EPS, cash flow |
| `.key_stats()` | `DefaultKeyStatistics` | Extended statistics (beta, shares outstanding, etc.) |
| `.asset_profile()` | `AssetProfile` | Company info (sector, industry, description, officers) |
| `.calendar_events()` | `CalendarEvents` | Upcoming earnings, dividends, splits |
| `.earnings()` | `Earnings` | Historical and forecasted earnings |
| `.earnings_trend()` | `EarningsTrend` | Analyst earnings estimates and trends |
| `.earnings_history()` | `EarningsHistory` | Past earnings surprises |
| `.recommendation_trend()` | `RecommendationTrend` | Analyst buy/sell/hold recommendations |
| `.insider_holders()` | `InsiderHolders` | Insider ownership |
| `.insider_transactions()` | `InsiderTransactions` | Recent insider trading activity |
| `.institution_ownership()` | `InstitutionOwnership` | Institutional holders |
| `.fund_ownership()` | `FundOwnership` | Mutual fund holders |
| `.major_holders()` | `MajorHoldersBreakdown` | Ownership percentages |
| `.share_purchase_activity()` | `NetSharePurchaseActivity` | Insider net purchase activity |
| `.quote_type()` | `QuoteTypeData` | Asset type, exchange, timezone |
| `.summary_profile()` | `SummaryProfile` | Company summary (address, employees, etc.) |
| `.sec_filings()` | `SecFilings` | Recent SEC filings |
| `.grading_history()` | `UpgradeDowngradeHistory` | Analyst upgrade/downgrade history |

All methods return `Result<Option<T>>` - the `Option` is `None` if the module is not available for this symbol (e.g., crypto doesn't have SEC filings).

### Example: Company Analysis

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("MSFT").await?;

    // Get financial health
    if let Some(fd) = ticker.financial_data().await? {
        println!("Financials:");
        let revenue = fd.total_revenue.as_ref().and_then(|v| v.raw).unwrap_or(0) as f64;
        println!("  Revenue: ${:.2}B", revenue / 1e9);
        let profit_margins = fd
            .profit_margins
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("  Profit Margin: {:.2}%", profit_margins * 100.0);
        let roe = fd
            .return_on_equity
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("  ROE: {:.2}%", roe * 100.0);
        let dte = fd
            .debt_to_equity
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("  Debt to Equity: {:.2}", dte);
    }

    // Get valuation
    if let Some(sd) = ticker.summary_detail().await? {
        println!("\nValuation:");
        let trailing_pe = sd.trailing_pe.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
        println!("  P/E Ratio: {:.2}", trailing_pe);
        let forward_pe = sd.forward_pe.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
        println!("  Forward P/E: {:.2}", forward_pe);
    }

    // Get analyst sentiment
    if let Some(rt) = ticker.recommendation_trend().await?
        && let Some(latest) = rt.trend.first()
    {
        println!("\nAnalyst Recommendations:");
        println!("  Strong Buy: {}", latest.strong_buy.unwrap_or(0));
        println!("  Buy: {}", latest.buy.unwrap_or(0));
        println!("  Hold: {}", latest.hold.unwrap_or(0));
        println!("  Sell: {}", latest.sell.unwrap_or(0));
        println!("  Strong Sell: {}", latest.strong_sell.unwrap_or(0));
    }
    Ok(())
}
```

```text soothfast-output
Financials:
  Revenue: $318.27B
  Profit Margin: 39.34%
  ROE: 34.01%
  Debt to Equity: 30.27

Valuation:
  P/E Ratio: 0.00
  Forward P/E: 0.00

Analyst Recommendations:
  Strong Buy: 12
  Buy: 42
  Hold: 3
  Sell: 0
  Strong Sell: 0
```

## Historical Data

### Chart (OHLCV) Data

Get historical candlestick data:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;

    // Daily candles for the past month
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    println!("Symbol: {}", chart.symbol);
    println!(
        "Currency: {}",
        chart.meta.currency.as_deref().unwrap_or("N/A")
    );
    println!(
        "Exchange: {}",
        chart.meta.exchange_name.as_deref().unwrap_or("N/A")
    );
    println!(
        "Timezone: {}",
        chart.meta.timezone.as_deref().unwrap_or("N/A")
    );

    for candle in &chart.candles {
        println!(
            "{}: O=${:.2}, H=${:.2}, L=${:.2}, C=${:.2}, V={}",
            candle.timestamp, candle.open, candle.high, candle.low, candle.close, candle.volume
        );
    }
    Ok(())
}
```

```text soothfast-output
Symbol: AAPL
Currency: USD
Exchange: NMS
Timezone: EDT
1781789400: O=$298.11, H=$300.57, L=$295.62, C=$298.01, V=85962200
1782135000: O=$297.31, H=$302.42, L=$296.76, C=$297.01, V=44879900
1782221400: O=$297.54, H=$301.64, L=$294.18, C=$294.30, V=52010900
1782307800: O=$295.36, H=$299.70, L=$292.94, C=$293.08, V=53081900
1782394200: O=$287.40, H=$288.80, L=$273.75, C=$275.15, V=107013700
1782480600: O=$275.00, H=$285.95, L=$274.21, C=$283.78, V=261775500
1782739800: O=$286.73, H=$288.37, L=$279.85, C=$281.74, V=66427000
1782826200: O=$281.17, H=$289.94, L=$280.70, C=$289.36, V=65100200
1782912600: O=$293.44, H=$296.59, L=$289.20, C=$294.38, V=50164200
1782999000: O=$294.12, H=$309.42, L=$293.68, C=$308.63, V=75352800
1783344600: O=$307.36, H=$314.20, L=$307.00, C=$312.66, V=53590000
1783431000: O=$315.29, H=$315.48, L=$310.15, C=$310.66, V=42490000
1783517400: O=$311.91, H=$314.82, L=$307.05, C=$313.39, V=41323500
1783603800: O=$310.51, H=$316.53, L=$308.16, C=$316.22, V=48124500
1783690200: O=$314.72, H=$316.91, L=$312.17, C=$315.32, V=34132300
1783949400: O=$317.02, H=$323.45, L=$315.78, C=$317.31, V=43257800
1784035800: O=$313.76, H=$316.19, L=$311.91, C=$314.86, V=36336800
1784122200: O=$317.62, H=$328.73, L=$317.32, C=$327.50, V=60957600
1784208600: O=$328.01, H=$334.68, L=$326.79, C=$333.26, V=62970600
1784295000: O=$331.98, H=$334.99, L=$329.00, C=$333.74, V=63365300
```

**Available Intervals:**

- Intraday: `OneMinute`, `FiveMinutes`, `FifteenMinutes`, `ThirtyMinutes`, `OneHour`
- Daily and above: `OneDay`, `OneWeek`, `OneMonth`, `ThreeMonths`

**Available Time Ranges:**

- `OneDay`, `FiveDays`, `OneMonth`, `ThreeMonths`, `SixMonths`
- `OneYear`, `TwoYears`, `FiveYears`, `TenYears`
- `YearToDate`, `Max`

**Chart Structure:**
```rust ignore
pub struct Chart {
    pub symbol: String,         // Stock symbol
    pub meta: ChartMeta,        // Metadata (exchange, currency, timezone, etc.)
    pub candles: Vec<Candle>,   // OHLCV candles
    pub interval: Option<Interval>,
    pub range: Option<TimeRange>,
}

pub struct Candle {
    pub timestamp: i64,         // Unix timestamp (seconds)
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,            // Signed integer
    pub adj_close: Option<f64>, // Adjusted close (accounts for splits/dividends)
}
```

### Corporate Events

#### Dividends

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let dividends = ticker.dividends(TimeRange::TwoYears).await?;

    for div in &dividends {
        // div.timestamp is a Unix timestamp (i64, seconds since epoch)
        println!("timestamp={}, amount=${:.4}", div.timestamp, div.amount);
    }
    Ok(())
}
```

```text soothfast-output
timestamp=1723469400, amount=$0.2500
timestamp=1731076200, amount=$0.2500
timestamp=1739197800, amount=$0.2500
timestamp=1747056600, amount=$0.2600
timestamp=1754919000, amount=$0.2600
timestamp=1762785000, amount=$0.2600
timestamp=1770647400, amount=$0.2600
timestamp=1778506200, amount=$0.2700
```

#### Stock Splits

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let splits = ticker.splits(TimeRange::Max).await?;

    for split in &splits {
        // ratio is a human-readable string like "4:1"
        println!(
            "timestamp={}, ratio={} ({}/{})",
            split.timestamp, split.ratio, split.numerator, split.denominator
        );
    }
    Ok(())
}
```

```text soothfast-output
timestamp=550848600, ratio=2:1 (2/1)
timestamp=961594200, ratio=2:1 (2/1)
timestamp=1109601000, ratio=2:1 (2/1)
timestamp=1402320600, ratio=7:1 (7/1)
timestamp=1598880600, ratio=4:1 (4/1)
```

#### Capital Gains

Distributions of capital gains (common for ETFs and mutual funds):

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Ticker, TimeRange};

    // SPY is a fund, so unlike a single stock it regularly distributes capital gains
    let ticker = Ticker::new("SPY").await?;
    let gains = ticker.capital_gains(TimeRange::FiveYears).await?;

    for gain in &gains {
        println!(
            "timestamp={}, amount=${:.4} per share",
            gain.timestamp, gain.amount
        );
    }
    Ok(())
}
```

```text soothfast-output
```

#### Dividend Analytics

Compute analytics from the dividend history (pure calculation, no extra network request):

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let dividends = ticker.dividends(TimeRange::FiveYears).await?;
    let analytics = finance_query::DividendAnalytics::from_dividends(&dividends);

    println!("Total paid:      ${:.2}", analytics.total_paid);
    println!("Payments:        {}", analytics.payment_count);
    println!("Average payment: ${:.4}", analytics.average_payment);

    if let Some(cagr) = analytics.cagr {
        println!("CAGR:            {:.1}%", cagr * 100.0);
    }
    if let Some(last) = &analytics.last_payment {
        println!(
            "Most recent:     ${:.4} at timestamp {}",
            last.amount, last.timestamp
        );
    }
    Ok(())
}
```

```text soothfast-output
Total paid:      $4.85
Payments:        20
Average payment: $0.2425
CAGR:            4.4%
Most recent:     $0.2700 at timestamp 1778506200
```

**`DividendAnalytics` fields:**

| Field | Type | Description |
|-------|------|-------------|
| `total_paid` | `f64` | Total dividends paid in the requested range |
| `payment_count` | `usize` | Number of dividend payments |
| `average_payment` | `f64` | Average dividend per payment |
| `cagr` | `Option<f64>` | Compound Annual Growth Rate of the dividend; `None` if fewer than 2 payments spanning a full year |
| `last_payment` | `Option<Dividend>` | Most recent dividend |
| `first_payment` | `Option<Dividend>` | Earliest dividend in the range |

### Technical Indicators

Calculate technical indicators with three approaches:

#### 1. Summary API

Get all pre-calculated indicators at once:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    // Simple indicators (Option<f64>)
    if let Some(rsi) = indicators.rsi_14 {
        println!("RSI(14): {:.2}", rsi);
        if rsi < 30.0 {
            println!("  -> Oversold");
        } else if rsi > 70.0 {
            println!("  -> Overbought");
        }
    }

    if let Some(sma) = indicators.sma_200 {
        println!("SMA(200): {:.2}", sma);
    }

    // Compound indicators (Option<Struct>)
    if let Some(macd) = &indicators.macd
        && let (Some(line), Some(signal)) = (macd.macd, macd.signal)
    {
        println!("MACD: {:.4} | Signal: {:.4}", line, signal);
        if line > signal {
            println!("  -> Bullish");
        }
    }

    if let Some(bb) = &indicators.bollinger_bands
        && let (Some(upper), Some(lower)) = (bb.upper, bb.lower)
    {
        println!("Bollinger: Upper={:.2}, Lower={:.2}", upper, lower);
    }
    Ok(())
}
```

```text soothfast-output
RSI(14): 80.78
  -> Overbought
MACD: 8.9736 | Signal: 5.7113
  -> Bullish
Bollinger: Upper=338.35, Lower=272.69
```

#### 2. Chart Extension Methods

Calculate specific indicators with any period:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    // Custom periods
    let sma_15 = chart.sma(15);
    let rsi_21 = chart.rsi(21)?;
    let macd = chart.macd(8, 21, 5)?; // Fast, slow, signal

    // Access latest value
    if let Some(&latest_sma) = sma_15.last().and_then(|v| v.as_ref()) {
        println!("SMA(15): {:.2}", latest_sma);
    }
    if let Some(&latest_rsi) = rsi_21.last().and_then(|v| v.as_ref()) {
        println!("RSI(21): {:.2}", latest_rsi);
    }
    if let Some(&latest_macd) = macd.macd_line.last().and_then(|v| v.as_ref()) {
        println!("MACD: {:.4}", latest_macd);
    }
    Ok(())
}
```

```text soothfast-output
SMA(15): 209.72
RSI(21): 74.80
MACD: 8.9736
```

#### 3. Direct Functions

Use indicator functions directly with custom data:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::indicators::{rsi, sma};
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let closes: Vec<f64> = chart.candles.iter().map(|c| c.close).collect();

    let sma_25 = sma(&closes, 25);
    let rsi_10 = rsi(&closes, 10)?;

    if let Some(&latest) = sma_25.last().and_then(|v| v.as_ref()) {
        println!("SMA(25): {:.2}", latest);
    }
    if let Some(&latest) = rsi_10.last().and_then(|v| v.as_ref()) {
        println!("RSI(10): {:.2}", latest);
    }
    Ok(())
}
```

```text soothfast-output
SMA(25): 209.72
RSI(10): 81.96
```

!!! tip "See Also"
    For complete indicator documentation including all available indicators, see [Indicators](indicators.md).

### Candlestick Patterns

Detect candlestick patterns from chart data (requires `indicators` feature):

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::indicators::PatternSentiment;
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    // Returns Vec<Option<CandlePattern>>, 1:1 aligned with chart.candles
    let signals = chart.patterns();

    // Zip patterns with candles for context
    for (candle, pattern) in chart.candles.iter().zip(signals.iter()) {
        if let Some(p) = pattern {
            println!(
                "timestamp={}: {:?} ({:?})",
                candle.timestamp,
                p,
                p.sentiment()
            );
        }
    }

    // Count bullish signals in the period
    let bullish_count = signals
        .iter()
        .filter(|s| {
            s.map(|p| p.sentiment() == PatternSentiment::Bullish)
                .unwrap_or(false)
        })
        .count();
    println!("{bullish_count} bullish patterns detected");
    Ok(())
}
```

```text soothfast-output
timestamp=1777555800: SpinningTop (Neutral)
timestamp=1777642200: ShootingStar (Bearish)
timestamp=1778506200: BearishHarami (Bearish)
timestamp=1778679000: ThreeWhiteSoldiers (Bullish)
timestamp=1778851800: BullishEngulfing (Bullish)
timestamp=1779111000: BearishEngulfing (Bearish)
timestamp=1779370200: ThreeWhiteSoldiers (Bullish)
timestamp=1779888600: BullishEngulfing (Bullish)
timestamp=1780061400: BearishHarami (Bearish)
timestamp=1780493400: BearishHarami (Bearish)
timestamp=1780579800: BullishHarami (Bullish)
timestamp=1780925400: ThreeBlackCrows (Bearish)
timestamp=1781098200: BullishHarami (Bullish)
timestamp=1781271000: TweezerTop (Bearish)
timestamp=1781703000: DarkCloudCover (Bearish)
timestamp=1781789400: BullishHarami (Bullish)
timestamp=1782480600: PiercingLine (Bullish)
timestamp=1782826200: BullishEngulfing (Bullish)
timestamp=1782912600: SpinningTop (Neutral)
timestamp=1782999000: BullishMarubozu (Bullish)
timestamp=1783344600: ThreeWhiteSoldiers (Bullish)
timestamp=1783517400: BullishHarami (Bullish)
timestamp=1783690200: BearishHarami (Bearish)
timestamp=1783949400: Doji (Neutral)
timestamp=1784035800: SpinningTop (Neutral)
12 bullish patterns detected
```

**Available Patterns (20 total):**

| Category | Pattern | Signal |
|----------|---------|--------|
| Three-bar | `MorningStar` | Bullish reversal |
| Three-bar | `EveningStar` | Bearish reversal |
| Three-bar | `ThreeWhiteSoldiers` | Bullish continuation |
| Three-bar | `ThreeBlackCrows` | Bearish continuation |
| Two-bar | `BullishEngulfing` | Bullish reversal |
| Two-bar | `BearishEngulfing` | Bearish reversal |
| Two-bar | `BullishHarami` | Bullish reversal |
| Two-bar | `BearishHarami` | Bearish reversal |
| Two-bar | `PiercingLine` | Bullish reversal |
| Two-bar | `DarkCloudCover` | Bearish reversal |
| Two-bar | `TweezerBottom` | Bullish reversal at support |
| Two-bar | `TweezerTop` | Bearish reversal at resistance |
| One-bar | `Hammer` | Bullish reversal (downtrend) |
| One-bar | `InvertedHammer` | Bullish reversal (downtrend) |
| One-bar | `HangingMan` | Bearish reversal (uptrend) |
| One-bar | `ShootingStar` | Bearish reversal (uptrend) |
| One-bar | `BullishMarubozu` | Bullish strength |
| One-bar | `BearishMarubozu` | Bearish strength |
| One-bar | `Doji` | Indecision |
| One-bar | `SpinningTop` | Indecision |

**Pattern priority:** Three-bar → two-bar → one-bar. Each candle slot holds at most one pattern. Output is always the same length as `chart.candles`.

!!! tip "Combine with indicators"
    Patterns are most useful as filters on top of indicators. For example, `RSI < 30` combined with a `Hammer` or `BullishEngulfing` gives a stronger entry signal than either alone.

## Risk Analytics

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["risk"] }
    ```

Compute a comprehensive risk summary from historical price data:

```rust capture-output feature=risk
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;

    // With S&P 500 as benchmark (enables beta calculation)
    let risk = ticker
        .risk(Interval::OneDay, TimeRange::OneYear, Some("^GSPC"))
        .await?;
    if let Some(beta) = risk.beta {
        println!("Beta (vs ^GSPC): {:.2}", beta);
    }

    // Without a benchmark
    let risk = ticker
        .risk(Interval::OneDay, TimeRange::OneYear, None)
        .await?;

    println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
    println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
    if let Some(sharpe) = risk.sharpe {
        println!("Sharpe:       {:.2}", sharpe);
    }
    if let Some(beta) = risk.beta {
        println!("Beta:         {:.2}", beta);
    }
    Ok(())
}
```

```text soothfast-output
VaR 95%:      1.97%
Max Drawdown: 13.82%
Sharpe:       2.01
```

See [Risk Analytics](risk.md) for the full `RiskSummary` field reference and standalone metric functions.

## Recommendations

Get similar stocks and analyst recommendations:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;
    let rec = ticker.recommendations(5).await?;

    println!("Similar stocks to {}:", ticker.symbol());

    for similar in &rec.recommendations {
        println!("  {} - {}", similar.symbol, similar.score);
    }
    Ok(())
}
```

```text soothfast-output
Similar stocks to AAPL:
  AMZN - 0.190787
  TSLA - 0.17989
  GOOG - 0.167981
  META - 0.160631
  MSFT - 0.147947
```

## Financial Statements

Get income statement, balance sheet, or cash flow statement:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Frequency, StatementType, Ticker};

    let ticker = Ticker::new("AAPL").await?;

    // Annual income statement
    let income = ticker
        .financials(StatementType::Income, Frequency::Annual)
        .await?;

    // Access data by metric name
    if let Some(revenue_map) = income.statement.get("TotalRevenue") {
        for (date, value) in revenue_map {
            println!("{}: Revenue ${:.2}B", date, value / 1e9);
        }
    }

    if let Some(net_income_map) = income.statement.get("NetIncome") {
        for (date, value) in net_income_map {
            println!("{}: Net Income ${:.2}B", date, value / 1e9);
        }
    }

    // Quarterly balance sheet
    let balance = ticker
        .financials(StatementType::Balance, Frequency::Quarterly)
        .await?;
    println!("Balance sheet line items: {}", balance.statement.len());

    // Cash flow statement
    let cashflow = ticker
        .financials(StatementType::CashFlow, Frequency::Annual)
        .await?;
    println!("Cash flow line items: {}", cashflow.statement.len());
    Ok(())
}
```

```text soothfast-output
2024-09-30: Revenue $391.04B
2023-09-30: Revenue $383.29B
2025-09-30: Revenue $416.16B
2022-09-30: Revenue $394.33B
2022-09-30: Net Income $99.80B
2025-09-30: Net Income $112.01B
2023-09-30: Net Income $97.00B
2024-09-30: Net Income $93.74B
```

**Statement Types:**

- `StatementType::Income` - Income statement (revenue, expenses, profit)
- `StatementType::Balance` - Balance sheet (assets, liabilities, equity)
- `StatementType::CashFlow` - Cash flow statement (operating, investing, financing)

**Frequencies:**

- `Frequency::Annual` - Yearly statements
- `Frequency::Quarterly` - Quarterly statements

## Options Data

Get options chains:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;

    // Get all available expiration dates and options
    let options = ticker.options(None).await?;

    // expiration_dates() and strikes() are methods
    println!("Available expiration dates:");
    for exp in options.expiration_dates() {
        println!("  {}", exp); // Unix timestamp (i64)
    }

    // calls() and puts() return a Contracts collection
    println!("\nCalls:");
    for call in &*options.calls() {
        println!(
            "  Strike ${:.2}: last=${:.2}, volume={}",
            call.strike,
            call.last_price.unwrap_or(0.0),
            call.volume.unwrap_or(0),
        );
    }

    println!("\nPuts:");
    for put in &*options.puts() {
        println!(
            "  Strike ${:.2}: last=${:.2}, IV={:.4}",
            put.strike,
            put.last_price.unwrap_or(0.0),
            put.implied_volatility.unwrap_or(0.0),
        );
    }

    // Get options for a specific expiration date
    let exp_dates = options.expiration_dates();
    if exp_dates.len() > 1 {
        let options_dated = ticker.options(Some(exp_dates[1])).await?;
        println!(
            "\nExpiration {}: {} calls, {} puts",
            exp_dates[1],
            options_dated.calls().len(),
            options_dated.puts().len()
        );
    }
    Ok(())
}
```

```text soothfast-output
Available expiration dates:
  1784505600
  1784678400
  1784851200
  1785110400
  1785283200
  1785456000
  1786060800
  1786665600
  1787270400
  1787875200
  1789689600
  1792108800
  1795132800
  1797552000
  1799971200
  1802995200
  1805414400
  1813190400
  1821139200
  1829001600
  1832025600
  1836864000
  1860451200

Calls:
  Strike $215.00: last=$100.19, volume=0
  Strike $220.00: last=$95.22, volume=0
  Strike $225.00: last=$90.25, volume=2
  Strike $230.00: last=$100.22, volume=16
  Strike $235.00: last=$79.00, volume=0
  Strike $240.00: last=$74.00, volume=0
  Strike $245.00: last=$70.12, volume=0
  Strike $250.00: last=$65.13, volume=0
  Strike $255.00: last=$60.17, volume=0
  Strike $265.00: last=$65.80, volume=4
  Strike $270.00: last=$56.85, volume=2
  Strike $275.00: last=$58.67, volume=22
  Strike $277.50: last=$56.19, volume=22
  Strike $280.00: last=$52.72, volume=177
  Strike $282.50: last=$51.10, volume=182
  Strike $285.00: last=$48.00, volume=10
  Strike $287.50: last=$42.57, volume=1
  Strike $290.00: last=$42.95, volume=16
  Strike $292.50: last=$41.62, volume=7
  Strike $295.00: last=$34.90, volume=17
  Strike $297.50: last=$36.43, volume=4
  Strike $300.00: last=$33.96, volume=17
  Strike $302.50: last=$32.84, volume=47
  Strike $305.00: last=$30.10, volume=61
  Strike $307.50: last=$24.69, volume=31
  Strike $310.00: last=$24.03, volume=25
  Strike $312.50: last=$20.50, volume=315
  Strike $315.00: last=$18.78, volume=526
  Strike $317.50: last=$16.50, volume=395
  Strike $320.00: last=$14.26, volume=1119
  Strike $322.50: last=$12.23, volume=407
  Strike $325.00: last=$9.50, volume=1921
  Strike $327.50: last=$7.13, volume=2112
  Strike $330.00: last=$5.13, volume=11294
  Strike $332.50: last=$3.53, volume=18310
  Strike $335.00: last=$2.24, volume=40861
  Strike $337.50: last=$1.25, volume=17554
  Strike $340.00: last=$0.75, volume=17286
  Strike $342.50: last=$0.41, volume=4087
  Strike $345.00: last=$0.23, volume=7212
  Strike $347.50: last=$0.13, volume=2097
  Strike $350.00: last=$0.07, volume=5473
  Strike $355.00: last=$0.03, volume=1365
  Strike $360.00: last=$0.02, volume=757
  Strike $365.00: last=$0.02, volume=32
  Strike $370.00: last=$0.02, volume=41
  Strike $375.00: last=$0.01, volume=11
  Strike $380.00: last=$0.03, volume=1

Puts:
  Strike $220.00: last=$0.05, IV=1.8359
  Strike $240.00: last=$0.14, IV=1.5156
  Strike $245.00: last=$0.15, IV=1.6680
  Strike $250.00: last=$0.03, IV=1.5723
  Strike $255.00: last=$0.01, IV=1.4785
  Strike $260.00: last=$0.01, IV=1.3867
  Strike $265.00: last=$0.45, IV=0.8125
  Strike $270.00: last=$0.22, IV=1.1328
  Strike $275.00: last=$0.01, IV=0.6875
  Strike $277.50: last=$0.08, IV=1.0674
  Strike $280.00: last=$0.02, IV=0.9590
  Strike $282.50: last=$0.02, IV=0.9141
  Strike $285.00: last=$0.01, IV=0.8691
  Strike $287.50: last=$0.32, IV=0.8252
  Strike $290.00: last=$0.05, IV=0.5156
  Strike $292.50: last=$0.02, IV=0.7373
  Strike $295.00: last=$0.05, IV=0.5898
  Strike $297.50: last=$0.01, IV=0.4531
  Strike $300.00: last=$0.02, IV=0.4531
  Strike $302.50: last=$0.01, IV=0.4414
  Strike $305.00: last=$0.03, IV=0.4102
  Strike $307.50: last=$0.04, IV=0.3906
  Strike $310.00: last=$0.04, IV=0.3672
  Strike $312.50: last=$0.06, IV=0.3418
  Strike $315.00: last=$0.07, IV=0.3252
  Strike $317.50: last=$0.11, IV=0.3018
  Strike $320.00: last=$0.17, IV=0.2837
  Strike $322.50: last=$0.29, IV=0.2710
  Strike $325.00: last=$0.46, IV=0.2590
  Strike $327.50: last=$0.82, IV=0.2559
  Strike $330.00: last=$1.39, IV=0.2468
  Strike $332.50: last=$2.30, IV=0.2556
  Strike $335.00: last=$3.55, IV=0.2505
  Strike $337.50: last=$5.25, IV=0.3021
  Strike $340.00: last=$6.75, IV=0.2993
  Strike $342.50: last=$9.14, IV=0.3765
  Strike $345.00: last=$11.10, IV=0.3731
  Strike $350.00: last=$16.00, IV=0.4810
  Strike $360.00: last=$29.63, IV=0.7222
```

## Event Calendar

`calendar(range)` aggregates this symbol's upcoming events — earnings (with
analyst estimates), ex-dividend and dividend-payment dates, and standard monthly
options expirations — into one list sorted ascending by timestamp. Events are
limited to the forward window `[now, now + range]`. With the `fred` feature
enabled, market-wide economic releases (CPI, NFP, GDP, …) are appended with a
`None` symbol.

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{EventKind, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;
    let events = ticker.calendar(TimeRange::ThreeMonths).await?;

    for event in &events {
        let symbol = event.symbol.as_deref().unwrap_or("market");
        match &event.event {
            EventKind::Earnings {
                eps_estimate_avg, ..
            } => {
                println!(
                    "{} {} earnings, est. EPS {:?}",
                    event.date, symbol, eps_estimate_avg
                );
            }
            EventKind::ExDividend { .. } => println!("{} {} ex-dividend", event.date, symbol),
            EventKind::DividendPayment { .. } => {
                println!("{} {} dividend paid", event.date, symbol)
            }
            EventKind::OptionsExpiration { .. } => {
                println!("{} {} options expire", event.date, symbol)
            }
            _ => println!("{} {} event", event.date, symbol),
        }
    }
    Ok(())
}
```

```text soothfast-output
2026-07-30 AAPL earnings, est. EPS Some(1.89396)
2026-08-21 AAPL options expire
2026-09-18 AAPL options expire
2026-10-16 AAPL options expire
```

**`CalendarEvent` fields:**

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `i64` | Unix timestamp (seconds) when the event occurs |
| `date` | `String` | ISO 8601 date string for display (e.g. `"2026-01-23"`) |
| `symbol` | `Option<String>` | Ticker the event belongs to; `None` for market-wide events |
| `event` | `EventKind` | The specific event and its payload |

**`EventKind` variants:** `Earnings` (analyst EPS/revenue estimates), `ExDividend`,
`DividendPayment`, `OptionsExpiration` (standard monthly only), and — with the
`fred` feature — `EconomicRelease`.

## News

Get recent news for the symbol:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;
    let news = ticker.news().await?;

    for article in &news {
        println!("{}", article.title);
        println!("  Source: {}", article.source);
        println!("  Published: {}", article.time);
        println!("  URL: {}", article.link);
        println!();
    }
    Ok(())
}
```

```text soothfast-output
Apple briefly overtakes Nvidia as world's most valuable company amid AI investment doubts
  Source: Fox Business
  Published: 21 hours ago
  URL: https://www.foxbusiness.com/markets/apple-briefly-overtakes-nvidia-worlds-most-valuable-company-amid-ai-investment-doubts

Apple and Google ordered to purge ‘nudify' apps from App Stores
  Source: TechCrunch
  Published: 22 hours ago
  URL: https://techcrunch.com/2026/07/17/apple-and-google-ordered-to-purge-nudify-apps-from-app-stores/

Berkshire's Equity Portfolio Is Rallying, but the Apple Sales Still Sting
  Source: Barrons
  Published: 22 hours ago
  URL: https://www.barrons.com/articles/berkshire-hathaway-apple-stock-portfolio-240a36cc

Apple (AAPL) Raises the Cost of Its Music Streaming Service
  Source: TipRanks
  Published: 22 hours ago
  URL: https://www.tipranks.com/news/apple-aapl-raises-the-cost-of-its-music-subscription-service

Apple raises prices for Apple Music and Apple One subscriptions
  Source: TheFly
  Published: 22 hours ago
  URL: https://www.tipranks.com/news/the-fly/apple-raises-prices-for-apple-music-and-apple-one-subscriptions-thefly-news

How Apple's big lawsuit could disrupt OpenAI's IPO plans
  Source: TechCrunch
  Published: 1 day ago
  URL: https://techcrunch.com/video/how-apples-big-lawsuit-could-disrupt-openais-ipo-plans/

Apple's lawsuit couldn't come at a worse time for OpenAI
  Source: TechCrunch
  Published: 1 day ago
  URL: https://techcrunch.com/podcast/apples-lawsuit-couldnt-come-at-a-worse-time-for-openai/

Apple in early settlement talks with US DOJ over antitrust case, Bloomberg News reports
  Source: Reuters
  Published: 1 day ago
  URL: https://www.reuters.com/legal/litigation/apple-early-settlement-talks-with-us-doj-over-antitrust-case-bloomberg-news-2026-07-17/

Apple in early talks with DOJ to settle antitrust suit, Bloomberg says
  Source: TheFly
  Published: 1 day ago
  URL: https://www.tipranks.com/news/the-fly/apple-in-early-talks-with-doj-to-settle-antitrust-suit-bloomberg-says-thefly-news

Apple in talks to settle DOJ antitrust lawsuit, Bloomberg reports
  Source: TheFly
  Published: 1 day ago
  URL: https://www.tipranks.com/news/the-fly/apple-in-talks-to-settle-doj-antitrust-lawsuit-bloomberg-reports-thefly-news

Apple's ‘Wait and See' AI Strategy Just Earned the Stock an Upgrade
  Source: Barrons
  Published: 1 day ago
  URL: https://www.barrons.com/articles/apples-stock-price-ai-iphone-upgrade-5095e27f

Apple (AAPL) Hits Dozens of OpenAI Staff With Legal Notices as Lawsuit Escalates
  Source: TipRanks
  Published: 1 day ago
  URL: https://www.tipranks.com/news/apple-aapl-hits-dozens-of-openai-staff-with-legal-notices-as-lawsuit-escalates

Apple races past Nvidia to reclaim crown as world's most valuable company
  Source: New York Post
  Published: 1 day ago
  URL: https://nypost.com/2026/07/17/business/apple-races-past-nvidia-to-reclaim-crown-as-worlds-most-valuable-company/

Apple Retakes Top Valuation Spot as Wall Street Rewards Smart AI Spending
  Source: PYMNTS
  Published: 1 day ago
  URL: https://www.pymnts.com/news/artificial-intelligence/2026/apple-retakes-top-valuation-spot-wall-street-rewards-smart-ai-spending/

Apple dethrones Nvidia to regain title of world's most valuable company
  Source: The Guardian
  Published: 1 day ago
  URL: https://www.theguardian.com/technology/2026/jul/17/apple-nvidia-most-valuable-company

Apple's stock is beating the S&P 500 by a remarkable degree — and it may have more room to run
  Source: Market Watch
  Published: 1 day ago
  URL: https://www.marketwatch.com/story/apples-stock-is-beating-the-s-p-500-by-a-remarkable-degree-and-it-may-have-more-room-to-run-1b0a1454

Apple reclaims title as world's most valuable company, overtaking Nvidia
  Source: Invezz
  Published: 1 day ago
  URL: https://invezz.com/news/2026/07/17/apple-reclaims-title-as-worlds-most-valuable-company-overtaking-nvidia/

Apple Unseats Nvidia As World's Largest Company
  Source: Forbes
  Published: 1 day ago
  URL: https://www.forbes.com/sites/tylerroush/2026/07/17/apple-unseats-nvidia-as-worlds-largest-company/

Apple Demands Documents From Former Employees Now at OpenAI
  Source: PYMNTS
  Published: 1 day ago
  URL: https://www.pymnts.com/apple/2026/apple-demands-documents-from-former-employees-openai/

Apple dethrones Nvidia as world's most valuable company, ending the chipmaker's long run at the top
  Source: CNBC
  Published: 1 day ago
  URL: https://www.cnbc.com/2026/07/17/apple-nvidia-aapl-nvda-market-cap.html
```

With the `sentiment` feature enabled, each article carries an optional
`sentiment` score (offline VADER, no API key), and `news_sentiment()` returns the
average sentiment across recent headlines:

```rust capture-output feature=sentiment
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;
    let news = ticker.news().await?;
    for article in &news {
        if let Some(s) = &article.sentiment {
            println!("{} → {} ({:+.2})", article.title, s.label.as_str(), s.score);
        }
    }

    let overall = ticker.news_sentiment().await?;
    println!(
        "Average coverage: {} ({:+.2})",
        overall.label.as_str(),
        overall.score
    );
    Ok(())
}
```

```text soothfast-output
Apple briefly overtakes Nvidia as world's most valuable company amid AI investment doubts → Bullish (+0.36)
Apple and Google ordered to purge ‘nudify' apps from App Stores → Neutral (+0.00)
Berkshire's Equity Portfolio Is Rallying, but the Apple Sales Still Sting → Neutral (+0.00)
Apple (AAPL) Raises the Cost of Its Music Streaming Service → Neutral (+0.00)
Apple raises prices for Apple Music and Apple One subscriptions → Neutral (+0.00)
How Apple's big lawsuit could disrupt OpenAI's IPO plans → Bearish (-0.23)
Apple's lawsuit couldn't come at a worse time for OpenAI → Bullish (+0.17)
Apple in early settlement talks with US DOJ over antitrust case, Bloomberg News reports → Neutral (+0.00)
Apple in early talks with DOJ to settle antitrust suit, Bloomberg says → Neutral (+0.00)
Apple in talks to settle DOJ antitrust lawsuit, Bloomberg reports → Bearish (-0.23)
Apple's ‘Wait and See' AI Strategy Just Earned the Stock an Upgrade → Neutral (+0.00)
Apple (AAPL) Hits Dozens of OpenAI Staff With Legal Notices as Lawsuit Escalates → Bearish (-0.10)
Apple races past Nvidia to reclaim crown as world's most valuable company → Bullish (+0.57)
Apple Retakes Top Valuation Spot as Wall Street Rewards Smart AI Spending → Bullish (+0.76)
Apple dethrones Nvidia to regain title of world's most valuable company → Bullish (+0.57)
Apple's stock is beating the S&P 500 by a remarkable degree — and it may have more room to run → Bullish (+0.15)
Apple reclaims title as world's most valuable company, overtaking Nvidia → Bullish (+0.57)
Apple Unseats Nvidia As World's Largest Company → Neutral (+0.00)
Apple Demands Documents From Former Employees Now at OpenAI → Neutral (+0.00)
Apple dethrones Nvidia as world's most valuable company, ending the chipmaker's long run at the top → Bullish (+0.67)
Average coverage: Bullish (+0.16)
```

## Earnings Transcripts

Get earnings call transcripts:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Ticker, finance};

    let ticker = Ticker::new("AAPL").await?;

    // Get latest transcript
    let transcript = finance::earnings_transcript(ticker.symbol(), None, None).await?;

    println!(
        "Transcript for {} - Q{} {}",
        ticker.symbol(),
        transcript.quarter(),
        transcript.year()
    );

    // Access paragraphs with speaker names resolved
    for (paragraph, speaker) in transcript.paragraphs_with_speakers() {
        if let Some(name) = speaker {
            println!("[{:.1}s] {}: {}", paragraph.start, name, paragraph.text);
        }
    }

    // Get a specific quarter transcript
    let _q1 = finance::earnings_transcript(ticker.symbol(), Some("Q1"), Some(2024)).await?;

    // Get all available transcripts (metadata only)
    let all_transcripts = finance::earnings_transcripts(ticker.symbol(), None).await?;

    for meta in &all_transcripts {
        println!(
            "{} {} - {}",
            meta.year.unwrap_or(0),
            meta.quarter.as_deref().unwrap_or("?"),
            meta.title
        );
    }
    Ok(())
}
```

```text soothfast-output
Transcript for AAPL - QQ2 2026
[0.1s] Suhasini Chandramouli: Good afternoon, welcome to the Apple Q2 fiscal year 2026 earnings conference call. My name is Suhasini Chandramouli, Director of Investor Relations. Today's call is being recorded. Speaking first today is Apple CEO Tim Cook. John Ternus will be joining after that for a brief set of remarks, and he'll be followed by CFO Kevan Parekh. After that, we'll open the call to questions from analysts. Please note that some of the information you'll hear during our discussion today will consist of forward-looking statements, including, without limitation, those regarding revenue, gross margin, operating expenses, other income and expense, taxes, capital allocation, and future business outlook.
[48.0s] Suhasini Chandramouli: These statements involve risks and uncertainties that may cause actual results or trends to differ materially from our forecast, including risks related to the potential impact to the company's business and results of operations from macroeconomic conditions, tariffs and other measures, and legal and regulatory proceedings. For more information, please refer to the risk factors discussed in Apple's most recently filed reports on Form 10-Q and Form 10-K and the Form 8-K filed with the SEC today, along with the associated press release. Additional information will also be in our report on Form 10-Q for the quarter ended March 28th, 2026 to be filed tomorrow and in other reports and filings we make with the SEC. Apple assumes no obligation to update any forward-looking statements which speak only as of the date they are made. I'd now like to turn the call over to Tim for introductory remarks.
[110.0s] Tim Cook: Thank you, Suhasini. Good afternoon, everyone, and thanks for joining the call. Before we get into the quarter, I wanted to take a moment to talk about the transition we recently announced. I just celebrated my 28th anniversary of being here at Apple, 15 years as CEO. In fact, this will be my 89th earnings call. I'll always be proud of the impact Apple has had on our users' lives, and I can't begin to express how grateful I am for our amazing teams. It's because of them that there is no company like Apple, and I truly believe there never will be. This moment for the transition is the right one for a number of reasons. First, our business has been performing extremely well. The first half of this year was very strong, growing double digits year-over-year. Second, our roadmap is incredible.
[163.7s] Tim Cook: Most importantly, we have the right leader ready to step into the role. As I have said, there is no 1 on this planet I trust more to lead Apple into the future than John Ternus. John is a brilliant engineer, a deep thinker, a person of remarkable character, and a born leader. I know he will push us to go further than we think is possible in order to deliver the greatest products and services for our users. I have been so proud to call him a colleague and a friend, and I will be even more proud to call him Apple's CEO. Over the coming months, John and I will be working closely together to make sure this transition is perfectly smooth. I very much look forward to stepping into the role of Executive Chairman on September 1st.
[211.2s] Tim Cook: As I've told John, I will be here to support him in any way he needs and in any way I can. I am incredibly optimistic about Apple's future, and I know we have the right team in place to deliver on the promise of this company. I also want to take just a moment to share my profound gratitude for our shareholders, especially our long-term shareholders, for believing in Apple and for your support over the years. It means a great deal to all of us. With that, I'd like to bring John on the call for a moment to say a few words. John?
[246.9s] John Ternus: Thanks, Tim, and thanks to everyone on the call. In my view, Tim is one of the greatest business leaders of all time. Stepping into the role of CEO is an incredible honor, and it means a great deal to me to have Tim's trust and confidence. I want to echo Tim's sentiment about our shareholders, especially those who have been with us for many years. Thank you so much for your confidence in our company. As you know, one of the hallmarks of Tim's tenure has been a deep thoughtfulness, deliberateness, and discipline when it comes to the financial decision-making of the company. I want you to know that is something Kevan and I intend to continue when I transition into the role in September. This is an especially exciting moment for Apple. As Tim mentioned, we have an incredible roadmap ahead.
[291.6s] John Ternus: While you're not going to get me to talk about the details of that roadmap, suffice it to say this is the most exciting time in my 25-year career at Apple to be building products and services. There are so many opportunities before us, and I couldn't be more optimistic about what's to come. For now, let me simply say I am deeply grateful to Tim, to the executive team, and to everyone at Apple, and I look forward to all of the important work ahead. With that, let me turn it back over to Tim.
[319.8s] Tim Cook: Thanks, John. Let me turn to the quarter. Today, Apple is proud to report $111.2 billion in revenue, up 17% from a year ago, and a March quarter record, which was above the high end of our guidance range despite supply constraints. Customer enthusiasm for iPhone has been extraordinary, with revenue growing 22% year-over-year to achieve a March quarter record. Services reached an all-time revenue record, growing 16% from a year ago, while EPS set a March quarter record of $2.01, up 22% year-over-year. We set March quarter revenue records and grew double digits in every geographic segment, including strong double-digit growth in Greater China and the rest of Asia Pacific.
[373.0s] Tim Cook: We also achieved March quarter revenue records in both developed and emerging markets and saw double-digit growth in nearly every emerging market we track, including India. We recently marked Apple's 50th anniversary with celebrations in our retail stores and with users around the world. It was a special moment for us to reflect on the incredible journey we've shared with our users, to thank everyone who's been a part of it, and to look forward to writing the next chapter in our story of innovation. We have always believed that people who think different can change the world, and we have been proud to build tools and technologies that allow them to do just that. In March, we put an amazing showcase of human creativity and ingenuity in action with updates across iPhone, iPad, and Mac.
[425.9s] Tim Cook: Through an unforgettable week of innovation, we also unveiled MacBook Neo, giving us an opportunity to bring the power of Mac to more people than ever before. I'll have more to say on that and all the incredible things we delivered for our customers over the last few months. Let's take a closer look at results from across our product line, beginning with iPhone. As I mentioned earlier, iPhone had an excellent quarter with $57 billion in revenue, a March quarter record despite supply constraints. During the quarter, we welcomed iPhone 17e, the newest addition to what is already the strongest iPhone lineup we've ever had. It brings outstanding performance and core iPhone experiences at a remarkable value for everyone from enterprise teams to consumers. Across the lineup, this is the most powerful, capable, and versatile iPhone family we've ever created.
[489.1s] Tim Cook: That starts with the latest in Apple silicon for iPhone, A19 and A19 Pro, which include neural accelerators in the GPU to deliver a huge boost to AI performance. With incredible performance and battery life and deep integration of Apple Intelligence, iPhone continues to set the standard for what a smartphone can be. Customers are capturing stunning photos and videos with our most advanced camera system ever on iPhone 17 Pro and Pro Max, including an 8x optical quality zoom and the all-new Center Stage front camera, unlocking entirely new ways to frame, create, and share their moments. In fact, during their recent mission, Artemis II astronauts captured some truly otherworldly images of Earth and space using iPhone 17 Pro Max. Meanwhile, iPhone Air users are tapping into the pro-level performance in our slimmest iPhone ever.
[555.3s] Tim Cook: With iPhone 17, we're seeing a strong response, not only from customers upgrading from previous generations, but also from people choosing iPhone for the very first time. We've been enormously pleased with how the entire lineup has been received. In fact, the iPhone 17 family is now the most popular lineup in our history when looking at the launch through the March quarter. According to IDC, we gained market share during the quarter. Mac revenue was $8.4 billion for the March quarter, up 6% from a year-ago, despite supply constraints driven by higher-than-expected levels of demand. We're delighted with the reception of what is the most advanced Mac lineup in our history. We set March quarter records for upgraders and customers new to Mac. According to IDC, we gained market share in the quarter.
[615.4s] Tim Cook: From Mac mini to MacBook Pro and everything in between, Mac is the best platform for AI, with Apple silicon delivering exceptional performance, industry-leading efficiency, and the ability to run advanced models locally in ways that simply weren't possible before. It's so exciting to see how strongly users are embracing Mac for these capabilities. There's tremendous enthusiasm for MacBook Neo, which made its debut during the March quarter, opening up an entirely new way to experience Mac at a breakthrough price. We've also further improved MacBook Air, already the world's most popular laptop, with M5, making everyday tasks faster and more responsive than ever. MacBook Pro reaches new heights with M5 Pro and M5 Max, delivering extraordinary performance and dramatically advancing what users can do with AI on a portable system.
[678.9s] Tim Cook: For desktop users, Studio Display pairs beautifully with Mac, while the all-new Studio Display XDR takes things even further, bringing unmatched image quality and an extraordinarily immersive experience to pro workflows. Turning to iPad, revenue was $6.9 billion, up 8% from a year ago. iPad continues to be a great choice for students, small business owners, artists, and so many others because it empowers entirely new ways to work, learn, create, and connect. It's not just about mobility. It's about versatility, delivering a uniquely flexible experience that adapts to whatever users want to accomplish. Today, our iPad lineup is stronger than ever, led by the arrival of the M4-powered iPad Air. With a remarkable leap in performance, it raises the bar for what users can do on iPad, from advanced creative workflows to powerful productivity and immersive learning.
[748.7s] Tim Cook: With the addition of our latest Apple silicon, along with the N1 wireless networking chip and C1X modem, users can stay seamlessly connected wherever they are. Across wearables, home, and accessories, revenue for the March quarter came in at $7.9 billion, up 5% from a one year ago. Apple Watch Ultra 3, Apple Watch Series 11, and Apple Watch SE continue to play an essential role in users' lives, going far beyond fitness tracking to deliver meaningful insights and support for their health and well-being. From helping users stay active and reach their fitness goals to delivering powerful, science-backed health insights that can prompt meaningful conversations with care providers, Apple Watch is with them every step of the way. It's tremendously meaningful to see how Apple Watch continues to empower users to better understand their health, make more informed decisions, and in many cases change and even save lives.
[816.7s] Tim Cook: During the quarter, we introduced customers to a new level of audio experience with AirPods Max 2, delivering stunning sound quality and our most advanced active noise cancellation yet. At the same time, AirPods Pro 3 combine an incredibly immersive listening experience with intelligent features that adapt to how users move, train, and live. Whether it's a call across town or a conversation across continents, AirPods make it effortless to stay connected. AirPods can bridge languages too, thanks to live translation powered by Apple Intelligence. In addition to live translation, Apple Intelligence brings together dozens of powerful capabilities from visual intelligence to cleanup in photos that are seamlessly integrated into the moments that matter most to our users every day. We look forward to bringing a more personalized Siri to users coming this year.
[880.5s] Tim Cook: What truly sets Apple apart is how Apple Intelligence is woven into the core of our platforms, powered by Apple silicon and designed from the ground up to deliver intelligence that is fast, personal, and private. This is not AI as a standalone feature, but AI as an essential, intuitive part of the experience across our devices. It builds on years of innovation, from the neural engine to advanced on-device processing, enabling capabilities that are not only incredibly powerful, but also respectful of user privacy. Increasingly, that same foundation is drawing developers and researchers to our products as powerful platforms for building and running agentic AI, thanks to the unique combination of performance, efficiency, and on-device capabilities. When you combine this level of integration with our relentless focus on the customer experience, it becomes clear why Apple platforms are the best place to experience AI.
[949.1s] Tim Cook: Now let's turn to services, which set an all-time revenue record with $31 billion. We saw double-digit growth in both developed and emerging markets and set new all-time revenue records across most of the services categories. There's no better place to find celebrated storytellers than Apple TV. Audiences are applauding the return of shows like Your Friends & Neighbors, Shrinking, and For All Mankind, while discovering new favorites like Widow's Bay. Apple TV has also earned its place among the most decorated names in entertainment, with more than 800 wins and more than 3,400 nominations in the six years since launch. This is a great time for sports fans on Apple TV too. Formula One season kicked off in March, and Apple TV subscribers in the U.S. have one of the best views of the track.
[1006.5s] Tim Cook: The new MLS season is also well underway, and subscribers in more than 100 countries and regions can watch every match with no blackouts. Friday Night Baseball returned for its fifth year on Apple TV with a full season of marquee matchups. In retail, we had a March quarter revenue record and saw very high levels of store traffic throughout the quarter. From New York to Chengdu to Paris, it was wonderful to see stores around the world at the center of Apple's 50th anniversary celebrations. We were also thrilled to open the doors to our sixth store in India. It has been wonderful to see how we've continued to grow in India in recent years, part of our larger efforts to connect with even more customers in emerging markets all over the world.
[1059.9s] Tim Cook: At Apple, we believe powerful innovation and uncompromising quality can go hand in hand with sustainability. Over the last year, we've reached new milestones in the environment, including the use of recycled content in 30% of the materials in all of our products shipped in 2025, the most we've ever had. That includes the use of 100% recycled cobalt in all Apple-designed batteries and 100% recycled rare earth elements in all magnets. We've also achieved our goal of removing plastic from packaging with every Apple product now shipping in fiber-based packaging. All of this is a testament to the outstanding forward-thinking and innovative work of our teams. We're also making great progress in advancing American supply chain innovation.
[1117.0s] Tim Cook: As part of our $600 billion commitment to the U.S., we were pleased to share recently that Mac mini production is coming to America later this year, expanding our factory operations in Houston with a brand-new facility. In March, we were thrilled to welcome four new companies to our American manufacturing program to help manufacture essential materials and components for Apple products sold worldwide. These include sensors that support key iPhone features like camera stabilization and integrated circuits essential for features like crash detection and activity tracking. These efforts build on the progress we've made in the American manufacturing program, including the work we're doing to advance an end-to-end silicon supply chain across the U.S. At TSMC's Arizona facility, for example, Apple is on track to purchase well over 100 million advanced chips.
[1179.5s] Tim Cook: We're accelerating our long-standing support for U.S. innovation, we're also investing in America's workforce. We're looking forward to opening the doors to an all-new advanced manufacturing center in Houston later this year, which will provide hands-on training led by Apple experts and tailor-made for students, supplier employees, and American businesses. Whether around the world or in our own backyard, we're proud of the difference Apple has made to enrich lives and support the communities we serve. Looking ahead, we're delighted to welcome developers back to Apple Park for WWDC 2026. We can't wait to share what we've been working on, from AI advancements to exciting new software and developer tools. It's going to be an incredible week. As always, we remain in relentless pursuit of even more powerful innovations guided by our North Star, our users.
[1242.3s] Tim Cook: As we celebrated 50 years of Apple, we are even more excited and more optimistic about the next 50 years and beyond. With that, I'll turn it over to Kevan.
[1254.2s] Kevan Parekh: Thanks, Tim. Good afternoon, everyone. Our revenue of $111.2 billion was up 17% year-over-year, a March quarter revenue record. We saw strong performance around the world with March quarter revenue records in every geographic segment. Foreign exchange was about a 2.5 percentage point tailwind to the March quarter growth rate. We also faced supply constraints on iPhone and to a lesser extent on Mac. We believe if you remove the favorable benefit from foreign exchange and add back the unfavorable impact from supply constraints, we would have had a higher growth rate for total company revenue for the quarter. Products revenue was $80.2 billion, up 17% year-over-year, driven by double-digit growth on iPhone, setting a new March quarter record.
[1302.5s] Kevan Parekh: Our install base of over 2.5 billion active devices has reached another all-time high across all major product categories and geographic segments. Services revenue was $31 billion, up 16% year-over-year. We saw strong performance across the board with double-digit growth in the vast majority of the markets we track. Company gross margin was 49.3%, above the high end of our guidance range and up 110 basis points sequentially. Products gross margin was 38.7%, down 200 basis points sequentially. Services gross margin was 76.7%, up 20 basis points sequentially. Operating expenses landed at $18.9 billion, up 24% year-over-year. This was slightly above the high end of our guidance range due to a one-time expense in SG&A.
[1357.8s] Kevan Parekh: Net income was $29.6 billion, and diluted earnings per share was $2.01, up 22% year-over-year. Both net income and diluted EPS achieved March quarter records and drove a very strong level of operating cash flow at $28.7 billion. I'm going to provide some more details for each of our revenue categories. iPhone revenue was $57 billion, up 22% year-over-year, driven by the iPhone 17 family. iPhone grew double digits in the majority of markets we track, including the U.S., Latin America, Greater China, Western Europe, India, Japan, and Southeast Asia. The iPhone active install base grew to an all-time high, and we set March quarter record for iPhone upgraders. According to a recent survey from Worldpanel, iPhone was a top-selling model in the U.S., urban China, the U.K., Australia, and Japan.
[1424.2s] Kevan Parekh: We have been extremely pleased with the positive reception of the iPhone 17 family. In fact, customer satisfaction for the iPhone 17 family in the U.S. was recently measured at 99% by 451 Research. Mac revenue was $8.4 billion, up 6% year-over-year, driven by the strength of the recent product launches, including MacBook Neo. We grew in both developed and emerging markets with double-digit growth in many emerging markets, including India and Indonesia. As Tim mentioned earlier, we had a March quarter record for customers new to the Mac. And this helped drive a new all-time record for the overall Mac install base. In the U.S., customer satisfaction for Mac was recently reported at 97%. iPad revenue was $6.9 billion, up 8% year-over-year, driven by the continued strength of the A16-powered iPad and the M5-powered iPad Pro.
[1485.2s] Kevan Parekh: The iPad install base reached a new all-time high as iPad continued to reach new customers around the world. During the quarter, over half of the customers who purchased an iPad were new to the product. Many of these customers are in our emerging markets, where we grew iPad revenue by double digits, including in India, Mexico, and Thailand. Based on the latest reports from 451 Research, customer satisfaction was 98% in the U.S. Wearables, home, and accessories revenue was $7.9 billion, up 5% year-over-year, driven by strength in wearables and accessories. We were pleased to see strength in our emerging markets, where we set a new March quarter revenue record. The wearables install base reached a new all-time high, with over half of the customers purchasing an Apple Watch during the quarter being new to the product.
[1541.4s] Kevan Parekh: In the U.S., customer satisfaction on Apple Watch was measured at 96%. Our services revenue reached an all-time high of $31 billion, up 16% year-over-year. The strong performance was broad-based, with all-time records in both developed and emerging markets. As Tim mentioned, we also set all-time revenue records in most of the services categories. We're optimistic about the future of our services business. With our large install base of over 2.5 billion active devices, we have an incredibly strong foundation for growth opportunities. Both transacting and paid accounts reached new all-time highs in the quarter as we continue to see more customers leveraging our services offerings. We continue to improve the quality and expand the breadth of our services from the expansion of features like Tap to Pay, now available in over 50 markets, to deeper support for enterprise customers.
[1602.0s] Kevan Parekh: Building on this, we've launched Apple Business, a new all-in-one platform that combines our hardware, software, and enterprise services, enabling companies to efficiently manage their deployments and scale their business. We continue to see more organizations and enterprise choosing Apple's devices for performance and productivity. Marsh, a leading professional services firm, deployed a large-scale refresh of corporate devices to iPhone 17 as part of a commitment to security alongside adopting Mac for internal AI development. With Apple silicon and its powerful unified memory architecture, leading AI developers like Perplexity are choosing Mac as their preferred platform to build enterprise-grade AI assistants that power autonomous agents and boost workplace productivity. Across the Mac lineup, customers are finding the right device for their needs.
[1657.4s] Kevan Parekh: From MacBook Pro and MacBook Air to our newest edition, MacBook Neo, which delivers an unprecedented combination of quality, value, and industry-leading security that is resonating strongly in enterprise and education. Kansas City Public Schools, for example, is switching their high school students from Windows laptops and Chromebooks to MacBook Neo, completing their transition to an all-Apple district. In India, leading enterprise software provider Freshworks deployed over 5,000 MacBook Pro and MacBook Air to accelerate their AI development. Let's turn to our cash position and capital return program. We ended the quarter with $147 billion in cash and marketable securities. We had $5.8 billion of debt maturities, and commercial paper remained unchanged at $2 billion, resulting in $85 billion in total debt. Therefore, at the end of the quarter, net cash was $62 billion.
[1720.3s] Kevan Parekh: During the quarter, we returned $15 billion to shareholders. This included $3.8 billion in dividends and equivalents and $11 billion through open market repurchases of 42 million Apple shares. Our repurchase activity at any time can be affected by a number of factors that we take into account. As you're aware, we recently announced a CEO transition. Taking a step back, we plan to continue our capital allocation philosophy of first making all the necessary investments needed to support the business and then returning excess cash to shareholders over time. Net cash neutral has been a valuable framework for our capital structure, and since 2018, we have significantly right-sized our balance sheet and reduced net cash by over $100 billion.
[1773.0s] Kevan Parekh: As we move ahead, we are no longer providing net cash neutral as a formal target, and we will independently evaluate cash and debt. Capital returns will continue to be important to our overall approach by delivering long-term shareholder value. Accordingly, our board has authorized an additional $100 billion for share repurchases, and we're also raising our dividend by 4% to $0.27 per share of common stock. This cash dividend will be payable on May 14th, 2026 to shareholders of record as of May 11th, 2026. As we move ahead into the June quarter, I'd like to review our outlook, which includes the types of forward-looking information that Suhasini referred to.
[1820.0s] Kevan Parekh: Importantly, the color we're providing assumes that global tariff rates, policies, and their application remain in effect as of this call. The global macroeconomic outlook does not worsen from today. We expect our June quarter total company revenue to grow by 14%-17% year-over-year, which comprehends our best view of constrained supply. On iPad, keep in mind, we face a difficult compare driven by the launch of the A16-powered iPad in the prior year. We expect services revenue to grow at a year-over-year rate similar to what we reported in the March quarter after removing the favorable year-over-year impact from foreign exchange tailwinds. Keep in mind, during the March quarter, FX was a 2.5 percentage point tailwind to the total company growth rate, and for services, that impact was slightly more favorable.
[1879.0s] Kevan Parekh: We expect gross margin to be between 47.5% and 48.5%. We expect operating expenses to be between $18.8 billion and $19.1 billion. We expect OINE to be around $250 million, excluding any potential impact from the mark-to-market of minority investments, and our tax rate to be around 17%. With that, Tim and I will take questions.
[1909.1s] Suhasini Chandramouli: Thank you, Kevan. We ask that you limit yourself to two questions. Operator, may we have the first question, please?
[1918.1s] Operator: Certainly. We'll go ahead and take our first question from Erik Woodring with Morgan Stanley. Please go ahead.
[1925.4s] Erik Woodring: Great. Thank you very much for taking my questions, guys. Tim, I'll save the congrats or the au revoir for next quarter. It's been a pleasure working together. I would love, maybe, Tim, if I could ask you just to maybe contextualize the supply constraints you alluded to in your prepared remarks. Meaning, you know, how much did demand outpace supply for iPhone and Mac in the March quarter? Does your June quarter guidance also reflect supply constraints for those segments? Or is that kind of an unconstrained guide as you see it today? A quick follow-up, please. Thank you.
[1959.6s] Tim Cook: Yeah. Hi, Erik. Thanks for your comments. We were constrained during the March quarter. This was primarily on iPhone and to a lesser extent on the Mac. As we talked about in the last call, the constraints were primarily driven by the availability of the advanced nodes our SOCs are produced on. If you look forward to the June quarter, the majority of our supply constraints will be on several Mac models, given the continued high levels of demand that we're seeing. We have less flexibility in the supply chain than we normally would. For Mac, in the June quarter, there's 2 factors that are driving the constraints.
[2005.9s] Tim Cook: One is that on the Mac mini and the Mac Studio, both of these are amazing platforms for AI and agentic tools, and the customer recognition of that is happening faster than what we had predicted. We saw higher than expected demand. The second reason is that the customer response to MacBook Neo has just been off the charts, with higher than expected demand. The March quarter record for customers, we set a March quarter record for customers new to the Mac, partly due to the Neo. We think, looking forward, that the Mac mini and the Mac Studio may take several months to reach supply-demand balance. Hopefully that gives you a view of both Q2 and Q3 on the supply side.
[2065.5s] Erik Woodring: All right. Awesome. Thank you very much for that color, Tim. You know, Kevan, I'd love to maybe turn to you and kind of a surprise little announcement there talking about net cash neutral. Still a great path, but we're no longer providing this as a formal target. Could you maybe expand on that a bit? Are we thinking about any different type of capital return policy? It doesn't seem so, but maybe give a little bit more detail when you talk about making investments. Is that organic versus inorganic? Just maybe tease that comment out a little bit more for us would be super helpful. Thank you so much, guys.
[2099.1s] Kevan Parekh: Erik, thanks for the question. Let me just reiterate what we said, which is really more of a comment on the capital structure. Our goal of net cash neutral has really served us well. It's been a valuable framework for us, you know, and for our capital structure since, you know, 2018. We believe we're at a stage where we're evaluating cash and debt independently is really the right approach for us and allows us to make more optimal economic decisions around how we best utilize our debt and cash portfolios to support the business, you know, based on business factors and market conditions. We also believe we can manage this flexibility while also being very efficient and remaining disciplined.
[2135.4s] Kevan Parekh: With all that being said, we remain very committed to returning excess cash to shareholders. As we talked about, you know, our investment in the business, I think, as you know, we invest in the business, you know, first and foremost, and then look to, you know, kind of return excess cash to shareholders. I think we have a very good track record of being disciplined. You know, we've returned over $1 trillion to shareholders from the start of the program, over $850 billion of which has been through share repurchases. You know, and the other piece as well that's really important is, you know, as part of that, we also have, you know, increased our buyback authorization by another $100 billion, and that's on top of, you know, the leftover capacity from the prior authorization.
[2173.5s] Kevan Parekh: You can see, you know, the capital return piece is something, you know, very important to us, and as we talked about in the prepared remarks, important to the overall approach, you know, to delivering long-term shareholder value.
[2186.2s] Erik Woodring: Thanks so much, Kevan. Good luck, guys.
[2187.8s] Kevan Parekh: Thanks, Erik.
[2188.2s] Tim Cook: Thank you.
[2188.4s] Suhasini Chandramouli: Awesome. Thank you, Erik. Operator, could we get the next question, please?
[2194.2s] Operator: Our next question is from Ben Reitzes with Melius Research. Please go ahead.
[2199.9s] Ben Reitzes: Yeah. Hey, thanks. I'll ask 2 myself. The first one is, there's just been a lot of talk and it's great to, by the way, speak with you, Tim and John, and Kevan. The first question is around there's been some commentary around an agentic smartphone. By the way, I don't even know what that means, but there's comments that, you know, about AI on the edge and that agents could catalyze smartphones, but also shift the smartphone kind of form factor or maybe not. I was just wondering, with the rise of agents, how you would like us to think about that. Is
[2240.2s] Ben Reitzes: Does this mean there's new products coming, of a totally new form factor, or does it change the game or anything high level you might wanna say about that and that trend or potential non-trend? Thanks.
[2253.1s] Tim Cook: Hi, Ben, it's Tim. you know, we don't get into our future roadmap. I don't wanna, you know, give too much info there, but I would just say that we're thrilled with how the iPhone is doing, growing 22% in the quarter and followed up from a incredible Q1 and having the strongest cycle that we've ever had in our history from the launch through March quarter. We could not be happier with it.
[2288.6s] Ben Reitzes: Okay. Well, thanks. I appreciate that. I'm sure we'll hear a lot more. With regard to, I guess the question around constraints and whatnot, Tim, you know, I may push you one more time. Try to do it nicely, though, just given my age. The big concern out there is it's maybe how margins go after the June quarter, given the components and trends and whatnot and all these constraints. Is there some kind of overarching philosophy that you want us to think about?
[2327.8s] Ben Reitzes: Do you feel, and maybe Kevan wants to weigh in on this, is do you see a lot of variability in the model, or is 47, 48 kind of a range you think you might be able to stay in, or is there just no visibility, you know, beyond June to answer this question? I, you know, I think any comfort level there, as we go throughout the calendar year would be so helpful. Thanks.
[2348.9s] Tim Cook: Yeah. Ben, let me talk about memory specifically, which I think is the root of the question. And I'll go back to December for a moment and just walk you through the chronology. In the December quarter, we really had a minimal impact due to memory, and you can kinda see that in the gross margin results. We said it would be a bit more in the March quarter, and we did see higher memory costs in the March quarter, and they were partially offset by benefits from carry-in inventory that we had. For the June quarter, and what's embedded in the guidance that Kevan went through earlier, we expect significantly higher memory costs. They are also partly offset by the benefit of carry-in inventory.
[2403.5s] Tim Cook: Then where we don't give color beyond June, I can tell you that beyond the June quarter, we believe memory costs will drive an increasing impact on our business. We'll continue to evaluate this, and as we've said before, we'll look at a range of options.
[2426.8s] Ben Reitzes: Okay. Thanks, Tim.
[2427.7s] Tim Cook: Yeah. Thank you, Ben.
[2429.5s] Suhasini Chandramouli: Thank you, Ben. Operator, could we have the next question, please?
[2434.9s] Operator: Our next question comes from Michael Ng with Goldman Sachs. Please go ahead.
[2440.4s] Michael Ng: Hey, good afternoon. Thank you for the questions. I have two as well. First, given the success of the MacBook Neo, I was wondering if you could talk a little bit about how it's helped drive penetration with new customer segments, whether that be, you know, education or value or emerging markets. Then, how do you think about opportunities in, you know, under-penetrated markets more broadly, how will your future product roadmap inform that strategy? Thank you.
[2471.9s] Tim Cook: Yeah. Right now we're supply constrained on the MacBook Neo. The response has been. We were very bullish on the product before announcing it. We undercalled the level of enthusiasm that would be with it. It's very much focused on getting the Mac to even more people than we were reaching before. We're very focused on customers new to the Mac and customers that have been holding onto their Mac a very long period of time. We're doing well with both of those.
[2512.0s] Tim Cook: As Kevan alluded to in his comments, we're seeing school systems like the Kansas City Public Schools that are switching from Chromebooks and Windows PCs to the MacBook Neo, and I'm hearing anecdotally more and more of those kind of stories, both happening at the school system level and at the individual consumer level. We could not be happier with how things are going at the moment.
[2545.7s] Michael Ng: Great. Thank you, Tim. For the second question, I wanted to ask about advertising within services. I think Apple introduced new inventory to ads on the App Store earlier this year. Has that new ad inventory on the App Store been a notable contributor to the services growth and outperformance in the quarter? Could you talk more broadly about your ad strategy, given the plans to also introduce ads to Maps this summer. Thank you.
[2574.8s] Kevan Parekh: Michael Ng, it's Kevan Parekh. Thanks for the question. In advertising, we did see year-over-year growth in our advertising business. As you alluded to, we recently did introduce additional ads, you know, across the App Store search results to provide developers with more ways to drive downloads on platforms, you know, that users trust. And this summer, as you said, in the U.S. and Canada, Apple Maps will feature ads during key search and discovery moments, creating a new way for local businesses to reach customers and explore new places. Importantly, I think, you know, we believe it's possible to help businesses of all sizes grow via advertising while still delivering a great customer experience, while also importantly respecting people's fundamental right to privacy.
[2615.9s] Michael Ng: Thank you, Kevan.
[2618.3s] Kevan Parekh: Thanks, Mike.
[2619.8s] Suhasini Chandramouli: Thank you, Mike. Operator, could we get the next question, please?
[2625.4s] Operator: Our next question is from Wamsi Mohan with Bank of America. Please go ahead.
[2631.0s] Wamsi Mohan: Hi, yes. Thank you so much. Tim, you noted higher impact from memory as you look beyond the June quarter. Clearly, you guys have a lot of scale, supply chain efficiencies, relationships from a long time. As you think about product position relative to your competitors, so when you think about product position and pricing relative to competition, do you think in such times of dislocation that Apple would be strategically more focused on share gain or where potentially you don't raise pricing and perhaps lower ends of the portfolio where your competitors are struggling or more focused on profitability? Like, what's the right framework for us to think through as you enter that period? I have a follow-up.
[2678.6s] Tim Cook: Wamsi, we will look at a range of options, with memory costs increasing. I really don't wanna go beyond that at this point.
[2691.8s] Wamsi Mohan: Okay, Tim. As a follow-up here, how's Apple thinking about the broader monetization, maybe following Ben's question here in the agentic AI world? What parts of the stack do you think Apple will be focused on, internally versus maybe leveraging your partners? I mean, we have some early looks into where you are developing relationships. As we think longer term, where will Apple invest more heavily over the next several years? Is this at all related to your net cash comments in terms of perhaps building out more infrastructure as we enter an AI-centric world? Thank you.
[2734.1s] Tim Cook: We are clearly investing more. You can see that in the OpEx numbers. If you click down on those a step deeper and look at the R&D areas separate than SG&A, you'll find that R&D is even accelerating much higher than the company is. We're clearly investing. We're investing in products and services, and we see opportunities in both of those. We could not be more excited about how the future is playing out.
[2766.7s] Kevan Parekh: I think Wamsi, as we've talked about, you know, building on what Tim said, you know, from the start, we've said we, you know, believe AI is a really important investment area for Apple, and we're gonna be doing that incrementally on top of what we normally invest in our product, you know, roadmap. I think just wanted to reiterate that point as well.
[2783.2s] Wamsi Mohan: Okay. Thanks, Tim. Thanks, Kevan.
[2784.7s] Kevan Parekh: Mm-hmm.
[2784.9s] Suhasini Chandramouli: Awesome.
[2785.3s] Tim Cook: Yep.
[2785.4s] Suhasini Chandramouli: Thank you, Wamsi. Operator, could we get the next question, please?
[2791.3s] Operator: Our next question is from Amit Daryanani with Evercore. Please go ahead.
[2797.4s] Amit Daryanani: Yep, I have two as well. Good afternoon, everyone. You know, I guess first one, maybe just going back to the iPhone performance, which, you know, for a couple of quarters you folks have had 20% plus growth despite the supply constraint, and I think the guide sort of implies the momentum will continue in June. I'd love for you folks to just maybe double-click and talk about what are the levers that's driving this sort of impressive iPhone growth despite the supply constraints, and then sort of what is the durability of this growth.
[2823.3s] Tim Cook: Yeah, if you look at it's the iPhone 17 family that's driving it. That is, as you point out, is despite the supply constraints that we're experiencing. It's the things that are driving people to the 17 are people love the design, people love the performance, they love the durability, they love the camera, they love Center Stage, and they love that Apple Intelligence is integrated across the platform. From a where we're seeing the growth, it is amazing. We're seeing double-digit growth in the majority of the markets we track, from the U.S. to Latin America, to Greater China, to Western Europe, to India, to Japan, to Southeast Asia. We set a new March quarter record for upgraders as well.
[2876.9s] Tim Cook: You know, what's driving all this is that the customer satisfaction for the iPhone 17 family in the U.S., as an example, is 99%. These numbers are just unheard of. We're thrilled with how things are going.
[2895.3s] Amit Daryanani: Perfect. Thank you. Tim, I think we have you for 1 more earnings call, but I would really appreciate if you could kind of share a bit about the upcoming transition. You know, you have historically, I think, talked about the advice that Steve gave you when you took over, and I might be paraphrasing this, but it was around, "Don't ask what I would do, just do the right thing." That's really been a big win, I think, for Apple and shareholders over the last 15 years. Would love to understand, what advice are you giving John to help him build on Apple's strengths while shaping up the next chapter for the company? Thank you.
[2923.8s] Tim Cook: Well, I think Steve's advice to me lifted a huge burden. So that advice did well, for me and over the 15 years. For John, I think my advice is that, or what I've told him is that one of the most important decisions he'll make is where to spend his time. I would spend it where the greatest benefit to the company and the users are. Never forget the North Star for the company. You know, we're about making the best products in the world that really enrich other people's lives. If you keep focusing on that and make your decisions around that, it will produce a great business, and we'll be able to build more products and do it all over again. Thank you for the question.
[2977.0s] Amit Daryanani: Thank you.
[2978.1s] Suhasini Chandramouli: Thanks, Amit. Operator, could we get the next question, please?
[2984.2s] Operator: Our next question is from David Vogt with UBS. Please go ahead.
[2990.3s] David Vogt: Great. Thanks, guys, for taking my question. Maybe, Tim, I wanna come back to the supply chain for a second. I don't think I heard you state in your prepared remarks or in response to a question if the iPhone is constrained in the June quarter. Can you walk through kind of how you're thinking about your ability to secure not just SOC, but also memory? Are you thinking about using alternative sources of memory outside of sort of the traditional partners that you have? Just what's kind of driving that confidence that the iPhone isn't constrained given the amount of share it sounds like you're taking in that market? I have a follow-up as well.
[3024.9s] Tim Cook: Yeah. David, the constraint in the March quarter and the June quarter, the primary constraint is the availability of the advanced nodes our SOCs are produced on, not memory. I don't want to predict our ability to for supply and demand to match because if I look at it realistically, I think on the Mac mini and the Mac Studio, I believe it will take several months to reach supply-demand balance. We're not at the point where we're saying this is going to end anytime soon. It's not because of a problem per se, other than we just under called the demand.
[3083.8s] Tim Cook: You know, there are lead time to this, as you well understand, it takes a while to correct that. The primary constraint from a product point of view in the or the majority of it for this quarter, for the June quarter, will be on the Mac. It's Mac mini, Mac Studio, and the MacBook Neo. It's all of those.
[3113.8s] David Vogt: Great.
[3114.6s] Tim Cook: Yeah. Thank you.
[3115.4s] David Vogt: Maybe just on services real quick. You know, obviously, you know, real-relatively strong gross margins yet again. Are we getting to a point, given sort of the product mix within services, I know a lot of different offerings are growing double digits, that we're sort of asymptotically getting to a level where, we're seeing, you know, increasingly more challenging to scale that business from a profitability perspective? Or is there still sort of low-hanging fruit in terms of volume leverage in some of the offerings, or maybe lower losses in some different categories that can continue to scale gross margin across the services base? Thanks.
[3152.1s] Kevan Parekh: Yeah, David, it's Kevan. Thanks for the question. Look, as you know, our services portfolio contains a wide range of businesses that have different, you know, business models and profitability profiles and also are growing at different rates. At any given time, right, the relative performance of those can impact the gross margin. This time in particular, we look at the, you know, Q2 services margin. We talked about the fact that it increased 20 basis points sequentially. That's primarily driven by mix. Again, I think it's hard to speculate, you know, how that evolves over time. You know, we're encouraged by what we're seeing. We do have some services that are improving in profitability as they gain scale.
[3186.1s] Kevan Parekh: Again, I think we have a wide portfolio that has different characteristics and can grow, you know, at different rates at different times. Overall, we're encouraged by the overall trajectory that we've seen.
[3197.6s] David Vogt: Great. Thank you, guys.
[3198.8s] Suhasini Chandramouli: All right. Thank you, David. Operator, could we get the next question, please?
[3205.5s] Operator: Our next question is from Samik Chatterjee with JP Morgan. Please go ahead.
[3211.1s] Samik Chatterjee: Hi, thanks for taking my questions. Tim, for my first question, last quarter you did talk about Apple foundational models and sort of the two-pronged strategy there of the collaboration with Google as well as continuing to internally sort of work on your own models. Hoping you can sort of give us an update in terms of how you're able to balance those two priorities, as well as do you feel like you need to double down, invest more to be able to balance those two priorities side by side? Kind of a follow-up.
[3240.4s] Tim Cook: Yeah, it's a good question. We are investing more. You can see that in the OpEx numbers. As I'd mentioned before, the R&D in particular has scaled rather significantly on a year-over-year basis. The collaboration with Google is going well. We're happy with where things are, and we're happy with the work that we're doing independently as well.
[3266.8s] Samik Chatterjee: Okay, great. My follow-up for Kevan. Kevan, the sequential moderation in the product gross margin this year is relatively muted compared to what you've historically seen, at least over the last couple of years. Is it primarily mix, or what was the, maybe the FX tailwind as well? How would we sort of break it down in terms of what was different this year relative to what we typically see? If you could sort of also clarify what the FX impact on gross margin was for the quarter. Thank you.
[3295.5s] Kevan Parekh: Sure, Samik. Well, let me start. On products, for Q2, basically products gross margin did decrease by 200 basis points sequentially, driven by, you know, seasonal loss of leverage and higher memory costs, as Tim had alluded. If I zoom out, though, I think it's important just to look at what drove the overall company gross margin performance, and let me just give you a quick kinda rundown of that. If you look at our overall performance, right, our sequential gross margin impact was 110 basis points positively, and that was driven by favorable mix, lower tariff-related costs, and that was partly offset by seasonal loss of leverage and higher memory costs.
[3332.6s] Kevan Parekh: I did wanna turn it over to Tim, 'cause we do wanna provide some clarity around the lower tariff-related costs, and just make a comment on that as well.
[3340.2s] Tim Cook: Yeah, thanks, Kevan. For the March quarter, the gross margin of 49.3% did include the impact of tariff-related costs. However, tariffs in the March quarter versus the December quarter were lower because we had lower product volume, as you know, sequentially from Q1 to Q2. There was the full quarter benefit from a reduction in the IEEPA tariff rates, as well as the reduced global tariff rate under Section 122. In terms of applying for a refund of tariffs paid, we're following the established processes, and we plan to reinvest any amount we receive back into U.S. innovation and advanced manufacturing. These would be new investments and would be in addition to our prior commitments in the U.S.
[3400.8s] Kevan Parekh: 1 last point on your FX question. We really didn't see any sequential impact related to foreign exchange as a factor going from Q1 gross margin to Q2.
[3412.4s] Samik Chatterjee: Thank you.
[3413.1s] Kevan Parekh: Mm.
[3413.7s] Suhasini Chandramouli: All right. Thank you. Operator, could we get the last question, please?
[3421.0s] Operator: We'll go ahead and take our last question from Aaron Rakers with Wells Fargo. Please go ahead.
[3427.4s] Aaron Rakers: Yeah, thanks for taking the question, and congrats on the quarter. I wanted to ask about a few of the end markets. I guess particularly, Tim, if you could comment a little bit on what you're seeing specifically in China. I guess from a competitive perspective, are you seeing advantages from supply constraints impacting some of your competitors? Any thoughts on the China market, and I do have a quick follow-up.
[3450.7s] Tim Cook: Yeah. We are thrilled with the performance in Greater China. The first half of the year grew at 33%. In the March quarter, revenue was up 28%. It's a quarterly revenue record for us. The performance is really driven by iPhone, which was also a March quarter record. If you look at the individual products, iPhone was the top-selling model in urban China. The Mac mini was the top-selling desktop in China, and the MacBook Air was the top-selling laptop model. We're really doing well, pretty well across the board there. I was over there in March. The traffic in our stores grew by double digit. We were celebrating the Apple's fiftieth anniversary there, and it was just amazing to be a part of the community there.
[3512.7s] Tim Cook: I'm really happy with how things have gone the first half of this year.
[3520.0s] Aaron Rakers: Yeah. Maybe I'll stick with a similar theme, kind of the same question on the India market. It seems like that continues to be a focal point on these last several quarterly conference calls. I mean, how are you seeing the market in India evolve around, you know, the base of iPhones and the opportunity of kind of a rising middle class, just the overall opportunity set in that large mobile market?
[3545.1s] Tim Cook: Yeah. I think it's a huge opportunity for us. You know, we've been focused on this for a while. It's the second-largest smartphone market in the world and the third-largest PC market. Despite doing extremely well there for quite some time, we still have a modest share. I think there's that really speaks to the opportunity that we have. There are a lot of people moving into the middle class there, and we've got some great products for them, both currently and coming. If you look at the majority of customers on all of our categories, from the iPhone to the Mac to the iPad to the Watch, are new to that product there. It speaks very well to growing the install base there.
[3599.3s] Tim Cook: Net-net, I'm over the moon excited about India.
[3606.3s] Aaron Rakers: Thank you.
[3607.2s] Tim Cook: Yep.
[3608.3s] Suhasini Chandramouli: Thank you, Aaron Rakers. A replay of today's call will be available for 2 weeks on Apple Podcasts, as a webcast on apple.com/investor, and via telephone. The number for the telephone replay is 866-583-1035. Please enter confirmation code 2803309, followed by the pound sign. These replays will be available by approximately 5:00 P.M. Pacific Time today. Members of the press with additional questions can contact Josh Rosenstock at 408-862-1142. Financial analysts can contact me, Suhasini Chandramouli, with additional questions at 408-974-3123. Thanks again for joining us today.
[3654.3s] Operator: Once again, this does conclude today's conference. We do appreciate your participation.
2026 Q2 - Q2 2026 Earnings Call
2026 Q1 - Q1 2026 Earnings Call
2025 Q4 - Q4 2025 Earnings Call
2025 Q3 - Q3 2025 Earnings Call
2025 Q2 - Q2 2025 Earnings Call
2025 Q1 - Q1 2025 Earnings Call
2024 Q4 - Q4 2024 Earnings Call
2024 Q3 - Q3 2024 Earnings Call
2024 Q2 - Q2 2024 Earnings Call
2024 Q1 - Q1 2024 Earnings Call
2023 Q4 - Q4 2023 Earnings Call
2023 Q3 - Q3 2023 Earnings Call
2023 Q2 - Q2 2023 Earnings Call
2023 Q1 - Q1 2023 Earnings Call
2022 Q4 - Q4 2022 Earnings Call
2022 Q3 - Q3 2022 Earnings Call
2022 Q2 - Q2 2022 Earnings Call
2022 Q1 - Q1 2022 Earnings Call
2021 Q4 - Q4 2021 Earnings Call
2021 Q3 - Q3 2021 Earnings Call
2021 Q2 - Q2 2021 Earnings Call
2021 Q1 - Q1 2021 Earnings Call
2020 Q4 - Q4 2020 Earnings Call
2020 Q3 - Q3 2020 Earnings Call
2020 Q2 - Q2 2020 Earnings Call
2020 Q1 - Q1 2020 Earnings Call
2019 Q4 - Q4 2019 Earnings Call
2019 Q3 - Q3 2019 Earnings Call
2019 Q2 - Q2 2019 Earnings Call
2019 Q1 - Q1 2019 Earnings Call
2018 Q4 - Q4 2018 Earnings Call
2018 Q3 - Q3 2018 Earnings Call
2018 Q2 - Q2 2018 Earnings Call
2018 Q1 - Q1 2018 Earnings Call
2017 Q4 - Q4 2017 Earnings Call
2017 Q3 - Q3 2017 Earnings Call
2017 Q2 - Q2 2017 Earnings Call
2017 Q1 - Q1 2017 Earnings Call
2016 Q4 - Q4 2016 Earnings Call
2016 Q3 - Q3 2016 Earnings Call
2016 Q2 - Q2 2016 Earnings Call
2016 Q1 - Q1 2016 Earnings Call
2015 Q4 - Q4 2015 Earnings Call
2015 Q3 - Q3 2015 Earnings Call
2015 Q2 - Q2 2015 Earnings Call
2015 Q1 - Q1 2015 Earnings Call
2014 Q4 - Q4 2014 Earnings Call
2014 Q3 - Q3 2014 Earnings Call
2014 Q2 - Q2 2014 Earnings Call
2014 Q1 - Q1 2014 Earnings Call
2013 Q4 - Q4 2013 Earnings Call
2013 Q3 - Q3 2013 Earnings Call
2013 Q2 - Q2 2013 Earnings Call
2013 Q1 - Q1 2013 Earnings Call
2012 Q4 - Q4 2012 Earnings Call
2012 Q3 - Q3 2012 Earnings Call
2012 Q2 - Q2 2012 Earnings Call
2012 Q1 - Q1 2012 Earnings Call
2011 Q4 - Q4 2011 Earnings Call
2011 Q3 - Q3 2011 Earnings Call
2011 Q2 - Q2 2011 Earnings Call
```

## Caching Behavior

Understanding how Ticker caches data is important for efficient usage.

### In-Memory Cache

Caching is **on by default** and lasts as long as the `Ticker` handle lives. Fetched data is stored in an `Arc<RwLock<...>>` cache inside the `Ticker`.

Use `.cache(Duration)` to bound how long a response is reused, or `.no_cache()` to fetch fresh on every call.

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;
    use std::time::Duration;

    // Default: cached for the lifetime of the handle
    let ticker = Ticker::new("AAPL").await?;
    println!("default: {}", ticker.symbol());

    // Bounded: entries expire after 5 minutes
    let ticker = Ticker::builder("AAPL")
        .cache(Duration::from_secs(300))
        .build()
        .await?;
    println!("bounded: {}", ticker.symbol());

    // Off: every accessor issues a fresh request
    let ticker = Ticker::builder("AAPL").no_cache().build().await?;
    println!("no_cache: {}", ticker.symbol());
    Ok(())
}
```

```text soothfast-output
default: AAPL
bounded: AAPL
no_cache: AAPL
```

### Quote Summary Modules

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;

    // First access to ANY quote module -> 1 API call fetching ALL ~30 modules
    let price = ticker.price().await?;

    // All subsequent module accesses -> 0 API calls (same Ticker instance)
    let financial_data = ticker.financial_data().await?; // no network
    let profile = ticker.asset_profile().await?; // no network
    let stats = ticker.key_stats().await?; // no network

    println!(
        "price={} financial_data={} asset_profile={} key_stats={}",
        price.is_some(),
        financial_data.is_some(),
        profile.is_some(),
        stats.is_some(),
    );
    Ok(())
}
```

```text soothfast-output
price=true financial_data=true asset_profile=true key_stats=true
```

### Chart Data

Charts are cached separately per `(interval, range)` combination:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await?;

    // First call -> 1 API call
    let daily_1mo = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    // Same interval+range -> cached (0 API calls)
    let daily_1mo_again = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    // Different interval or range -> new API call
    let hourly_1mo = ticker.chart(Interval::OneHour, TimeRange::OneMonth).await?;
    let daily_3mo = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    println!("daily_1mo candles: {}", daily_1mo.candles.len());
    println!(
        "daily_1mo == daily_1mo_again: {}",
        daily_1mo.candles.len() == daily_1mo_again.candles.len()
    );
    println!("hourly_1mo candles: {}", hourly_1mo.candles.len());
    println!("daily_3mo candles: {}", daily_3mo.candles.len());
    Ok(())
}
```

```text soothfast-output
daily_1mo candles: 20
daily_1mo == daily_1mo_again: true
hourly_1mo candles: 141
daily_3mo candles: 62
```

### Financials and Options

Financials are cached per `(statement_type, frequency)` combination:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::{Frequency, StatementType, Ticker};

    let ticker = Ticker::new("AAPL").await?;

    // First call -> 1 API call
    let income_annual = ticker
        .financials(StatementType::Income, Frequency::Annual)
        .await?;

    // Same parameters -> cached
    let income_annual_again = ticker
        .financials(StatementType::Income, Frequency::Annual)
        .await?;

    // Different parameters -> new API call
    let income_quarterly = ticker
        .financials(StatementType::Income, Frequency::Quarterly)
        .await?;
    let balance_annual = ticker
        .financials(StatementType::Balance, Frequency::Annual)
        .await?;

    println!("income_annual metrics: {}", income_annual.statement.len());
    println!(
        "income_annual == income_annual_again: {}",
        income_annual.statement.len() == income_annual_again.statement.len()
    );
    println!(
        "income_quarterly metrics: {}",
        income_quarterly.statement.len()
    );
    println!("balance_annual metrics: {}", balance_annual.statement.len());
    Ok(())
}
```

```text soothfast-output
income_annual metrics: 30
income_annual == income_annual_again: true
income_quarterly metrics: 26
balance_annual metrics: 39
```

Options are cached per expiration date:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;

    // First call -> 1 API call
    let current_options = ticker.options(None).await?;

    // Same date -> cached
    let current_again = ticker.options(None).await?;

    // Different date -> new API call
    let future_options = ticker.options(Some(1735689600)).await?;

    println!(
        "current contracts: {}",
        current_options.calls().len() + current_options.puts().len()
    );
    println!(
        "current == current_again: {}",
        current_options.expiration_dates() == current_again.expiration_dates()
    );
    println!(
        "future contracts: {}",
        future_options.calls().len() + future_options.puts().len()
    );
    Ok(())
}
```

```text soothfast-output
current contracts: 87
current == current_again: true
future contracts: 0
```

### News and Recommendations

These are fetched once per Ticker instance and cached:

```rust capture-output
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await?;
    let news = ticker.news().await?;
    let news_again = ticker.news().await?; // cached

    let recs = ticker.recommendations(10).await?;
    let recs_again = ticker.recommendations(10).await?; // cached

    println!("news articles: {}", news.len());
    println!("news == news_again: {}", news.len() == news_again.len());
    println!("recommendations: {}", recs.recommendations.len());
    println!(
        "recs == recs_again: {}",
        recs.recommendations.len() == recs_again.recommendations.len()
    );
    Ok(())
}
```

```text soothfast-output
news articles: 20
news == news_again: true
recommendations: 10
recs == recs_again: true
```

### Best Practices

!!! tip "Optimize Performance with Caching"
    - **Reuse Ticker instances** across multiple queries to benefit from caching
    - **Request the data you need upfront** - accessing one quote module fetches them all anyway
    - **Be strategic with chart requests** - each new `(interval, range)` pair triggers a new request

    ```rust no_run
    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        use finance_query::{Ticker, Interval, TimeRange};
        use finance_query::format::Raw;

        // Good: Reuse ticker for multiple operations
        let ticker = Ticker::builder("AAPL").logo().build().await?;
        let quote = ticker.quote::<Raw>().await?;
        let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
        let profile = ticker.asset_profile().await?;

        // Less efficient: Creating new tickers each time
        // (loses caching benefits, re-authenticates with Yahoo each time)
        let ticker1 = Ticker::builder("AAPL").logo().build().await?;
        let quote = ticker1.quote::<Raw>().await?;
        let ticker2 = Ticker::new("AAPL").await?;
        let chart = ticker2.chart(Interval::OneDay, TimeRange::OneMonth).await?;
        Ok(())
    }
    ```

## Next Steps

- [Technical Indicators](indicators.md) - Access 42 indicators + candlestick patterns for analysis
- [Backtesting](backtesting.md) - Test trading strategies against historical data
- [Risk Analytics](risk.md) - VaR, Sharpe/Sortino/Calmar, beta, and drawdown (requires `risk` feature)
- [Batch Tickers](tickers.md) - Efficient operations for multiple symbols
- [DataFrame Support](dataframe.md) - Convert responses to Polars DataFrames for analysis
- [Configuration](configuration.md) - Customize language, region, and network settings
