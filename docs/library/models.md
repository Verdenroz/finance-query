# Models Reference

This page documents all the data structures and response types in finance-query.

All structs are marked with `#[non_exhaustive]` and cannot be manually constructed. Obtain them through the appropriate `Ticker` or `finance` module methods.

## Chart Data

### Chart

The main structure for historical OHLCV data.

```rust
pub struct Chart {
    /// Stock symbol
    pub symbol: String,

    /// Chart metadata (exchange, currency, timezone, etc.)
    pub meta: ChartMeta,

    /// OHLCV candles/bars
    pub candles: Vec<Candle>,

    /// Time interval used (e.g., "1d", "1h")
    pub interval: Option<String>,

    /// Time range used (e.g., "1mo", "1y")
    pub range: Option<String>,
}
```

**Obtained via:**
```rust
let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
```

### Candle

A single OHLCV candle (candlestick bar).

```rust
pub struct Candle {
    /// Unix timestamp
    pub timestamp: i64,

    /// Open price
    pub open: f64,

    /// High price
    pub high: f64,

    /// Low price
    pub low: f64,

    /// Close price
    pub close: f64,

    /// Volume (signed integer)
    pub volume: i64,

    /// Adjusted close price (accounts for splits/dividends)
    pub adj_close: Option<f64>,
}
```

**Note:** Candle doesn't have a `date` field - only `timestamp`. Convert to date using chrono:

```rust
use chrono::{DateTime, Utc};

for candle in &chart.candles {
    let dt = DateTime::from_timestamp(candle.timestamp, 0).unwrap();
    println!("{}: ${:.2}", dt.format("%Y-%m-%d"), candle.close);
}
```

### ChartMeta

Metadata about the chart.

```rust
pub struct ChartMeta {
    pub symbol: String,
    pub currency: String,
    pub exchange_name: String,
    pub timezone: String,
    pub regular_market_price: Option<f64>,
    pub previous_close: Option<f64>,
    pub chart_previous_close: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    // ... and more fields
}
```

## Corporate Events

### Dividend

Dividend payment information.

```rust
pub struct Dividend {
    /// Unix timestamp
    pub timestamp: i64,

    /// Dividend amount per share
    pub amount: f64,
}
```

**Obtained via:**
```rust
let dividends = ticker.dividends(TimeRange::TwoYears).await?;
```

### Split

Stock split information.

```rust
pub struct Split {
    /// Unix timestamp
    pub timestamp: i64,

    /// Numerator of the split ratio
    pub numerator: f64,

    /// Denominator of the split ratio
    pub denominator: f64,

    /// Split ratio as string (e.g., "2:1", "10:1")
    pub ratio: String,
}
```

**Obtained via:**
```rust
let splits = ticker.splits(TimeRange::Max).await?;
```

### CapitalGain

Capital gains distribution (common for ETFs and mutual funds).

```rust
pub struct CapitalGain {
    /// Unix timestamp
    pub timestamp: i64,

    /// Capital gain amount per share
    pub amount: f64,
}
```

**Obtained via:**
```rust
let gains = ticker.capital_gains(TimeRange::FiveYears).await?;
```

## Quote Data

### Quote

The aggregated quote structure with all key metrics. This flattens data from multiple Yahoo Finance quote modules into a single convenient structure.

```rust
pub struct Quote {
    // Identity
    pub symbol: String,
    pub logo_url: Option<String>,
    pub company_logo_url: Option<String>,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub quote_type: Option<String>,
    pub currency: Option<String>,

    // Pricing (wrapped in FormattedValue)
    pub regular_market_price: Option<FormattedValue<f64>>,
    pub regular_market_change: Option<FormattedValue<f64>>,
    pub regular_market_change_percent: Option<FormattedValue<f64>>,
    pub regular_market_time: Option<i64>,
    pub regular_market_day_high: Option<FormattedValue<f64>>,
    pub regular_market_day_low: Option<FormattedValue<f64>>,
    pub regular_market_volume: Option<FormattedValue<i64>>,
    pub regular_market_previous_close: Option<FormattedValue<f64>>,
    pub regular_market_open: Option<FormattedValue<f64>>,

    // Extended hours (wrapped in FormattedValue)
    pub pre_market_price: Option<FormattedValue<f64>>,
    pub pre_market_change: Option<FormattedValue<f64>>,
    pub pre_market_change_percent: Option<FormattedValue<f64>>,
    pub post_market_price: Option<FormattedValue<f64>>,
    pub post_market_change: Option<FormattedValue<f64>>,
    pub post_market_change_percent: Option<FormattedValue<f64>>,

    // Valuation (wrapped in FormattedValue)
    pub market_cap: Option<FormattedValue<i64>>,
    pub trailing_pe: Option<FormattedValue<f64>>,
    pub forward_pe: Option<FormattedValue<f64>>,
    pub price_to_book: Option<FormattedValue<f64>>,
    pub enterprise_value: Option<FormattedValue<i64>>,

    // Range (wrapped in FormattedValue)
    pub fifty_two_week_low: Option<FormattedValue<f64>>,
    pub fifty_two_week_high: Option<FormattedValue<f64>>,
    pub fifty_day_average: Option<FormattedValue<f64>>,
    pub two_hundred_day_average: Option<FormattedValue<f64>>,

    // Dividends (wrapped in FormattedValue)
    pub dividend_rate: Option<FormattedValue<f64>>,
    pub dividend_yield: Option<FormattedValue<f64>>,
    pub ex_dividend_date: Option<FormattedValue<i64>>,

    // Shares (wrapped in FormattedValue)
    pub shares_outstanding: Option<FormattedValue<i64>>,
    pub float_shares: Option<FormattedValue<i64>>,

    // Financials (wrapped in FormattedValue)
    pub total_revenue: Option<FormattedValue<i64>>,
    pub revenue_per_share: Option<FormattedValue<f64>>,
    pub profit_margins: Option<FormattedValue<f64>>,
    pub operating_margins: Option<FormattedValue<f64>>,
    pub ebitda: Option<FormattedValue<i64>>,
    pub trailing_eps: Option<FormattedValue<f64>>,
    pub forward_eps: Option<FormattedValue<f64>>,

    // ... and many more optional fields
}
```

**Important:** Most numeric fields in Quote are wrapped in `FormattedValue<T>`, not plain Option<T>. The `FormattedValue<T>` struct contains:
- `raw: Option<T>` - The actual numeric value
- `fmt: Option<String>` - Formatted string (e.g., "150.25")
- `long_fmt: Option<String>` - Long format string (e.g., "150.25000")

Access numeric values using `.as_ref().and_then(|v| v.raw)`:
```rust
let price = quote.regular_market_price.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
```

**Field Precedence:** For duplicate fields across modules:
- Price → SummaryDetail → DefaultKeyStatistics → FinancialData → AssetProfile

**Obtained via:**
```rust
let quote = ticker.quote(true).await?;  // true = include logo URLs
```

### Quote Modules

Individual quote modules provide more detailed, structured data. All are `Option<T>` since Yahoo may not return every module for every symbol.

#### Price

```rust
pub struct Price {
    pub symbol: Option<String>,
    pub market_state: Option<String>,           // "REGULAR", "PRE", "POST", "CLOSED"
    pub currency: Option<String>,
    pub regular_market_price: Option<FormattedValue<f64>>,
    pub regular_market_time: Option<i64>,
    pub regular_market_change: Option<FormattedValue<f64>>,
    pub regular_market_change_percent: Option<FormattedValue<f64>>,
    // ... more pricing fields
}
```

**Note:** Most numeric fields are wrapped in `FormattedValue<T>` which contains `raw: Option<T>`, `fmt: Option<String>`, and `long_fmt: Option<String>`. Access the numeric value via `.as_ref().and_then(|v| v.raw)`.

**Obtained via:** `ticker.price().await?`

#### SummaryDetail

```rust
pub struct SummaryDetail {
    pub market_cap: Option<FormattedValue<i64>>,
    pub trailing_pe: Option<FormattedValue<f64>>,
    pub forward_pe: Option<FormattedValue<f64>>,
    pub peg_ratio: Option<Value>,  // Generic JSON value
    pub dividend_rate: Option<FormattedValue<f64>>,
    pub dividend_yield: Option<FormattedValue<f64>>,
    pub beta: Option<FormattedValue<f64>>,
    pub fifty_two_week_low: Option<FormattedValue<f64>>,
    pub fifty_two_week_high: Option<FormattedValue<f64>>,
    pub fifty_day_average: Option<FormattedValue<f64>>,
    pub two_hundred_day_average: Option<FormattedValue<f64>>,
    // ... more summary fields
}
```

**Note:** Numeric fields are wrapped in `FormattedValue<T>`. Access values via `.as_ref().and_then(|v| v.raw)`.

**Obtained via:** `ticker.summary_detail().await?`

#### FinancialData

```rust
pub struct FinancialData {
    pub total_cash: Option<FormattedValue<i64>>,
    pub total_cash_per_share: Option<FormattedValue<f64>>,
    pub ebitda: Option<FormattedValue<i64>>,
    pub total_debt: Option<FormattedValue<i64>>,
    pub total_revenue: Option<FormattedValue<i64>>,
    pub debt_to_equity: Option<FormattedValue<f64>>,
    pub revenue_per_share: Option<FormattedValue<f64>>,
    pub return_on_assets: Option<FormattedValue<f64>>,
    pub return_on_equity: Option<FormattedValue<f64>>,
    pub gross_profits: Option<FormattedValue<i64>>,
    pub free_cashflow: Option<FormattedValue<i64>>,  // Note: field name is 'free_cashflow'
    pub operating_cashflow: Option<FormattedValue<i64>>,  // Note: field name is 'operating_cashflow'
    pub earnings_growth: Option<FormattedValue<f64>>,
    pub revenue_growth: Option<FormattedValue<f64>>,
    pub gross_margins: Option<FormattedValue<f64>>,
    pub ebitda_margins: Option<FormattedValue<f64>>,
    pub operating_margins: Option<FormattedValue<f64>>,
    pub profit_margins: Option<FormattedValue<f64>>,
    pub current_price: Option<FormattedValue<f64>>,
    pub target_high_price: Option<FormattedValue<f64>>,
    pub target_low_price: Option<FormattedValue<f64>>,
    pub target_mean_price: Option<FormattedValue<f64>>,
    pub target_median_price: Option<FormattedValue<f64>>,
    pub recommendation_mean: Option<FormattedValue<f64>>,
    pub recommendation_key: Option<String>,
    pub number_of_analyst_opinions: Option<FormattedValue<i64>>,
    // ... more financial fields
}
```

**Note:** Numeric fields are wrapped in `FormattedValue<T>`. Access values via `.as_ref().and_then(|v| v.raw)`.

**Obtained via:** `ticker.financial_data().await?`

#### AssetProfile

```rust
pub struct AssetProfile {
    pub address1: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub sector: Option<String>,
    pub long_business_summary: Option<String>,
    pub full_time_employees: Option<i64>,
    pub company_officers: Option<Vec<CompanyOfficer>>,
    // ... more profile fields
}
```

**Obtained via:** `ticker.asset_profile().await?`

#### DefaultKeyStatistics

```rust
pub struct DefaultKeyStatistics {
    pub enterprise_value: Option<FormattedValue<i64>>,
    pub forward_pe: Option<FormattedValue<f64>>,
    pub profit_margins: Option<FormattedValue<f64>>,
    pub float_shares: Option<FormattedValue<i64>>,
    pub shares_outstanding: Option<FormattedValue<i64>>,
    pub shares_short: Option<FormattedValue<i64>>,
    pub shares_short_prior_month: Option<FormattedValue<i64>>,
    pub shares_short_previous_month_date: Option<FormattedValue<i64>>,
    pub date_short_interest: Option<FormattedValue<i64>>,
    pub shares_percent_shares_out: Option<FormattedValue<f64>>,
    pub held_percent_insiders: Option<FormattedValue<f64>>,
    pub held_percent_institutions: Option<FormattedValue<f64>>,
    pub short_ratio: Option<FormattedValue<f64>>,
    pub short_percent_of_float: Option<FormattedValue<f64>>,
    pub beta: Option<FormattedValue<f64>>,
    pub book_value: Option<FormattedValue<f64>>,
    pub price_to_book: Option<FormattedValue<f64>>,
    pub last_fiscal_year_end: Option<FormattedValue<i64>>,
    pub next_fiscal_year_end: Option<FormattedValue<i64>>,
    pub most_recent_quarter: Option<FormattedValue<i64>>,
    pub earnings_quarterly_growth: Option<FormattedValue<f64>>,
    pub trailing_eps: Option<FormattedValue<f64>>,
    pub forward_eps: Option<FormattedValue<f64>>,
    pub peg_ratio: Option<Value>,  // Generic JSON value
    pub last_split_factor: Option<String>,
    pub last_split_date: Option<FormattedValue<i64>>,
    // ... more stats
}
```

**Note:** Numeric fields are wrapped in `FormattedValue<T>`. Access values via `.as_ref().and_then(|v| v.raw)`.

**Obtained via:** `ticker.key_stats().await?`

#### CalendarEvents

```rust
pub struct CalendarEvents {
    pub earnings: Option<Earnings>,
    pub ex_dividend_date: Option<i64>,
    pub dividend_date: Option<i64>,
}
```

**Obtained via:** `ticker.calendar_events().await?`

#### RecommendationTrend

```rust
pub struct RecommendationTrend {
    pub trend: Vec<Trend>,
}

pub struct Trend {
    pub period: String,
    pub strong_buy: i32,
    pub buy: i32,
    pub hold: i32,
    pub sell: i32,
    pub strong_sell: i32,
}
```

**Obtained via:** `ticker.recommendation_trend().await?`

**See the [Ticker API Reference](ticker.md#quote-modules) for the complete list of 19 available quote modules.**

## Recommendations

### Recommendation

Similar/recommended symbols data.

```rust
pub struct Recommendation {
    /// Symbol that was queried
    pub symbol: String,

    /// Recommended/similar symbols with scores
    pub recommendations: Vec<SimilarSymbol>,
}
```

### SimilarSymbol

```rust
pub struct SimilarSymbol {
    pub symbol: String,
    pub score: f64,
}
```

**Obtained via:**
```rust
let rec = ticker.recommendations(5).await?;
for similar in &rec.recommendations {
    println!("{} - score: {}", similar.symbol, similar.score);
}
```

## Options Data

### Options

Options chain container with expiration dates and contracts.

```rust
pub struct Options {
    // Internal structure - use accessor methods
}

impl Options {
    /// Get available expiration dates (Unix timestamps)
    pub fn expiration_dates(&self) -> Vec<i64>;

    /// Get all option chains
    pub fn options(&self) -> Vec<OptionChain>;

    /// Get call contracts
    pub fn calls(&self) -> Vec<Vec<OptionContract>>;

    /// Get put contracts
    pub fn puts(&self) -> Vec<Vec<OptionContract>>;
}
```

**Obtained via:**
```rust
// Get all available expirations
let options = ticker.options(None).await?;

// Get specific expiration
let exp_date = options.expiration_dates().first().copied();
let options = ticker.options(exp_date).await?;
```

### OptionContract

Individual option contract data.

```rust
pub struct OptionContract {
    pub contract_symbol: String,
    pub strike: f64,
    pub currency: String,
    pub last_price: f64,
    pub change: f64,
    pub percent_change: f64,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
    pub bid: f64,
    pub ask: f64,
    pub contract_size: String,
    pub expiration: i64,
    pub last_trade_date: i64,
    pub implied_volatility: f64,
    pub in_the_money: bool,
}
```

## Financial Statements

### FinancialsResponse

Container for financial statement data.

```rust
pub struct FinancialsResponse {
    pub symbol: String,
    pub statement_type: StatementType,  // Income, Balance, or CashFlow
    pub frequency: Frequency,            // Annual or Quarterly
    pub statements: Vec<FinancialStatement>,
}
```

### FinancialStatement

A single financial statement with line items.

```rust
pub struct FinancialStatement {
    pub end_date: String,
    pub line_items: HashMap<String, f64>,
}

impl FinancialStatement {
    /// Get a specific value from the statement
    pub fn get_value(&self, key: &str) -> Option<f64>;
}
```

**Common line item keys:**

**Income Statement:**
- `TotalRevenue`
- `CostOfRevenue`
- `GrossProfit`
- `OperatingExpense`
- `OperatingIncome`
- `NetIncome`
- `EBITDA`
- `BasicEPS`
- `DilutedEPS`

**Balance Sheet:**
- `TotalAssets`
- `TotalLiabilitiesNetMinorityInterest`
- `TotalEquityGrossMinorityInterest`
- `CashAndCashEquivalents`
- `CurrentAssets`
- `CurrentLiabilities`
- `LongTermDebt`
- `StockholdersEquity`

**Cash Flow:**
- `OperatingCashFlow`
- `InvestingCashFlow`
- `FinancingCashFlow`
- `FreeCashFlow`
- `CapitalExpenditure`
- `EndCashPosition`

**Obtained via:**
```rust
use finance_query::{StatementType, Frequency};

let income = ticker.financials(
    StatementType::Income,
    Frequency::Annual
).await?;

for statement in &income.statements {
    println!("Year: {}", statement.end_date);
    if let Some(revenue) = statement.get_value("TotalRevenue") {
        println!("  Revenue: ${:.2}B", revenue / 1e9);
    }
}
```

## Technical Indicators

### IndicatorsSummary

Container for all technical indicators. Returns single latest values (not time series).

```rust
pub struct IndicatorsSummary {
    // === MOVING AVERAGES (21) ===
    // Simple Moving Averages
    pub sma_10: Option<f64>,
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub sma_100: Option<f64>,
    pub sma_200: Option<f64>,

    // Exponential Moving Averages
    pub ema_10: Option<f64>,
    pub ema_20: Option<f64>,
    pub ema_50: Option<f64>,
    pub ema_100: Option<f64>,
    pub ema_200: Option<f64>,

    // Weighted Moving Averages
    pub wma_10: Option<f64>,
    pub wma_20: Option<f64>,
    pub wma_50: Option<f64>,
    pub wma_100: Option<f64>,
    pub wma_200: Option<f64>,

    // Advanced Moving Averages
    pub dema_20: Option<f64>,
    pub tema_20: Option<f64>,
    pub hma_20: Option<f64>,
    pub vwma_20: Option<f64>,
    pub alma_9: Option<f64>,
    pub mcginley_dynamic_20: Option<f64>,

    // === MOMENTUM OSCILLATORS (10) ===
    pub rsi_14: Option<f64>,
    pub stochastic: Option<StochasticData>,
    pub stochastic_rsi: Option<StochasticData>,
    pub cci_20: Option<f64>,
    pub williams_r_14: Option<f64>,
    pub roc_12: Option<f64>,
    pub momentum_10: Option<f64>,
    pub cmo_14: Option<f64>,
    pub awesome_oscillator: Option<f64>,
    pub coppock_curve: Option<f64>,

    // === TREND INDICATORS (8) ===
    pub macd: Option<MacdData>,
    pub adx_14: Option<f64>,
    pub aroon: Option<AroonData>,
    pub supertrend: Option<SuperTrendData>,
    pub ichimoku: Option<IchimokuData>,
    pub parabolic_sar: Option<f64>,
    pub bull_bear_power: Option<BullBearPowerData>,
    pub elder_ray_index: Option<ElderRayData>,

    // === VOLATILITY INDICATORS (6) ===
    pub bollinger_bands: Option<BollingerBandsData>,
    pub atr_14: Option<f64>,
    pub keltner_channels: Option<KeltnerChannelsData>,
    pub donchian_channels: Option<DonchianChannelsData>,
    pub true_range: Option<f64>,
    pub choppiness_index_14: Option<f64>,

    // === VOLUME INDICATORS (7) ===
    pub obv: Option<f64>,
    pub mfi_14: Option<f64>,
    pub cmf_20: Option<f64>,
    pub chaikin_oscillator: Option<f64>,
    pub accumulation_distribution: Option<f64>,
    pub vwap: Option<f64>,
    pub balance_of_power: Option<f64>,
}
```

### MacdData

```rust
pub struct MacdData {
    pub macd: Option<f64>,
    pub signal: Option<f64>,
    pub histogram: Option<f64>,
}
```

### BollingerBandsData

```rust
pub struct BollingerBandsData {
    pub upper: Option<f64>,
    pub middle: Option<f64>,
    pub lower: Option<f64>,
}
```

### StochasticData

```rust
pub struct StochasticData {
    pub k: Option<f64>,  // %K line
    pub d: Option<f64>,  // %D line
}
```

### AroonData

```rust
pub struct AroonData {
    pub aroon_up: Option<f64>,
    pub aroon_down: Option<f64>,
}
```

### SuperTrendData

```rust
pub struct SuperTrendData {
    pub value: Option<f64>,
    pub trend: Option<String>,  // "up" or "down"
}
```

### IchimokuData

```rust
pub struct IchimokuData {
    pub conversion_line: Option<f64>,    // Tenkan-sen
    pub base_line: Option<f64>,          // Kijun-sen
    pub leading_span_a: Option<f64>,     // Senkou Span A
    pub leading_span_b: Option<f64>,     // Senkou Span B
    pub lagging_span: Option<f64>,       // Chikou Span
}
```

### KeltnerChannelsData

```rust
pub struct KeltnerChannelsData {
    pub upper: Option<f64>,
    pub middle: Option<f64>,
    pub lower: Option<f64>,
}
```

### DonchianChannelsData

```rust
pub struct DonchianChannelsData {
    pub upper: Option<f64>,
    pub middle: Option<f64>,
    pub lower: Option<f64>,
}
```

### BullBearPowerData

```rust
pub struct BullBearPowerData {
    pub bull_power: Option<f64>,
    pub bear_power: Option<f64>,
}
```

### ElderRayData

```rust
pub struct ElderRayData {
    pub bull_power: Option<f64>,
    pub bear_power: Option<f64>,
}
```

**Obtained via:**
```rust
let indicators = ticker.indicators(
    Interval::OneDay,
    TimeRange::ThreeMonths
).await?;

// Single values, not time series
if let Some(rsi) = indicators.rsi_14 {
    println!("RSI(14): {:.2}", rsi);
}

if let Some(macd) = &indicators.macd {
    if let (Some(macd_val), Some(signal)) = (macd.macd, macd.signal) {
        println!("MACD: {:.4}, Signal: {:.4}", macd_val, signal);
    }
}

if let Some(bb) = &indicators.bollinger_bands {
    if let (Some(upper), Some(middle), Some(lower)) = (bb.upper, bb.middle, bb.lower) {
        println!("BB: Upper={:.2}, Mid={:.2}, Lower={:.2}", upper, middle, lower);
    }
}
```

## News

### NewsArticle

News article information.

```rust
pub struct NewsArticle {
    pub uuid: String,
    pub title: String,
    pub publisher: String,
    pub link: String,
    pub provider_publish_time: i64,
    pub type_: String,
    pub thumbnail: Option<ThumbnailResolutions>,
    pub related_tickers: Option<Vec<String>>,
}
```

**Obtained via:**
```rust
let news = ticker.news().await?;
for article in &news {
    println!("{}", article.title);
    println!("  Source: {}", article.publisher);
    println!("  URL: {}", article.link);
}
```

## Search & Lookup

### SearchResults

Search results from the Yahoo Finance search endpoint.

```rust
pub struct SearchResults {
    pub quotes: Vec<SearchQuote>,
    pub news: Vec<NewsArticle>,
    // ... other result types
}
```

### LookupResults

Type-filtered lookup results.

```rust
pub struct LookupResults {
    pub documents: Vec<LookupDocument>,
}

pub struct LookupDocument {
    pub symbol: String,
    pub name: String,
    pub exchange: Option<String>,
    pub quote_type: Option<String>,
    // ... and optional logo URLs if requested
}
```

## Market Data

### MarketSummary

Summary of major market indices and commodities.

```rust
pub struct MarketSummary {
    pub market_summary_response: MarketSummaryResponse,
}

pub struct MarketSummaryResponse {
    pub result: Vec<MarketSummaryQuote>,
}
```

### TrendingResults

Trending symbols by region.

```rust
pub struct TrendingResults {
    pub count: usize,
    pub quotes: Vec<TrendingQuote>,
}
```

## Screener

### ScreenerResults

Results from stock screener queries.

```rust
pub struct ScreenerResults {
    pub finance: ScreenerFinance,
}

pub struct ScreenerFinance {
    pub result: Vec<ScreenerResult>,
}

pub struct ScreenerResult {
    pub id: String,
    pub title: String,
    pub description: String,
    pub canonical_name: String,
    pub count: usize,
    pub quotes: Vec<ScreenerQuote>,
}
```

## DataFrame Support

With the `dataframe` feature enabled, many types can be converted to Polars DataFrames:

```rust
// Enable in Cargo.toml:
// finance-query = { version = "2.0", features = ["dataframe"] }

// Convert chart candles to DataFrame
let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let df = chart.to_dataframe()?;

// Convert quote to DataFrame
let quote = ticker.quote(true).await?;
let df = quote.to_dataframe()?;

// Convert recommendations to DataFrame
let rec = ticker.recommendations(10).await?;
let df = rec.to_dataframe()?;
```

## Working with Optional Fields

Most fields in Yahoo Finance responses are `Option<T>` because data availability varies by symbol and asset type. Additionally, most **numeric** fields in quote-related structs (Quote, Price, SummaryDetail, FinancialData, DefaultKeyStatistics) are wrapped in `FormattedValue<T>`.

### Understanding FormattedValue

`FormattedValue<T>` provides both the raw numeric value and formatted strings:

```rust
pub struct FormattedValue<T> {
    pub raw: Option<T>,         // The actual number
    pub fmt: Option<String>,     // Short format: "150.25"
    pub long_fmt: Option<String>, // Long format: "150.250000"
}
```

### Safe Pattern Matching with FormattedValue

```rust
// Access numeric value from Quote fields
if let Some(market_cap) = quote.market_cap.as_ref().and_then(|v| v.raw) {
    println!("Market cap: ${}", market_cap);
}

// Access formatted string
if let Some(price_fmt) = quote.regular_market_price.as_ref().and_then(|v| v.fmt.as_ref()) {
    println!("Price (formatted): {}", price_fmt);
}

// For non-numeric optional fields (strings, etc.)
if let Some(profile) = ticker.asset_profile().await? {
    if let Some(sector) = profile.sector {
        println!("Sector: {}", sector);
    }
}
```

### Unwrapping Safely

```rust
// With FormattedValue fields - unwrap with default
let price = quote.regular_market_price
    .as_ref()
    .and_then(|v| v.raw)
    .unwrap_or(0.0);

// Non-optional fields
let symbol = quote.symbol; // symbol is NOT optional

// IndicatorsSummary fields are plain Option<f64>, not FormattedValue
if let Some(sma_20) = indicators.sma_20 {
    println!("SMA(20): {:.2}", sma_20);
}
```

### Helper Pattern for Multiple FormattedValues

```rust
// Extract raw values safely
fn get_raw<T: Copy>(fv: &Option<FormattedValue<T>>) -> Option<T> {
    fv.as_ref().and_then(|v| v.raw)
}

// Usage
let price = get_raw(&quote.regular_market_price).unwrap_or(0.0);
let market_cap = get_raw(&quote.market_cap).unwrap_or(0);
```

## Next Steps

- [Ticker API Reference](ticker.md) - Methods to obtain these models
- [Technical Indicators](indicators.md) - Indicator data structures and usage
- [Backtesting](backtesting.md) - Backtest result models and metrics
- [DataFrame Support](dataframe.md) - Convert models to Polars DataFrames for analysis
- [Configuration](configuration.md) - Configure regional settings and network options
