# RSS / Atom Feeds

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["rss"] }
    ```

The `feeds` module aggregates RSS and Atom news from over 30 named financial sources, or any custom URL. Multiple feeds can be fetched concurrently in a single call with automatic deduplication and chronological sorting.

```rust
use finance_query::feeds::{self, FeedSource};
```

## Fetching a Single Feed

```rust
use finance_query::feeds::{self, FeedSource};

// Federal Reserve press releases and speeches
let fed_news = feeds::fetch(FeedSource::FederalReserve).await?;

for entry in fed_news.iter().take(5) {
    println!("{}: {}", entry.published.as_deref().unwrap_or("?"), entry.title);
    if let Some(url) = entry.url.as_str().chars().next() {
        println!("  {}", entry.url);
    }
}
```

## Fetching Multiple Feeds

```rust
use finance_query::feeds::{self, FeedSource};

// Aggregate multiple sources concurrently
let news = feeds::fetch_all(&[
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
```

`fetch_all` fetches all sources concurrently, deduplicates by URL, and sorts newest-first where dates are available. Individual feed failures are silently skipped.

## Custom Feed URLs

```rust
let custom = feeds::fetch(FeedSource::Custom(
    "https://example.com/feed.xml".to_string()
)).await?;
```

## `FeedEntry` Fields

| Field | Type | Description |
|-------|------|-------------|
| `title` | `String` | Article or item title |
| `url` | `String` | Canonical link to the article |
| `published` | `Option<String>` | Publication date/time as RFC 3339 string |
| `summary` | `Option<String>` | Short summary or description |
| `source` | `String` | Human-readable source name (e.g., `"Federal Reserve"`) |

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

```rust
use finance_query::feeds::{self, FeedSource};

// Stream the latest 10-K filings
let filings = feeds::fetch(FeedSource::SecFilings("10-K".to_string())).await?;

for f in &filings {
    println!("{}: {}", f.published.as_deref().unwrap_or("?"), f.title);
    println!("  {}", f.url);
}
```

## Next Steps

- [Finance Module](finance.md) - Financial news via Yahoo Finance
- [EDGAR](edgar.md) - Structured SEC filing data with XBRL facts
- [Getting Started](getting-started.md) - Feature flag setup
