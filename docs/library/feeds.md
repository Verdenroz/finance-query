# RSS / Atom Feeds

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — feeds](https://docs.rs/finance-query/latest/finance_query/feeds/index.html)

The `feeds` module aggregates RSS and Atom news from over 30 named financial sources, or any custom URL. Multiple feeds can be fetched concurrently in a single call with automatic deduplication and chronological sorting.

This page is a **living document**: every code block is compiled as a
generated test (`cargo soothfast docs gen-tests`), the offline parsing example
actually runs, and the performance statements are `soothfast:claim` markers
checked against real measurements in CI.

## Fetching a Single Feed

```rust no_run
use finance_query::feeds::{self, FeedSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Federal Reserve press releases and speeches
    let fed_news = feeds::fetch(FeedSource::FederalReserve).await?;

    for entry in fed_news.iter().take(5) {
        println!("{}: {}", entry.published.as_deref().unwrap_or("?"), entry.title);
        println!("  {}", entry.url);
    }
    Ok(())
}
```

## Fetching Multiple Feeds

```rust no_run
use finance_query::feeds::{self, FeedSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Aggregate multiple sources concurrently
    let news = feeds::fetch_all([
        FeedSource::FederalReserve,
        FeedSource::SecPressReleases,
        FeedSource::MarketWatch,
        FeedSource::Bloomberg,
        FeedSource::WsjMarkets,
    ]).await?;

    println!("Total entries (deduplicated): {}", news.len());
    for entry in news.iter().take(10) {
        println!("[{}] {}: {}", entry.source, entry.published.as_deref().unwrap_or("?"), entry.title);
    }
    Ok(())
}
```

`fetch_all` fetches all sources concurrently, deduplicates by URL, and sorts newest-first where dates are available. Individual feed failures are silently skipped.

## Custom Feed URLs

```rust no_run
use finance_query::feeds::{self, FeedSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let custom = feeds::fetch(FeedSource::Custom(
        "https://example.com/feed.xml".to_string()
    )).await?;
    println!("{} entries", custom.len());
    Ok(())
}
```

## Offline Parsing

The extractor behind every fetch is `feeds::parse_bytes` — a hand-rolled,
dependency-free RSS/Atom parser. It works on raw bytes, so you can use it on
feeds obtained by other means. This example runs as a real test:

```rust capture-output covers=finance_query::rss_parse
use finance_query::feeds;

let xml = br#"<?xml version="1.0"?>
<rss version="2.0"><channel>
  <title>Example</title>
  <item>
    <title>Markets rally</title>
    <link>https://example.com/a</link>
    <pubDate>Mon, 06 Jul 2026 12:00:00 GMT</pubDate>
  </item>
</channel></rss>"#;

let entries = feeds::parse_bytes(xml, "Example").unwrap();
assert_eq!(entries.len(), 1);
assert_eq!(entries[0].title, "Markets rally");
assert_eq!(entries[0].url, "https://example.com/a");
assert_eq!(entries[0].source, "Example");
println!("parsed {} entr(y/ies)", entries.len());
println!("title  = {:?}", entries[0].title);
println!("url    = {:?}", entries[0].url);
```

```text soothfast-output
parsed 1 entr(y/ies)
title  = "Markets rally"
url    = "https://example.com/a"
```

<!-- soothfast:claim finance_query::rss_parse.perfcnt.instructions < 100000 -->
<!-- soothfast:claim finance_query::rss_parse.walltime.median_ns < 30000 -->
Cheap enough to re-parse on every poll.

## `FeedEntry` Fields

<!-- soothfast:bind finance_query::feeds::FeedEntry -->

| Field | Type | Description |
|-------|------|-------------|
| `title` | `String` | Article or item title |
| `url` | `String` | Canonical link to the article |
| `published` | `Option<String>` | Publication date/time as RFC 3339 string |
| `summary` | `Option<String>` | Short summary or description |
| `source` | `String` | Human-readable source name (e.g., `"Federal Reserve"`) |

<!-- /soothfast:bind -->

## Available `FeedSource` Variants

### Regulatory & Government

| Variant | Source |
|---------|--------|
| `FederalReserve` | Federal Reserve press releases and speeches |
| `SecPressReleases` | SEC enforcement actions and rule changes |
| `SecFilings(form_type)` | SEC EDGAR filings by form type (e.g., `"10-K"`, `"8-K"`) |
| `Bea` | US Bureau of Economic Analysis data releases |
| `Ecb` | European Central Bank press releases and speeches |
| `Cfpb` | Consumer Financial Protection Bureau newsroom |
| `BankOfEngland` | Bank of England monetary policy notices |

### Financial News

| Variant | Source |
|---------|--------|
| `MarketWatch` | MarketWatch top stories |
| `WsjMarkets` | Wall Street Journal Markets |
| `Bloomberg` | Bloomberg Markets news |
| `FinancialTimes` | Financial Times Markets section |
| `FtLex` | FT Lex — daily market commentary column |
| `Cnbc` | CNBC Markets |
| `NytBusiness` | New York Times Business section |
| `GuardianBusiness` | The Guardian Business section |
| `Investing` | Investing.com all news |
| `Fortune` | Fortune — business and finance news |
| `BusinessWire` | Business Wire — corporate press releases (earnings, dividends, M&A) |
| `TheEconomist` | The Economist — global economics |
| `FinancialPost` | Financial Post — Canadian markets |
| `RitholtzBigPicture` | The Big Picture (Ritholtz) — macro commentary |
| `CalculatedRisk` | Calculated Risk — housing, mortgage, macro data |

### Crypto & Tech

| Variant | Source |
|---------|--------|
| `CoinDesk` | CoinDesk — cryptocurrency and blockchain news |
| `CoinTelegraph` | CoinTelegraph — crypto news and analysis |
| `TechCrunch` | TechCrunch — startup, VC, and tech news |
| `HackerNews` | Hacker News — curated tech posts (100+ points) |
| `VentureBeat` | VentureBeat — AI and enterprise technology |
| `YCombinator` | Y Combinator blog — startup ecosystem |

### International

| Variant | Source |
|---------|--------|
| `Scmp` | South China Morning Post — China business and trade |
| `NikkeiAsia` | Nikkei Asia — Japanese and Asian business news |
| `OilPrice` | OilPrice.com — energy geopolitics |

### Custom

| Variant | Description |
|---------|-------------|
| `Custom(String)` | Any RSS/Atom feed URL |

## Example: SEC EDGAR Filing Feed

```rust no_run
use finance_query::feeds::{self, FeedSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Stream the latest 10-K filings
    let filings = feeds::fetch(FeedSource::SecFilings("10-K".to_string())).await?;

    for f in &filings {
        println!("{}: {}", f.published.as_deref().unwrap_or("?"), f.title);
        println!("  {}", f.url);
    }
    Ok(())
}
```

## Next Steps

- [Finance Module](finance.md) - Financial news via Yahoo Finance
- [EDGAR](providers/edgar.md) - Structured SEC filing data with XBRL facts
- [Getting Started](getting-started.md) - Feature flag setup
