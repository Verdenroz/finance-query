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
    Ok(())
}
```

```text soothfast-output
Found 7 results
AAPL (NMS): N/A
APLE (NYQ): N/A
AAPL.SW (EBS): N/A
2788.T (JPX): N/A
D90.F (FRA): N/A
AAPLC.BA (BUE): N/A
AAPL19.BK (SET): N/A
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
TECH: Bio-Techne Corp - None
XDEF: N/A - None
MU: Micron Technology, Inc. - None
SPCX: Space Exploration Technologies  - None
PLTR: Palantir Technologies Inc. - None
MRVL: Marvell Technology, Inc. - None
POET: POET Technologies Inc. - None
SOFI: SoFi Technologies, Inc. - Some("https://s.yimg.com/lb/brands/50x50_sofi.png")
DELL: Dell Technologies Inc. - Some("https://s.yimg.com/lb/brands/50x50_delltechnologies.png")
UBER: Uber Technologies, Inc. - Some("https://s.yimg.com/lb/brands/50x50_uber.png")
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
^GSPTSE: $35263.85 (-0.22%)
^GSPC: $7457.69 (-1.01%)
^DJI: $52146.42 (-0.77%)
CADUSD=X: $0.71 (+0.14%)
CL=F: $81.78 (+4.47%)
BTC-CAD: $90388.92 (+0.73%)
XRP-CAD: $1.53 (+0.15%)
GC=F: $4018.80 (+0.67%)
^RUT: $2962.22 (-0.42%)
^TNX: $4.54 (-0.61%)
^IXIC: $25520.24 (-1.40%)
^VIX: $18.77 (+12.19%)
^FTSE: $10600.37 (+0.27%)
^N225: $64141.12 (-4.03%)
CADEUR=X: $0.62 (+0.19%)
```

### Trending

Get trending stocks for a region:

```rust capture-output
use finance_query::{Region, finance};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let trending = finance::trending(None).await?;
    // Or specify region
    let trending = finance::trending(Some(Region::Singapore)).await?;

    for quote in &trending {
        println!("{}", quote.symbol);
    }
    Ok(())
}
```

```text soothfast-output
MU
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
U.S. markets: closed
  Open: Some("2026-07-21T00:00:00Z")
  Close: Some("2026-07-21T06:30:00Z")
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
    Ok(())
}
```

```text soothfast-output
Fetched 41 indices
^MERV: 3199934.50 (+0.46%)
^DJI: 52146.42 (-0.77%)
^GSPTSE: 35263.85 (-0.22%)
^MXX: 66634.23 (+0.42%)
^GSPC: 7457.69 (-1.01%)
^IPSA: 10887.72 (-0.54%)
^RUT: 2962.22 (-0.42%)
^VIX: 18.77 (+12.19%)
^BVSP: 173714.08 (-0.06%)
^IXIC: 25520.24 (-1.40%)
^XAX: 8097.59 (+1.27%)
^NYA: 23816.97 (-0.56%)
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

    // Day losers
    let losers = finance::screener(Screener::DayLosers, 25).await?;

    // Process results
    for quote in &gainers.quotes {
        let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
        println!("{}: {:+.2}%", quote.symbol, change_pct);
    }
    Ok(())
}
```

```text soothfast-output
LCID: +13.93%
SLS: +12.26%
ORKA: +11.63%
DNTH: +10.75%
COAG: +10.69%
TRVI: +9.85%
TRV: +9.22%
SION: +9.10%
BHVN: +8.96%
VG: +8.92%
MBX: +7.89%
IOVA: +7.30%
AGL: +7.11%
PTGX: +6.73%
FRVO: +6.66%
MMED: +6.61%
EROC: +6.50%
SYRE: +6.49%
WU: +6.47%
PVLA: +6.41%
RAPP: +6.20%
LQDA: +6.20%
LEU: +6.11%
GLUE: +6.04%
TALO: +5.67%
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
  Market Cap: $26834.07B
Top companies:
  NVDA - NVIDIA Corporation
  AAPL - Apple Inc.
  MSFT - Microsoft Corporation
  AVGO - Broadcom Inc.
  SKHY - SK hynix Inc.
  MU - Micron Technology, Inc.
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
  Companies: Some(60)
  Market Cap: $Some(10275.506880512)B
  NVDA - NVIDIA Corporation
  AVGO - Broadcom Inc.
  SKHY - SK hynix Inc.
  MU - Micron Technology, Inc.
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
What to Know About the Chinese AI Models Rattling U.S. Stocks
  Source: WSJ
  Time: 6 hours ago
  Link: https://www.wsj.com/tech/ai/what-to-know-about-the-chinese-ai-models-rattling-u-s-stocks-1a80a479
Forget gasoline: Why the price surge in this under-the-radar fuel is the real threat to the U.S. economy
  Source: Market Watch
  Time: 6 hours ago
  Link: https://www.marketwatch.com/story/forget-gasoline-why-the-price-surge-in-this-under-the-radar-fuel-is-the-real-threat-to-the-u-s-economy-710b9470
'WarshGPT': How Wall Street is adapting to the Fed's new era of communication
  Source: CNBC
  Time: 6 hours ago
  Link: https://www.cnbc.com/2026/07/18/warshgpt-federal-reserve-communications-task-force-warsh.html
Will the Fed hike interest rates this month?
  Source: Market Watch
  Time: 7 hours ago
  Link: https://www.marketwatch.com/story/will-the-fed-hike-interest-rates-this-month-6e8f952a
U.S. military says it has completed the latest round of strikes against Iran, amid more disruptions to shipping
  Source: CNBC
  Time: 8 hours ago
  Link: https://www.cnbc.com/2026/07/18/us-military-says-it-completed-latest-round-of-strikes-against-iran.html
Dow Jones Forecast: Tariff Risks Test Rally as Index Eyes 55,000
  Source: FXEmpire
  Time: 9 hours ago
  Link: https://www.fxempire.com/forecasts/article/dow-jones-forecast-tariff-risks-test-rally-as-index-eyes-55000-1611225
View From the EDGE® July 2026: The Importance of Diversification
  Source: ETF Trends
  Time: 17 hours ago
  Link: https://www.etftrends.com/etf-strategist-content-hub/view-from-the-edge-july-2026-the-importance-of-diversification/
Review & Preview: It Could Have Been Worse
  Source: Barrons
  Time: 18 hours ago
  Link: https://www.barrons.com/articles/stocks-today-ai-tech-declines-market-indexes-lower-cc7f100f
Meet Kimi K3, the newest Chinese AI model haunting Silicon Valley
  Source: Market Watch
  Time: 19 hours ago
  Link: https://www.marketwatch.com/story/meet-kimi-k3-the-newest-chinese-ai-model-haunting-silicon-valley-755ed738
The average stock is having a moment as semiconductors struggle. It's a sign of a healthy market.
  Source: Market Watch
  Time: 20 hours ago
  Link: https://www.marketwatch.com/story/the-average-stock-is-having-a-moment-as-semiconductors-struggle-its-a-sign-of-a-healthy-market-0dc01e9a
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
Quarter: Q2 2026
Speakers: 13
Found 64 transcripts
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
What to Know About the Chinese AI Models Rattling U.S. Stocks → Neutral (+0.00)
Forget gasoline: Why the price surge in this under-the-radar fuel is the real threat to the U.S. economy → Bearish (-0.65)
'WarshGPT': How Wall Street is adapting to the Fed's new era of communication → Neutral (+0.00)
Will the Fed hike interest rates this month? → Bullish (+0.49)
U.S. military says it has completed the latest round of strikes against Iran, amid more disruptions to shipping → Bearish (-0.60)
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
Fear & Greed: 25 / 100
Classification: Extreme Fear
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
BBD: BBD
GTQ: GTQ
CLP: CLP
UGX: UGX
HNL: HNL
MXV: MXV
ZAR: ZAR
TND: TND
STN: STN
SLE: SLE
SLL: SLL
BSD: BSD
SDG: SDG
IQD: IQD
GMD: GMD
CUP: CUP
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
OMR: OMR
PGK: PGK
AZN: AZN
SEK: SEK
KES: KES
UAH: UAH
BTN: BTN
GNF: GNF
ERN: ERN
MZN: MZN
SVC: SVC
ARS: ARS
QAR: QAR
IRR: IRR
THB: THB
UZS: UZS
CNY: CNY
XPF: XPF
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
VUV: VUV
KGS: KGS
SOS: SOS
LAK: LAK
BND: BND
XAF: XAF
LRD: LRD
CHF: CHF
HRK: HRK
DJF: DJF
ALL: ALL
VES: VES
ZMW: ZMW
TZS: TZS
VND: VND
AUD: AUD
ILS: ILS
GHS: GHS
KPW: KPW
GYD: GYD
KHR: KHR
BOB: BOB
MDL: MDL
IDR: IDR
KYD: KYD
AMD: AMD
TRY: TRY
BWP: BWP
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
NOK: NOK
MUR: MUR
SYP: SYP
GIP: GIP
RON: RON
LKR: LKR
NGN: NGN
CZK: CZK
CRC: CRC
PKR: PKR
XCD: XCD
ANG: ANG
HTG: HTG
XCG: XCG
BHD: BHD
KZT: KZT
SZL: SZL
SRD: SRD
TTD: TTD
SAR: SAR
YER: YER
MVR: MVR
AFN: AFN
INR: INR
AWG: AWG
NPR: NPR
KRW: KRW
MNT: MNT
JPY: JPY
AOA: AOA
PLN: PLN
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
NZD: NZD
WST: WST
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
