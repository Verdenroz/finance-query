# Ticker API Reference

The `Ticker` struct is the primary interface for fetching symbol-specific data from Yahoo Finance. It provides lazy-loaded, cached access to quotes, charts, financials, and more.

!!! tip "Multiple Symbols"
    Need to fetch data for multiple symbols? Use the [`Tickers`](tickers.md) struct for efficient batch operations.

## Creation

### Simple Construction

```rust
use finance_query::Ticker;

let ticker = Ticker::new("AAPL").await?;
```

### Builder Pattern

For advanced configuration, use the builder:

```rust
use finance_query::{Ticker, Region};
use std::time::Duration;

// Using region enum (recommended - sets lang and region code correctly)
let ticker = Ticker::builder("2330.TW")
    .region(Region::Taiwan)
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

// Manual configuration
let ticker = Ticker::builder("AAPL")
    .lang("en-US")
    .region_code("US")
    .timeout(Duration::from_secs(20))
    .proxy("http://proxy.example.com:8080")
    .build()
    .await?;
```

**Builder Methods:**

- `.region(Region)` - Set region (automatically configures lang and region_code)
- `.lang(String)` - Set language code (e.g., "en-US", "de-DE", "zh-TW")
- `.region_code(String)` - Set region code (e.g., "US", "JP")
- `.timeout(Duration)` - Set HTTP request timeout
- `.proxy(String)` - Set proxy URL

See [Configuration](configuration.md) for details on available regions and settings.

## Quote Data

### Aggregated Quote

Get a comprehensive quote with all key metrics:

```rust
// Get quote with logo URL
let quote = ticker.quote(true).await?;

println!("Symbol: {}", quote.symbol);
println!("Name: {}", quote.short_name);
println!("Price: ${:.2}", quote.regular_market_price);
println!("Change: {:+.2} ({:+.2}%)",
    quote.regular_market_change,
    quote.regular_market_change_percent
);
println!("Market Cap: ${}", quote.market_cap);
println!("Logo: {:?}", quote.logo_url);
println!("Company Logo: {:?}", quote.company_logo_url);
```

The `Quote` struct aggregates data from multiple `quote modules` into a single structure.

### Quote Modules

Access specific quote modules directly. All modules are fetched together on first access and cached:

```rust
// First access triggers ONE API call for ALL modules
let price = ticker.price().await?;
if let Some(p) = price {
    println!("Market State: {}", p.market_state);
    println!("Currency: {}", p.currency);
}

// Subsequent calls use cached data (no network request)
let financial_data = ticker.financial_data().await?;
if let Some(fd) = financial_data {
    println!("Revenue: ${}", fd.total_revenue);
    println!("Profit Margin: {:.2}%", fd.profit_margins * 100.0);
    println!("EPS: ${:.2}", fd.trailing_eps);
}

let profile = ticker.asset_profile().await?;
if let Some(prof) = profile {
    println!("Sector: {}", prof.sector);
    println!("Industry: {}", prof.industry);
    println!("Website: {}", prof.website);
    println!("Description: {}", prof.long_business_summary);
}
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

```rust
let ticker = Ticker::new("MSFT").await?;

// Get financial health
if let Some(fd) = ticker.financial_data().await? {
    println!("Financials:");
    println!("  Revenue: ${:.2}B", fd.total_revenue / 1e9);
    println!("  Profit Margin: {:.2}%", fd.profit_margins * 100.0);
    println!("  ROE: {:.2}%", fd.return_on_equity * 100.0);
    println!("  Debt to Equity: {:.2}", fd.debt_to_equity);
}

// Get valuation
if let Some(sd) = ticker.summary_detail().await? {
    println!("\nValuation:");
    println!("  P/E Ratio: {:.2}", sd.trailing_pe);
    println!("  Forward P/E: {:.2}", sd.forward_pe);
    println!("  PEG Ratio: {:.2}", sd.peg_ratio);
}

// Get analyst sentiment
if let Some(rt) = ticker.recommendation_trend().await? {
    if let Some(latest) = rt.trend.first() {
        println!("\nAnalyst Recommendations:");
        println!("  Strong Buy: {}", latest.strong_buy);
        println!("  Buy: {}", latest.buy);
        println!("  Hold: {}", latest.hold);
        println!("  Sell: {}", latest.sell);
        println!("  Strong Sell: {}", latest.strong_sell);
    }
}
```

## Historical Data

### Chart (OHLCV) Data

Get historical candlestick data:

```rust
use finance_query::{Interval, TimeRange};

// Daily candles for the past month
let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

println!("Symbol: {}", chart.meta.symbol);
println!("Currency: {}", chart.meta.currency);
println!("Exchange: {}", chart.meta.exchange_name);
println!("Timezone: {}", chart.meta.timezone);

for candle in &chart.candles {
    println!(
        "{}: O=${:.2}, H=${:.2}, L=${:.2}, C=${:.2}, V={}",
        candle.timestamp, candle.open, candle.high, candle.low, candle.close, candle.volume
    );
}
```

**Available Intervals:**

- Intraday: `OneMinute`, `FiveMinutes`, `FifteenMinutes`, `ThirtyMinutes`, `OneHour`
- Daily and above: `OneDay`, `OneWeek`, `OneMonth`, `ThreeMonths`

**Available Time Ranges:**

- `OneDay`, `FiveDays`, `OneMonth`, `ThreeMonths`, `SixMonths`
- `OneYear`, `TwoYears`, `FiveYears`, `TenYears`
- `YearToDate`, `Max`

**Chart Structure:**
```rust
pub struct Chart {
    pub meta: ChartMeta,        // Metadata (symbol, currency, timezone, etc.)
    pub candles: Vec<Candle>,   // OHLCV candles
}

pub struct Candle {
    pub timestamp: i64,         // Unix timestamp
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

```rust
use chrono::{DateTime, Utc};

let dividends = ticker.dividends(TimeRange::TwoYears).await?;

for div in &dividends {
    let dt = DateTime::from_timestamp(div.timestamp, 0).unwrap();
    println!("{}: ${:.2} dividend", dt.format("%Y-%m-%d"), div.amount);
}
```

#### Stock Splits

```rust
use chrono::{DateTime, Utc};

let splits = ticker.splits(TimeRange::Max).await?;

for split in &splits {
    let dt = DateTime::from_timestamp(split.timestamp, 0).unwrap();
    println!("{}: {} split (ratio: {})",
        dt.format("%Y-%m-%d"), split.split_ratio, split.numerator / split.denominator
    );
}
```

#### Capital Gains

Distributions of capital gains (common for ETFs and mutual funds):

```rust
use chrono::{DateTime, Utc};

let gains = ticker.capital_gains(TimeRange::FiveYears).await?;

for gain in &gains {
    let dt = DateTime::from_timestamp(gain.timestamp, 0).unwrap();
    println!("{}: ${:.4} per share", dt.format("%Y-%m-%d"), gain.amount);
}
```

### Technical Indicators

Get pre-calculated technical indicators:

```rust
use finance_query::{Interval, TimeRange};

let indicators = ticker.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;

// Moving Averages
if let Some(sma20) = &indicators.sma_20 {
    println!("SMA(20): {:.2}", sma20.last().unwrap());
}
if let Some(ema50) = &indicators.ema_50 {
    println!("EMA(50): {:.2}", ema50.last().unwrap());
}

// RSI
if let Some(rsi14) = &indicators.rsi_14 {
    let rsi = rsi14.last().unwrap();
    println!("RSI(14): {:.2}", rsi);
    if rsi < 30.0 {
        println!("  -> Oversold");
    } else if rsi > 70.0 {
        println!("  -> Overbought");
    }
}

// MACD
if let Some(macd) = &indicators.macd {
    let last = macd.last().unwrap();
    println!("MACD: {:.4}", last.macd);
    println!("Signal: {:.4}", last.signal);
    println!("Histogram: {:.4}", last.histogram);

    if last.macd > last.signal {
        println!("  -> Bullish crossover");
    }
}

// Bollinger Bands
if let Some(bb) = &indicators.bollinger_bands {
    let last = bb.last().unwrap();
    println!("Bollinger Bands:");
    println!("  Upper: {:.2}", last.upper);
    println!("  Middle: {:.2}", last.middle);
    println!("  Lower: {:.2}", last.lower);
}

// Stochastic Oscillator
if let Some(stoch) = &indicators.stochastic {
    let last = stoch.last().unwrap();
    println!("Stochastic %K: {:.2}, %D: {:.2}", last.k, last.d);
}

// ATR (Average True Range)
if let Some(atr) = &indicators.atr_14 {
    println!("ATR(14): {:.2}", atr.last().unwrap());
}
```

**Available Indicators (52 total):**

**Moving Averages (21):**

- SMA: 10, 20, 50, 100, 200
- EMA: 10, 20, 50, 100, 200
- WMA: 10, 20, 50, 100, 200
- DEMA: 20
- TEMA: 20
- HMA: 20
- VWMA: 20
- ALMA: 9
- McGinley Dynamic: 20

**Momentum Oscillators (10):**

- RSI: 14
- Stochastic: 14, 3
- Stochastic RSI: 14, 14
- CCI: 20
- Williams %R: 14
- ROC: 12
- Momentum: 10
- CMO: 14
- Awesome Oscillator
- Coppock Curve

**Trend Indicators (8):**

- MACD: 12, 26, 9
- ADX: 14
- Aroon: 25
- SuperTrend: 10, 3.0
- Ichimoku Cloud
- Parabolic SAR
- Bull Bear Power
- Elder Ray Index

**Volatility (6):**

- Bollinger Bands: 20, 2.0
- ATR: 14
- Keltner Channels: 20, 10, 2.0
- Donchian Channels: 20
- True Range
- Choppiness Index: 14

**Volume (7):**

- OBV (On-Balance Volume)
- MFI: 14
- CMF: 20
- Chaikin Oscillator
- Accumulation/Distribution
- VWAP
- Balance of Power

All indicators are `Option<Vec<T>>` - use pattern matching to handle cases where data isn't available.

## Recommendations

Get similar stocks and analyst recommendations:

```rust
let rec = ticker.recommendations(5).await?;

println!("Similar stocks to {}:", ticker.symbol());

for similar in &rec.recommendations {
    println!("  {} - {}", similar.symbol, similar.score);
}
```

## Financial Statements

Get income statement, balance sheet, or cash flow statement:

```rust
use finance_query::{StatementType, Frequency};

// Annual income statement
let income = ticker.financials(
    StatementType::Income,
    Frequency::Annual
).await?;

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
let balance = ticker.financials(
    StatementType::Balance,
    Frequency::Quarterly
).await?;

// Cash flow statement
let cashflow = ticker.financials(
    StatementType::CashFlow,
    Frequency::Annual
).await?;
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

```rust
// Get all available expiration dates and options
let options = ticker.options(None).await?;

println!("Available expiration dates:");
for exp in &options.expiration_dates {
    let date = chrono::NaiveDateTime::from_timestamp_opt(*exp, 0).unwrap();
    println!("  {}", date.format("%Y-%m-%d"));
}

// Calls and puts for the first expiration
if let Some(chain) = options.options.first() {
    println!("\nCalls:");
    for call in &chain.calls {
        println!("  Strike ${:.2}: ${:.2} (volume: {})",
            call.strike, call.last_price, call.volume
        );
    }

    println!("\nPuts:");
    for put in &chain.puts {
        println!("  Strike ${:.2}: ${:.2} (volume: {})",
            put.strike, put.last_price, put.volume
        );
    }
}

// Get options for a specific expiration date
let specific_exp = options.expiration_dates[1]; // Second expiration
let options_dated = ticker.options(Some(specific_exp)).await?;
```

## News

Get recent news for the symbol:

```rust
let news = ticker.news().await?;

for article in &news {
    println!("{}", article.title);
    println!("  Source: {}", article.publisher);
    println!("  Published: {}", article.provider_publish_time);
    println!("  URL: {}", article.link);
    println!();
}
```

## Earnings Transcripts

Get earnings call transcripts:

```rust
use finance_query::finance;

// Get latest transcript
let transcript = finance::earnings_transcript(&ticker.symbol(), None, None).await?;

println!("Transcript for {} - Q{} {}",
    transcript.symbol,
    transcript.quarter,
    transcript.year
);

for entry in &transcript.transcript {
    println!("[{}] {}: {}",
        entry.start_time,
        entry.speaker,
        entry.content
    );
}

// Get specific quarter transcript
let q1_2024 = finance::earnings_transcript(&ticker.symbol(), Some("Q1"), Some(2024)).await?;

// Get all available transcripts (metadata only)
let all_transcripts = finance::earnings_transcripts(&ticker.symbol(), None).await?;

for meta in &all_transcripts {
    println!("{} Q{} - {}", meta.year, meta.quarter, meta.title);
}
```

## Caching Behavior

Understanding how Ticker caches data is important for efficient usage:

### Quote Summary Modules

```rust
let ticker = Ticker::new("AAPL").await?;

// First access to ANY quote module -> 1 API call fetching ALL ~30 modules
let price = ticker.price().await?;

// All subsequent module accesses -> 0 API calls (cached)
let financial_data = ticker.financial_data().await?;  // cached
let profile = ticker.asset_profile().await?;          // cached
let stats = ticker.key_stats().await?;                // cached
// ... all other modules are cached
```

### Chart Data

Charts are cached separately per `(interval, range)` combination:

```rust
// First call -> 1 API call
let daily_1mo = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

// Same interval+range -> cached (0 API calls)
let daily_1mo_again = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;

// Different interval or range -> new API call
let hourly_1mo = ticker.chart(Interval::OneHour, TimeRange::OneMonth).await?;  // 1 API call
let daily_3mo = ticker.chart(Interval::OneDay, TimeRange::ThreeMonths).await?; // 1 API call
```

### Financials and Options

Financials are cached per `(statement_type, frequency)` combination:

```rust
use finance_query::{StatementType, Frequency};

// First call -> 1 API call
let income_annual = ticker.financials(StatementType::Income, Frequency::Annual).await?;

// Same parameters -> cached (0 API calls)
let income_annual_again = ticker.financials(StatementType::Income, Frequency::Annual).await?;

// Different parameters -> new API call
let income_quarterly = ticker.financials(StatementType::Income, Frequency::Quarterly).await?;  // 1 API call
let balance_annual = ticker.financials(StatementType::Balance, Frequency::Annual).await?;     // 1 API call
```

Options are cached per expiration date:

```rust
// First call -> 1 API call
let current_options = ticker.options(None).await?;

// Same date -> cached (0 API calls)
let current_again = ticker.options(None).await?;

// Different date -> new API call
let future_options = ticker.options(Some(1735689600)).await?;  // 1 API call
```

### News and Recommendations

These are fetched once and cached:

```rust
// First call -> 1 API call
let news = ticker.news().await?;

// Second call -> cached (0 API calls)
let news_again = ticker.news().await?;

// Same for recommendations
let recs = ticker.recommendations(10).await?;  // 1 API call
let recs_again = ticker.recommendations(10).await?;  // cached
```

### Best Practices

!!! tip "Optimize Performance with Caching"
    - **Reuse Ticker instances** across multiple queries to benefit from caching
    - **Request the data you need upfront** - accessing one quote module fetches them all anyway
    - **Be strategic with chart requests** - each new `(interval, range)` pair triggers a new request

    ```rust
    // Good: Reuse ticker for multiple operations
    let ticker = Ticker::new("AAPL").await?;
    let quote = ticker.quote(true).await?;
    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let profile = ticker.asset_profile().await?;

    // Less efficient: Creating new tickers each time
    // (loses caching benefits, re-authenticates with Yahoo each time)
    let ticker1 = Ticker::new("AAPL").await?;
    let quote = ticker1.quote(true).await?;
    let ticker2 = Ticker::new("AAPL").await?;
    let chart = ticker2.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    ```

## Next Steps

- [Configuration](configuration.md) - Customize language, region, and network settings
- [DataFrame Support](dataframe.md) - Convert responses to Polars DataFrames for analysis
- [Models Reference](models.md) - Detailed documentation of all response types
