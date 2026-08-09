# Finance Module

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — finance](https://docs.rs/finance-query/latest/finance_query/finance/index.html)

The `finance` module provides market-wide operations that don't require a specific stock symbol. Use these functions to search for symbols, get market data, fetch screeners, and more.

## Search & Discovery

### Search

Search for stocks, ETFs, funds, and other securities by name or symbol:

```rust capture-output covers=finance_query::adapters::yahoo::discovery::search::SearchOptions
use finance_query::{Region, SearchOptions, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    println!("Found {} results", results.result_count());
    Ok(())
}
```

```text soothfast-output
Found 7 results
AAPL (NMS): N/A
APLE (NYQ): N/A
APC.DE (GER): N/A
D90.F (FRA): N/A
AAPL01.BK (SET): N/A
AAPL19.BK (SET): N/A
AAPL34.SA (SAO): N/A
Found 15 results
```

**SearchOptions Methods:**

<!-- soothfast:bind finance_query::adapters::yahoo::discovery::search::SearchOptions -->

- `.quotes_count(u32)` - Number of quote results (default: 6)
- `.news_count(u32)` - Number of news results (default: 4)
- `.enable_fuzzy_query(bool)` - Enable fuzzy matching (default: true)
- `.enable_logo_url(bool)` - Include logo URLs (default: true)
- `.enable_research_reports(bool)` - Include research reports (default: false)
- `.enable_cultural_assets(bool)` - Include cultural assets (default: false)
- `.recommend_count(u32)` - Number of recommendations (default: 5)
- `.region(Region)` - Search region (default: US)

<!-- /soothfast:bind -->

**SearchResults Fields:**

- `quotes` - Vec of matching quotes
- `news` - Vec of related news articles
- `research_reports` - Optional research reports
- `recommendations` - Recommended symbols

<!-- soothfast:claim finance_query::de_search.walltime.median_ns < 20000 -->
- Deserializing a search response takes **under 20 µs** — a measured claim,
  checked against real benchmarks in CI.

### Lookup

Lookup symbols filtered by asset type (equity, ETF, mutual fund, index, etc.):

```rust capture-output covers=finance_query::adapters::yahoo::discovery::lookup::LookupType
use finance_query::{LookupOptions, LookupType, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple lookup
    let results = finance::lookup("NVDA", &LookupOptions::default()).await?;
    println!("Found {} results", results.result_count());

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
    Ok(())
}
```

```text soothfast-output
Found 25 results
TECH: Bio-Techne Corp - None
BSEM: BioStem Technologies, Inc. - None
XDEF: N/A - None
MU: Micron Technology, Inc. - None
SPCX: Space Exploration Technologies  - None
PLTR: Palantir Technologies Inc. - None
SOFI: SoFi Technologies, Inc. - Some("https://s.yimg.com/lb/brands/50x50_sofi.png")
POET: POET Technologies Inc. - None
MRVL: Marvell Technology, Inc. - None
DELL: Dell Technologies Inc. - Some("https://s.yimg.com/lb/brands/50x50_delltechnologies.png")
```

**Available LookupTypes:**

<!-- soothfast:bind finance_query::adapters::yahoo::discovery::lookup::LookupType -->

- `All` - All asset types (default)
- `Equity` - Stocks
- `Etf` - Exchange-traded funds
- `MutualFund` - Mutual funds
- `Index` - Market indices
- `Future` - Futures contracts
- `Currency` - Currencies
- `Cryptocurrency` - Cryptocurrencies

<!-- /soothfast:bind -->

## Market Data

### Market Summary

Get current market summary with major indices, currencies, and commodities:

```rust capture-output
use finance_query::{Region, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default (US market)
    let summary = finance::market_summary(None).await?;
    println!("Fetched {} quotes (US)", summary.len());

    // Specific region
    let summary = finance::market_summary(Some(Region::Canada)).await?;

    for quote in &summary {
        let price = quote
            .regular_market_price
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        let change_pct = quote
            .regular_market_change_percent
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("{}: ${:.2} ({:+.2}%)", quote.symbol, price, change_pct);
    }
    Ok(())
}
```

```text soothfast-output
Fetched 15 quotes (US)
^GSPTSE: $36381.23 (+0.68%)
^GSPC: $7757.64 (+0.62%)
^DJI: $54036.93 (+0.28%)
CADUSD=X: $0.72 (+0.54%)
CL=F: $78.18 (+1.15%)
BTC-CAD: $90897.18 (+0.23%)
XRP-CAD: $1.45 (-0.16%)
GC=F: $4399.70 (+2.33%)
^RUT: $3034.49 (+1.10%)
^TNX: $4.66 (-0.21%)
^IXIC: $26690.62 (+1.30%)
^VIX: $14.90 (-1.65%)
^FTSE: $10901.09 (+0.31%)
^N225: $65606.71 (-0.12%)
CADEUR=X: $0.62 (+0.23%)
```

### Trending

Get trending stocks for a region:

```rust capture-output
use finance_query::{Region, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let trending = finance::trending(None).await?;
    println!("Fetched {} trending symbols", trending.len());
    // Or specify region
    let trending = finance::trending(Some(Region::Singapore)).await?;

    for quote in &trending {
        println!("{}", quote.symbol);
    }
    Ok(())
}
```

```text soothfast-output
Fetched 20 trending symbols
SGD=X
D05.SI
U11.SI
Z74.SI
SE
F34.SI
C6L.SI
GRAB
BN4.SI
S63.SI
A17U.SI
```

### Market Hours

Check market status and trading hours:

```rust capture-output
use finance_query::{Region, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // US market hours (default)
    let hours = finance::hours(None).await?;
    println!("{} markets (US)", hours.markets.len());

    // Japan market hours
    let hours = finance::hours(Some(Region::Japan)).await?;

    for market in &hours.markets {
        println!("{}: {}", market.name, market.status);
        println!("  Open: {:?}", market.open);
        println!("  Close: {:?}", market.close);
    }
    Ok(())
}
```

```text soothfast-output
1 markets (US)
Japanese markets: closed
  Open: Some("2026-08-10T00:00:00Z")
  Close: Some("2026-08-10T06:30:00Z")
```

### Indices

Get quotes for major world indices:

```rust capture-output
use finance_query::{IndicesRegion, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // All world indices
    let all = finance::indices(None).await?;
    println!("Fetched {} indices", all.success_count());

    // Only Americas indices (^DJI, ^GSPC, ^IXIC, etc.)
    let americas = finance::indices(Some(IndicesRegion::Americas)).await?;
    for (symbol, quote) in &americas.quotes {
        if let (Some(price_fv), Some(change_pct_fv)) = (
            &quote.regular_market_price,
            &quote.regular_market_change_percent,
        ) && let (Some(price), Some(change_pct)) = (price_fv.raw, change_pct_fv.raw)
        {
            println!("{}: {:.2} ({:+.2}%)", symbol, price, change_pct);
        }
    }

    // Other regions
    let europe = finance::indices(Some(IndicesRegion::Europe)).await?;
    let asia = finance::indices(Some(IndicesRegion::AsiaPacific)).await?;
    println!(
        "Fetched {} Europe, {} Asia-Pacific indices",
        europe.success_count(),
        asia.success_count()
    );
    Ok(())
}
```

```text soothfast-output
Fetched 41 indices
^BVSP: 172513.42 (-1.73%)
^RUT: 3034.49 (+1.10%)
^XAX: 8524.65 (+0.44%)
^MXX: 66938.64 (+0.82%)
^MERV: 3086784.50 (-0.45%)
^NYA: 24595.24 (+0.45%)
^GSPTSE: 36381.23 (+0.68%)
^GSPC: 7757.64 (+0.62%)
^IXIC: 26690.62 (+1.30%)
^IPSA: 10887.72 (-0.55%)
^DJI: 54036.93 (+0.28%)
^VIX: 14.90 (-1.65%)
Fetched 9 Europe, 12 Asia-Pacific indices
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

```rust capture-output covers=finance_query::de_screener
use finance_query::{Screener, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Top gainers
    let gainers = finance::screener(Screener::DayGainers, 25).await?;

    // Most actives
    let actives = finance::screener(Screener::MostActives, 25).await?;
    println!("Fetched {} most actives", actives.quotes.len());

    // Day losers
    let losers = finance::screener(Screener::DayLosers, 25).await?;
    println!("Fetched {} day losers", losers.quotes.len());

    // Process results
    for quote in &gainers.quotes {
        let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
        println!("{}: {:+.2}%", quote.symbol, change_pct);
    }
    Ok(())
}
```

```text soothfast-output
Fetched 25 most actives
Fetched 25 day losers
TEAM: +35.31%
DOCS: +32.62%
FIGS: +26.87%
TWLO: +24.89%
BTG: +22.98%
NTRA: +21.37%
HALO: +20.24%
FIVN: +19.81%
ROAD: +19.55%
AXTI: +17.84%
ABNB: +17.43%
FLR: +16.92%
BVC: +16.85%
CELH: +16.83%
TIC: +15.90%
SPCX: +15.83%
RDW: +14.88%
OKLO: +14.77%
ONTO: +14.74%
IAG: +14.29%
TXG: +14.10%
APPN: +13.91%
MCHP: +13.89%
CAI: +13.70%
CVSA: +13.46%
```

<!-- soothfast:claim finance_query::de_screener.walltime.median_ns < 2000000 -->
See [Screeners](screeners.md) for all 15 `Screener` variants and the complete list.

### Custom Screeners

Build type-safe screening queries using `EquityScreenerQuery` or `FundScreenerQuery`:

```rust capture-output
use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}
```

```text soothfast-output
Found 50 stocks
```

!!! tip "Full Screener Reference"
    See [Screeners](screeners.md) for the complete typed query API, all `EquityField` variants (80+), fund screener support, OR logic, preset constructors, and more.

## Sector & Industry Data

### Sectors

Get comprehensive sector data:

```rust capture-output covers=finance_query::constants::sectors::Sector
use finance_query::{Sector, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tech = finance::sector(Sector::Technology).await?;

    println!("Sector: {}", tech.name);
    if let Some(overview) = &tech.overview {
        if let Some(count) = overview.companies_count {
            println!("  Companies: {}", count);
        }
        if let Some(market_cap_fv) = &overview.market_cap
            && let Some(market_cap) = market_cap_fv.raw
        {
            println!("  Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
        }
    }

    // Top companies in the sector
    println!("Top companies:");
    for company in tech.top_companies.iter().take(10) {
        println!(
            "  {} - {}",
            company.symbol,
            company.name.as_deref().unwrap_or("N/A")
        );
    }

    // Sector ETFs
    println!("Sector ETFs: {}", tech.top_etfs.len());

    // Industries in this sector
    println!("Industries: {}", tech.industries.len());
    Ok(())
}
```

```text soothfast-output
Sector: Technology
  Companies: 851
  Market Cap: $28923.51B
Top companies:
  NVDA - NVIDIA Corporation
  AAPL - Apple Inc.
  MSFT - Microsoft Corporation
  AVGO - Broadcom Inc.
  MU - Micron Technology, Inc.
  SKHY - SK hynix Inc.
  AMD - Advanced Micro Devices, Inc.
  INTC - Intel Corporation
  CSCO - Cisco Systems, Inc.
  AMAT - Applied Materials, Inc.
Sector ETFs: 10
Industries: 13
```

**Available `Sector` variants:**

<!-- soothfast:bind finance_query::constants::sectors::Sector -->

- `BasicMaterials`
- `CommunicationServices`
- `ConsumerCyclical`
- `ConsumerDefensive`
- `Energy`
- `FinancialServices`
- `Healthcare`
- `Industrials`
- `RealEstate`
- `Technology`
- `Utilities`

<!-- /soothfast:bind -->

### Industries

Get detailed industry data:

```rust capture-output
use finance_query::finance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let semiconductors = finance::industry("semiconductors").await?;

    println!("Industry: {}", semiconductors.name);
    if let Some(overview) = &semiconductors.overview {
        println!("  Companies: {:?}", overview.companies_count);
        println!("  Market Cap: ${:?}B", overview.market_cap.map(|m| m / 1e9));
    }

    // Top companies
    for company in semiconductors.top_companies.iter().take(5) {
        println!(
            "  {} - {}",
            company.symbol,
            company.name.as_deref().unwrap_or("")
        );
    }
    Ok(())
}
```

```text soothfast-output
Industry: Semiconductors
  Companies: Some(61)
  Market Cap: $Some(11162.377781248)B
  NVDA - NVIDIA Corporation
  AVGO - Broadcom Inc.
  MU - Micron Technology, Inc.
  SKHY - SK hynix Inc.
  AMD - Advanced Micro Devices, Inc.
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

```rust capture-output covers=finance_query::de_news
use finance_query::finance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let news = finance::news().await?;

    for article in news.iter().take(10) {
        println!("{}", article.title);
        println!("  Source: {}", article.source);
        println!("  Time: {}", article.time);
        println!("  Link: {}", article.link);
    }
    Ok(())
}
```

```text soothfast-output
U.S. Consumer Brands Say Chicken, Soap and Snacks Are Selling Better Overseas
  Source: WSJ
  Time: 3 hours ago
  Link: https://www.wsj.com/business/hospitality/u-s-consumer-brands-say-chicken-soap-and-snacks-are-selling-better-overseas-3406cb68
Turbulent Month Leaves Stock Funds Up 10.6% So Far in 2026
  Source: WSJ
  Time: 5 hours ago
  Link: https://www.wsj.com/finance/investing/stock-funds-increase-year-to-date-4085a35d
US Interest Rate Forecast: Weak Jobs Cut Fed Hike Odds Ahead of CPI
  Source: FXEmpire
  Time: 9 hours ago
  Link: https://www.fxempire.com/forecasts/article/us-interest-rate-forecast-weak-jobs-cut-fed-hike-odds-ahead-of-cpi-1615671
The Week That Was, The Week Ahead: Macro and Markets, August 9
  Source: TipRanks
  Time: 9 hours ago
  Link: https://www.tipranks.com/news/the-week-that-was-the-week-ahead-macro-and-markets-august-9
The Week Ahead, CPI and Earnings Test the Stock Market Rally Near Record Highs
  Source: FXEmpire
  Time: 11 hours ago
  Link: https://www.fxempire.com/forecasts/article/the-week-ahead-cpi-and-earnings-test-the-stock-market-rally-near-record-highs-1615629
Blockbuster Earnings Bolster Stocks' Record Run
  Source: WSJ
  Time: 17 hours ago
  Link: https://www.wsj.com/finance/stocks/blockbuster-earnings-bolster-stocks-record-run-97551889
Iran Demands U.S. Withdrawal to Open Hormuz
  Source: WSJ
  Time: 22 hours ago
  Link: https://www.wsj.com/world/middle-east/u-a-e-says-iran-attacked-one-of-its-ships-in-hormuz-bfbec3b2
For retirees, staying in the stock market is critical. How much exposure is the make-or-break question
  Source: CNBC
  Time: 1 day ago
  Link: https://www.cnbc.com/2026/08/08/retirement-investing-equity-income-stocks.html
The Labor Market Might Not Need as Many Jobs to Be Healthy
  Source: WSJ
  Time: 1 day ago
  Link: https://www.wsj.com/articles/the-labor-market-might-not-need-as-many-jobs-to-be-healthy-3ed8046c
Dow Jones Forecast: Weak Jobs Report Supports Dow Near 55,000
  Source: FXEmpire
  Time: 1 day ago
  Link: https://www.fxempire.com/forecasts/article/dow-jones-forecast-weak-jobs-report-supports-dow-near-55000-1615590
```

<!-- soothfast:claim finance_query::de_news.walltime.median_ns < 15000 -->

### Earnings Transcripts

Fetch earnings call transcripts:

```rust capture-output
use finance_query::finance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the latest transcript
    let latest = finance::earnings_transcript("AAPL", None, None).await?;
    println!("Quarter: {} {}", latest.quarter(), latest.year());
    println!(
        "Speakers: {}",
        latest.transcript_content.speaker_mapping.len()
    );

    // Get specific quarter
    let q4_2024 = finance::earnings_transcript("AAPL", Some("Q4"), Some(2024)).await?;
    println!("Quarter: {} {}", q4_2024.quarter(), q4_2024.year());

    // Get all available transcripts
    let all = finance::earnings_transcripts("MSFT", None).await?;
    println!("Found {} transcripts", all.len());

    // Get only recent transcripts
    let recent = finance::earnings_transcripts("NVDA", Some(5)).await?;
    for t in &recent {
        println!(
            "{}: {} {}",
            t.title,
            t.transcript.quarter(),
            t.transcript.year()
        );
    }
    Ok(())
}
```

```text soothfast-output
Quarter: Q3 2026
Speakers: 12
Quarter: Q4 2024
Found 65 transcripts
Q1 2027 Earnings Call: Q1 2027
Q4 2026 Earnings Call: Q4 2026
Q3 2026 Earnings Call: Q3 2026
Q2 2026 Earnings Call: Q2 2026
Q1 2026 Earnings Call: Q1 2026
```

### News & Transcript Sentiment

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["sentiment"] }
    ```

With the `sentiment` feature enabled, news titles and transcript paragraphs are
scored automatically using an offline [VADER](https://github.com/cjhutto/vaderSentiment)
lexicon — no API key and no network call. Each `News` article carries an optional
`sentiment: Option<Sentiment>`, and every transcript paragraph is scored in place
when the transcript is fetched:

```rust capture-output feature=sentiment
use finance_query::{SentimentLabel, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let news = finance::news().await?;
    for article in news.iter().take(5) {
        if let Some(s) = &article.sentiment {
            println!("{} → {} ({:+.2})", article.title, s.label.as_str(), s.score);
        }
    }

    // Aggregate sentiment across a whole earnings call, length-weighted:
    let transcript = finance::earnings_transcript("AAPL", None, None).await?;
    let overall = transcript.overall_sentiment();
    assert!(matches!(
        overall.label,
        SentimentLabel::Bullish | SentimentLabel::Neutral | SentimentLabel::Bearish
    ));
    Ok(())
}
```

```text soothfast-output
U.S. Consumer Brands Say Chicken, Soap and Snacks Are Selling Better Overseas → Bullish (+0.44)
Turbulent Month Leaves Stock Funds Up 10.6% So Far in 2026 → Neutral (+0.00)
US Interest Rate Forecast: Weak Jobs Cut Fed Hike Odds Ahead of CPI → Bearish (-0.25)
The Week That Was, The Week Ahead: Macro and Markets, August 9 → Neutral (+0.00)
The Week Ahead, CPI and Earnings Test the Stock Market Rally Near Record Highs → Neutral (+0.00)
```

You can also score arbitrary text directly — the scoring itself is fully
offline, so this example runs as a real test:

```rust capture-output feature=sentiment
use finance_query::{SentimentLabel, analyze_sentiment};

let s = analyze_sentiment("Strong results and excellent guidance drove the stock higher.");
assert_eq!(s.label, SentimentLabel::Bullish);
assert!(s.score > 0.0);
assert!((0.0..=1.0).contains(&s.confidence));
println!("label = {:?}", s.label);
println!("score = {:.3}", s.score);
println!("confidence = {:.3}", s.confidence);
```

```text soothfast-output
label = Bullish
score = 0.791
confidence = 0.791
```

**`Sentiment` fields:**

- `label: SentimentLabel` — `Bullish`, `Neutral`, or `Bearish` (VADER threshold ±0.05)
- `score: f64` — Compound score from -1.0 (most bearish) to +1.0 (most bullish)
- `confidence: f64` — Magnitude of the score, 0.0 to 1.0

## Market Sentiment

### Fear & Greed Index

Get the current CNN Fear & Greed Index reading from Alternative.me (no API key required):

```rust capture-output covers=finance_query::models::sentiment::response::FearAndGreed
use finance_query::finance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fg = finance::fear_and_greed().await?;

    println!("Fear & Greed: {} / 100", fg.value);
    println!("Classification: {}", fg.classification.as_str());
    // e.g., "Extreme Fear", "Fear", "Neutral", "Greed", "Extreme Greed"
    Ok(())
}
```

```text soothfast-output
Fear & Greed: 31 / 100
Classification: Fear
```

**`FearAndGreed` fields:**

<!-- soothfast:bind finance_query::models::sentiment::response::FearAndGreed -->

- `value: u8` — Index value from 0 (Extreme Fear) to 100 (Extreme Greed)
- `classification: FearGreedLabel` — One of `ExtremeFear`, `Fear`, `Neutral`, `Greed`, `ExtremeGreed`
- `timestamp: i64` — Unix timestamp when the reading was recorded

<!-- /soothfast:bind -->

## Reference Data

### Exchanges

Get list of supported exchanges with their symbol suffixes:

```rust capture-output
use finance_query::finance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exchanges = finance::exchanges().await?;

    for exchange in &exchanges {
        println!(
            "{} - {} (suffix: {})",
            exchange.country, exchange.market, exchange.suffix
        );
    }
    Ok(())
}
```

```text soothfast-output
United States of America - Blue Ocean ATS (BOATS) (suffix: N/A)
United States of America - Cboe Indices (suffix: N/A)
United States of America - Chicago Board of Trade (CBOT)*** (suffix: .CBT)
United States of America - Chicago Mercantile Exchange (CME)*** (suffix: .CME)
United States of America - Dow Jones Indexes (suffix: N/A)
United States of America - ICE Futures US (suffix: .NYB)
United States of America - Nasdaq Global Index Data Service (suffix: N/A)
United States of America - Nasdaq Stock Exchange (suffix: N/A)
United States of America - New York Commodities Exchange (COMEX)*** (suffix: .CMX)
United States of America - New York Mercantile Exchange (NYMEX)*** (suffix: .NYM)
United States of America - NYSE Indices (suffix: N/A)
United States of America - Options Price Reporting Authority (OPRA) (suffix: N/A)
United States of America - OTC Markets Group** (suffix: N/A)
United States of America - S & P Indices (suffix: N/A)
Argentina - Buenos Aires Stock Exchange (BYMA) (suffix: .BA)
Austria - Vienna Stock Exchange (suffix: .VI)
Australia - Australian Stock Exchange (ASX) (suffix: .AX)
Australia - Cboe Australia (suffix: .XA)
Belgium - Euronext Brussels (suffix: .BR)
Brazil - Sao Paolo Stock Exchange (BOVESPA) (suffix: .SA)
Canada - Canadian Securities Exchange (suffix: .CN)
Canada - Cboe Canada (suffix: .NE)
Canada - Toronto Stock Exchange (TSX) (suffix: .TO)
Canada - TSX Venture Exchange (TSXV) (suffix: .V)
Chile - Santiago Stock Exchange (suffix: .SN)
China - Shanghai Stock Exchange (suffix: .SS)
China - Shenzhen Stock Exchange (suffix: .SZ)
Colombia - Colombia Stock Exchange (suffix: .CL)
Czech Republic - Prague Stock Exchange Index (suffix: .PR)
Denmark - Nasdaq OMX Copenhagen (suffix: .CO)
Egypt - Egyptian Exchange Index (EGID) (suffix: .CA)
Estonia - Nasdaq OMX Tallinn (suffix: .TL)
Europe - Cboe Europe (suffix: .XD)
Europe - Euronext (suffix: .NX)
Finland - Nasdaq OMX Helsinki (suffix: .HE)
France - Euronext Paris (suffix: .PA)
Germany - Berlin Stock Exchange (suffix: .BE)
Germany - Bremen Stock Exchange (suffix: .BM)
Germany - Dusseldorf Stock Exchange (suffix: .DU)
Germany - Frankfurt Stock Exchange (suffix: .F)
Germany - Hamburg Stock Exchange (suffix: .HM)
Germany - Hanover Stock Exchange (suffix: .HA)
Germany - Munich Stock Exchange (suffix: .MU)
Germany - Stuttgart Stock Exchange (suffix: .SG)
Germany - Deutsche Boerse XETRA (suffix: .DE)
Global - Collectable Indices (suffix: .REGA)
Global - Cryptocurrencies (suffix: N/A)
Global - Cryptocurrencies (suffix: N/A)
Global - Currency Rates (suffix: =X)
Global - MSCI Indices (suffix: N/A)
Greece - Athens Stock Exchange (ATHEX) (suffix: .AT)
Hong Kong - Hang Seng Indices (suffix: N/A)
Hong Kong - Hong Kong Stock Exchange (HKEX)* (suffix: .HK)
Hungary - Budapest Stock Exchange (suffix: .BD)
Iceland - Nasdaq OMX Iceland (suffix: .IC)
India - Bombay Stock Exchange (suffix: .BO)
India - National Stock Exchange of India (suffix: .NS)
Indonesia - Indonesia Stock Exchange (IDX) (suffix: .JK)
Ireland - Euronext Dublin (suffix: .IR)
Israel - Tel Aviv Stock Exchange (suffix: .TA)
Italy - EuroTLX (suffix: .TI)
Italy - Italian Stock Exchange (suffix: .MI)
Japan - Nikkei Indices (suffix: N/A)
Japan - Tokyo Stock Exchange (suffix: .T)
Kuwait - Boursa Kuwait (suffix: .KW)
Latvia - Nasdaq OMX Riga (suffix: .RG)
Lithuania - Nasdaq OMX Vilnius (suffix: .VS)
Malaysia - Malaysian Stock Exchange (suffix: .KL)
Mexico - Mexico Stock Exchange (BMV) (suffix: .MX)
Netherlands - Euronext Amsterdam (suffix: .AS)
New Zealand - New Zealand Stock Exchange (NZX) (suffix: .NZ)
Norway - Oslo Stock Exchange (suffix: .OL)
Philippines - Philippine Stock Exchange Indices (suffix: .PS)
Poland - Warsaw Stock Exchange (suffix: .WA)
Portugal - Euronext Lisbon (suffix: .LS)
Qatar - Qatar Stock Exchange (suffix: .QA)
Romania - Bucharest Stock Exchange (suffix: .RO)
Singapore - Singapore Stock Exchange (SGX) (suffix: .SI)
South Africa - Johannesburg Stock Exchange (suffix: .JO)
South Korea - Korea Stock Exchange (suffix: .KS)
South Korea - KOSDAQ (suffix: .KQ)
Spain - Madrid Stock Exchange (BME) (suffix: .MC)
Saudi Arabia - Saudi Stock Exchange (Tadawul) (suffix: .SAU)
Sweden - Nasdaq OMX Stockholm (suffix: .ST)
Switzerland - Swiss Exchange (SIX) (suffix: .SW)
Taiwan - Taiwan OTC Exchange (suffix: .TWO)
Taiwan - Taiwan Stock Exchange (TWSE) (suffix: .TW)
Thailand - Stock Exchange of Thailand (SET) (suffix: .BK)
Turkey - Borsa İstanbul (suffix: .IS)
United Arab Emirates - Dubai Financial Market (suffix: .AE)
United Kingdom - Aquis Exchange AQSE (suffix: .AQ)
United Kingdom - Cboe UK (suffix: .XC)
United Kingdom - FTSE Indices (suffix: N/A)
United Kingdom - London Stock Exchange (suffix: .L)
United Kingdom - London Stock Exchange (suffix: .IL)
Venezuela - Caracas Stock Exchange (suffix: .CR)
Vietnam - Ho Chi Minh City Stock Exchange (suffix: .VN)
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

```rust capture-output
use finance_query::finance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let currencies = finance::currencies().await?;

    for currency in &currencies {
        let symbol = currency.symbol.as_deref().unwrap_or("N/A");
        let name = currency.short_name.as_deref().unwrap_or("N/A");
        println!("{}: {}", symbol, name);
    }
    Ok(())
}
```

```text soothfast-output
FJD: FJD
MXN: MXN
SCR: SCR
CDF: CDF
GTQ: GTQ
BBD: BBD
CLP: CLP
UGX: UGX
HNL: HNL
ZAR: ZAR
MXV: MXV
TND: TND
STN: STN
SLE: SLE
BSD: BSD
SLL: SLL
SDG: SDG
IQD: IQD
CUP: CUP
GMD: GMD
TWD: TWD
RSD: RSD
DOP: DOP
KMF: KMF
MYR: MYR
FKP: FKP
XOF: XOF
GEL: GEL
UYU: UYU
MAD: MAD
CVE: CVE
TOP: TOP
PGK: PGK
OMR: OMR
AZN: AZN
SEK: SEK
KES: KES
BTN: BTN
UAH: UAH
GNF: GNF
MZN: MZN
ERN: ERN
SVC: SVC
ARS: ARS
QAR: QAR
IRR: IRR
THB: THB
XPF: XPF
UZS: UZS
CNY: CNY
MRU: MRU
BDT: BDT
LYD: LYD
BMD: BMD
PHP: PHP
KWD: KWD
RUB: RUB
PYG: PYG
JMD: JMD
ISK: ISK
COP: COP
USD: USD
MKD: MKD
DZD: DZD
PAB: PAB
SGD: SGD
ETB: ETB
SOS: SOS
KGS: KGS
VUV: VUV
LAK: LAK
BND: BND
XAF: XAF
LRD: LRD
HRK: HRK
CHF: CHF
ALL: ALL
DJF: DJF
VES: VES
ZMW: ZMW
TZS: TZS
VND: VND
AUD: AUD
ILS: ILS
KPW: KPW
GYD: GYD
GHS: GHS
KHR: KHR
MDL: MDL
BOB: BOB
IDR: IDR
KYD: KYD
AMD: AMD
BWP: BWP
TRY: TRY
SHP: SHP
LBP: LBP
TJS: TJS
JOD: JOD
HKD: HKD
RWF: RWF
AED: AED
EUR: EUR
LSL: LSL
DKK: DKK
CAD: CAD
BGN: BGN
MMK: MMK
MUR: MUR
NOK: NOK
SYP: SYP
GIP: GIP
RON: RON
LKR: LKR
NGN: NGN
CZK: CZK
CRC: CRC
PKR: PKR
XCD: XCD
HTG: HTG
ANG: ANG
XCG: XCG
BHD: BHD
SRD: SRD
SZL: SZL
KZT: KZT
SAR: SAR
TTD: TTD
YER: YER
MVR: MVR
AFN: AFN
INR: INR
AWG: AWG
KRW: KRW
NPR: NPR
JPY: JPY
MNT: MNT
PLN: PLN
AOA: AOA
SBD: SBD
GBP: GBP
BYN: BYN
HUF: HUF
BIF: BIF
MWK: MWK
MGA: MGA
XDR: XDR
BZD: BZD
BAM: BAM
EGP: EGP
MOP: MOP
NAD: NAD
SSP: SSP
NIO: NIO
PEN: PEN
WST: WST
NZD: NZD
TMT: TMT
CLF: CLF
BRL: BRL
```

## Next Steps

- [Screeners](screeners.md) - Full typed screener query builder with all 80+ `EquityField` variants
- [Ticker API](ticker.md) - Symbol-specific operations
- [Batch Tickers](tickers.md) - Efficient multi-symbol operations
- [FRED & Treasury](providers/fred.md) - Macro-economic data (requires `fred` feature)
- [Crypto](providers/coingecko.md) - CoinGecko cryptocurrency data (requires `crypto` feature)
- [Feeds](feeds.md) - RSS/Atom news aggregation (requires `rss` feature)
- [DataFrame Support](dataframe.md) - Convert responses to Polars DataFrames for analysis
- [Configuration](configuration.md) - Regional settings and network options
