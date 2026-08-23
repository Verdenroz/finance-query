# GDELT (Global News)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["gdelt"] }
    ```

News previously came from Yahoo (a scraper, fragile to page changes) or Alpha Vantage (keyed, 25 req/day). [GDELT](https://www.gdeltproject.org/)'s DOC 2.0 API indexes worldwide online news across 65 languages, updated roughly every 15 minutes, entirely keyless. That closes the gap for the news slice of `Capability::CORPORATE`.

## Setup

```rust no_run feature=gdelt
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::CORPORATE, [Provider::Gdelt])
        .build()
        .await?;

    let ticker = providers.ticker("GOOGL").build().await?;
    for article in ticker.news().await? {
        println!("{}: {} ({})", article.time, article.title, article.source);
    }
    Ok(())
}
```

!!! note "GDELT serves news only"
    GDELT has no concept of a corporate calendar (earnings/dividends/splits), so a `CORPORATE` route that includes GDELT should also include a provider that serves `fetch_events` if the calendar is needed.

## Query Derivation

GDELT has no ticker vocabulary of its own, so `Ticker::news()` searches GDELT for the ticker symbol itself, quoted for an exact phrase match (e.g. `"GOOGL"`). This favors precision over recall: it surfaces articles that print the ticker verbatim, common in financial press (e.g. `"(NASDAQ: GOOGL)"`), rather than every article about the underlying company by name, which would require a separate symbol-to-company-name lookup this adapter doesn't perform.

!!! warning "Short tickers are rejected"
    GDELT rejects phrase queries under 5 characters, so symbols shorter than that (`AAPL`, `TSLA`, `AMD`, `F`, and so on) fail with `InvalidParameter` and dispatch falls through to the next `CORPORATE` provider. GDELT serves longer tickers and non-US symbols well in practice; route a keyed or scraped provider ahead of it if you need coverage for short US tickers.

## What the Fields Mean

| `News` field | Source |
|--------------|--------|
| `title` | Article headline, or empty when GDELT omits it |
| `link` | Article URL |
| `source` | Publishing domain (e.g. `"reuters.com"`) |
| `img` | `socialimage`, or empty when GDELT has no thumbnail for the article |
| `time` | Relative time (`"3 hours ago"`) computed from `seendate`, GDELT's own first-indexed timestamp |

GDELT sometimes omits a field entirely, rather than sending an empty string, for older or thinly indexed sources, so everything but the URL degrades to an empty string rather than dropping the article.

## History Depth

Each request searches the last two weeks (GDELT's `timespan` parameter) and returns up to 50 articles. That's generous enough to cover a busy news day without paging; the DOC API caps at 250 records per call regardless.

## Rate Limits

GDELT documents no formal quota for the DOC API but asks callers to keep requests to roughly one every 5 seconds. The client paces itself to that rate rather than waiting to be throttled. A throttled response is still mapped to `RateLimited` when it does occur (GDELT states the retry interval in a plain-text body rather than a `Retry-After` header).

## Next Steps

- [Alpha Vantage](alphavantage.md): keyed news alternative with sentiment scoring
- [Providers Overview](index.md): routing and fallback
