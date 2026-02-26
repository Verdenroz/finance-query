# Finance Module

The `finance` module provides market-wide operations that don't require a specific stock symbol. Use these functions to search for symbols, get market data, fetch screeners, and more.

!!! tip "Import the finance module"
    ```rust
    use finance_query::finance;
    ```

## Search & Discovery

### Search

Search for stocks, ETFs, funds, and other securities by name or symbol:

```rust
use finance_query::{finance, SearchOptions, Region};

// Simple search with defaults
let results = finance::search("Apple", &SearchOptions::default()).await?;
println!("Found {} results", results.result_count());

for quote in &results.quotes {
    let exchange = quote.exchange.as_deref().unwrap_or("N/A");
    let name = quote.short_name.as_deref().unwrap_or("N/A");
    println!("{} ({}): {}", quote.symbol, exchange, name);
}

// Advanced search with options
let options = SearchOptions::new()
    .quotes_count(10)
    .news_count(5)
    .enable_research_reports(true)
    .enable_fuzzy_query(true)
    .region(Region::Canada);

let results = finance::search("tesla", &options).await?;
```

**SearchOptions Methods:**

- `.quotes_count(u32)` - Number of quote results (default: 6)
- `.news_count(u32)` - Number of news results (default: 4)
- `.enable_fuzzy_query(bool)` - Enable fuzzy matching (default: true)
- `.enable_logo_url(bool)` - Include logo URLs (default: true)
- `.enable_research_reports(bool)` - Include research reports (default: false)
- `.enable_cultural_assets(bool)` - Include cultural assets (default: false)
- `.recommend_count(u32)` - Number of recommendations (default: 5)
- `.region(Region)` - Search region (default: US)

**SearchResults Fields:**

- `quotes` - Vec of matching quotes
- `news` - Vec of related news articles
- `research_reports` - Optional research reports
- `recommendations` - Recommended symbols

### Lookup

Lookup symbols filtered by asset type (equity, ETF, mutual fund, index, etc.):

```rust
use finance_query::{finance, LookupOptions, LookupType};

// Simple lookup
let results = finance::lookup("NVDA", &LookupOptions::default()).await?;

// Lookup equities with logos
let options = LookupOptions::new()
    .lookup_type(LookupType::Equity)
    .count(10)
    .include_logo(true);

let results = finance::lookup("tech", &options).await?;
for quote in &results.quotes {
    let name = quote.short_name.as_deref().unwrap_or("N/A");
    println!("{}: {} - {:?}", quote.symbol, name, quote.logo_url);
}
```

**Available LookupTypes:**

- `All` - All asset types (default)
- `Equity` - Stocks
- `Etf` - Exchange-traded funds
- `MutualFund` - Mutual funds
- `Index` - Market indices
- `Future` - Futures contracts
- `Currency` - Currencies
- `Cryptocurrency` - Cryptocurrencies

## Market Data

### Market Summary

Get current market summary with major indices, currencies, and commodities:

```rust
use finance_query::{finance, Region};

// Default (US market)
let summary = finance::market_summary(None).await?;

// Specific region
let summary = finance::market_summary(Some(Region::Canada)).await?;

for quote in &summary {
    let price = quote.regular_market_price.as_ref().and_then(|v| v.raw);
    let change_pct = quote.regular_market_change_percent.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
    println!("{}: ${:?} ({:+.2}%)", quote.symbol, price, change_pct);
}
```

### Trending

Get trending stocks for a region:

```rust
let trending = finance::trending(None).await?;
// Or specify region
let trending = finance::trending(Some(Region::Singapore)).await?;

for quote in &trending {
    println!("{}", quote.symbol);
}
```

### Market Hours

Check market status and trading hours:

```rust
// US market hours (default)
let hours = finance::hours(None).await?;

// Japan market hours
let hours = finance::hours(Some("JP")).await?;

for market in &hours.markets {
    println!("{}: {}", market.name, market.status);
    println!("  Open: {:?}", market.open);
    println!("  Close: {:?}", market.close);
}
```

### Indices

Get quotes for major world indices:

```rust
use finance_query::{finance, IndicesRegion};

// All world indices
let all = finance::indices(None).await?;
println!("Fetched {} indices", all.success_count());

// Only Americas indices (^DJI, ^GSPC, ^IXIC, etc.)
let americas = finance::indices(Some(IndicesRegion::Americas)).await?;
for (symbol, quote) in &americas.quotes {
    if let (Some(price_fv), Some(change_pct_fv)) =
        (&quote.regular_market_price, &quote.regular_market_change_percent) {
        if let (Some(price), Some(change_pct)) = (price_fv.raw, change_pct_fv.raw) {
            println!("{}: {:.2} ({:+.2}%)", symbol, price, change_pct);
        }
    }
}

// Other regions
let europe = finance::indices(Some(IndicesRegion::Europe)).await?;
let asia = finance::indices(Some(IndicesRegion::AsiaPacific)).await?;
```

**Available Regions:**

- `Americas` - ^DJI, ^GSPC, ^IXIC, ^RUT, etc.
- `Europe` - ^FTSE, ^GDAXI, ^FCHI, etc.
- `AsiaPacific` - ^N225, ^HSI, 000001.SS, etc.
- `MiddleEastAfrica` - ^TA125.TA, etc.
- `Currencies` - Major currency pairs

## Screeners

### Predefined Screeners

Use Yahoo Finance's predefined screeners:

```rust
use finance_query::{finance, Screener};

// Top gainers
let gainers = finance::screener(Screener::DayGainers, 25).await?;

// Most actives
let actives = finance::screener(Screener::MostActives, 25).await?;

// Day losers
let losers = finance::screener(Screener::DayLosers, 25).await?;

// Process results
for quote in &gainers.quotes {
    let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
    println!("{}: {:+.2}%", quote.symbol, change_pct);
}
```

See [Screeners](screeners.md) for all 15 `Screener` variants and the complete list.

### Custom Screeners

Build type-safe screening queries using `EquityScreenerQuery` or `FundScreenerQuery`:

```rust
use finance_query::{finance, EquityScreenerQuery, EquityField, ScreenerFieldExt};

// Find US large-cap tech stocks
let query = EquityScreenerQuery::new()
    .size(50)
    .sort_by(EquityField::IntradayMarketCap, false)
    .add_condition(EquityField::Region.eq_str("us"))
    .add_condition(EquityField::Sector.eq_str("Technology"))
    .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0))
    .add_condition(EquityField::AvgDailyVol3M.gt(1_000_000.0));

let results = finance::custom_screener(query).await?;
println!("Found {} stocks", results.quotes.len());
```

!!! tip "Full Screener Reference"
    See [Screeners](screeners.md) for the complete typed query API, all `EquityField` variants (80+), fund screener support, OR logic, preset constructors, and more.

## Sector & Industry Data

### Sectors

Get comprehensive sector data:

```rust
use finance_query::{finance, Sector};

let tech = finance::sector(Sector::Technology).await?;

println!("Sector: {}", tech.name);
if let Some(overview) = &tech.overview {
    if let Some(count) = overview.companies_count {
        println!("  Companies: {}", count);
    }
    if let Some(market_cap_fv) = &overview.market_cap {
        if let Some(market_cap) = market_cap_fv.raw {
            println!("  Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
        }
    }
}

// Top companies in the sector
println!("Top companies:");
for company in tech.top_companies.iter().take(10) {
    println!("  {} - {}", company.symbol, company.name.as_deref().unwrap_or("N/A"));
}

// Sector ETFs
println!("Sector ETFs: {}", tech.top_etfs.len());

// Industries in this sector
println!("Industries: {}", tech.industries.len());
```

**Available `Sector` variants:**

- `BasicMaterials`
- `CommunicationServices`
- `ConsumerCyclical`
- `ConsumerDefensive`
- `Energy`
- `Financial`
- `Healthcare`
- `Industrials`
- `RealEstate`
- `Technology`
- `Utilities`

### Industries

Get detailed industry data:

```rust
let semiconductors = finance::industry("semiconductors").await?;

println!("Industry: {}", semiconductors.name);
if let Some(overview) = &semiconductors.overview {
    println!("  Companies: {:?}", overview.companies_count);
    println!("  Market Cap: ${:?}B", overview.market_cap.map(|m| m / 1e9));
}

// Top companies
for company in semiconductors.top_companies.iter().take(5) {
    println!("  {} - {}", company.symbol, company.name.as_deref().unwrap_or(""));
}
```

**Common Industry Keys:**

- `"semiconductors"` - Semiconductor manufacturers
- `"software-infrastructure"` - Software infrastructure
- `"software-application"` - Application software
- `"electronic-components"` - Electronic components
- `"consumer-electronics"` - Consumer electronics
- `"communication-equipment"` - Communication equipment
- `"internet-content-information"` - Internet content & information

To discover more industry keys, use the `sector()` function and check the `industries` field.

## News & Transcripts

### General News

Get general market news:

```rust
let news = finance::news().await?;

for article in news.iter().take(10) {
    println!("{}", article.title);
    println!("  Source: {}", article.source);
    println!("  Time: {}", article.time);
    println!("  Link: {}", article.link);
}
```

### Earnings Transcripts

Fetch earnings call transcripts:

```rust
// Get the latest transcript
let latest = finance::earnings_transcript("AAPL", None, None).await?;
println!("Quarter: {} {}", latest.quarter(), latest.year());
println!("Speakers: {}", latest.transcript_content.speaker_mapping.len());

// Get specific quarter
let q4_2024 = finance::earnings_transcript("AAPL", Some("Q4"), Some(2024)).await?;

// Get all available transcripts
let all = finance::earnings_transcripts("MSFT", None).await?;
println!("Found {} transcripts", all.len());

// Get only recent transcripts
let recent = finance::earnings_transcripts("NVDA", Some(5)).await?;
for t in &recent {
    println!("{}: {} {}", t.title, t.transcript.quarter(), t.transcript.year());
}
```

## Market Sentiment

### Fear & Greed Index

Get the current CNN Fear & Greed Index reading from Alternative.me (no API key required):

```rust
let fg = finance::fear_and_greed().await?;

println!("Fear & Greed: {} / 100", fg.value);
println!("Classification: {}", fg.classification.as_str());
// e.g., "Extreme Fear", "Fear", "Neutral", "Greed", "Extreme Greed"
```

**`FearAndGreed` fields:**

- `value: u8` — Index value from 0 (Extreme Fear) to 100 (Extreme Greed)
- `classification: FearGreedLabel` — One of `ExtremeFear`, `Fear`, `Neutral`, `Greed`, `ExtremeGreed`
- `timestamp: i64` — Unix timestamp when the reading was recorded

## Reference Data

### Exchanges

Get list of supported exchanges with their symbol suffixes:

```rust
let exchanges = finance::exchanges().await?;

for exchange in &exchanges {
    println!("{} - {} (suffix: {})",
        exchange.country,
        exchange.market,
        exchange.suffix
    );
}
```

**Example Output:**
```
United States - NYSE (suffix: )
United States - NASDAQ (suffix: )
Taiwan - Taiwan (suffix: .TW)
France - Paris (suffix: .PA)
Germany - XETRA (suffix: .DE)
```

### Currencies

Get list of available currency pairs:

```rust
let currencies = finance::currencies().await?;

for currency in &currencies {
    let symbol = currency.symbol.as_deref().unwrap_or("N/A");
    let name = currency.short_name.as_deref().unwrap_or("N/A");
    println!("{}: {}", symbol, name);
}
```

## Next Steps

- [Screeners](screeners.md) - Full typed screener query builder with all 80+ `EquityField` variants
- [Ticker API](ticker.md) - Symbol-specific operations
- [Batch Tickers](tickers.md) - Efficient multi-symbol operations
- [FRED & Treasury](fred.md) - Macro-economic data (requires `fred` feature)
- [Crypto](crypto.md) - CoinGecko cryptocurrency data (requires `crypto` feature)
- [Feeds](feeds.md) - RSS/Atom news aggregation (requires `rss` feature)
- [DataFrame Support](dataframe.md) - Convert responses to Polars DataFrames for analysis
- [Models](models.md) - Understanding response types
- [Configuration](configuration.md) - Regional settings and network options
