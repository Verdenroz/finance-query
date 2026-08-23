# Senate PTR (Congressional Trades)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["senatetrades"] }
    ```

The STOCK Act of 2012 requires members of Congress to disclose trades over $1,000 within 45 days via a Periodic Transaction Report (PTR). The Senate publishes these free of charge at `efdsearch.senate.gov`, but unlike the House Clerk's static per-year archive, the disclaimer gate, search results, and each filing's transaction table all render client-side behind Akamai bot protection. So this adapter drives a real headless Chromium via [`chromiumoxide`](https://docs.rs/chromiumoxide) instead of a plain HTTP client, and needs a Chrome or Chromium binary available on the host running it.

House PTRs are the counterpart adapter, [`housetrades`](housetrades.md). `Provider::CongressTrades` merges both when the corresponding features are compiled in; see [Providers Overview](index.md).

## Setup

```rust no_run feature=senatetrades
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FILINGS, [Provider::CongressTrades, Provider::Edgar])
        .build()
        .await?;

    let filings = providers.filings("AAPL");
    for trade in filings.congressional_trades().await? {
        println!(
            "{} {}: {} {} on {}",
            trade.office.as_deref().unwrap_or("?"),
            trade.last_name.as_deref().unwrap_or("?"),
            trade.trade_type.as_deref().unwrap_or("?"),
            trade.amount.as_deref().unwrap_or("?"),
            trade.transaction_date.as_deref().unwrap_or("?")
        );
    }
    Ok(())
}
```

With only `senatetrades` compiled in, `Provider::CongressTrades` serves Senate rows exclusively, and every `CongressionalTrade.office` reads `"Senate"`. Enable `housetrades` alongside it to merge in House rows too.

!!! danger "Akamai bot protection can block this source"
    `efdsearch.senate.gov` sits behind Akamai. The same launch flags that pass from a residential IP fail (HTTP 403, `errors.edgesuite.net`) from a VPN, cloud, or datacenter IP, and a separate, IP-independent rate block can also trigger under heavy request volume. This adapter can't bypass either condition, but it does recognize Akamai's error page and fail fast instead of exhausting the full element-detection timeout against it. When `Provider::CongressTrades` has both `housetrades` and `senatetrades` compiled in, a Senate failure just drops those rows from the merged result rather than failing the whole call; House data still comes back. Running with only `senatetrades` enabled, or on a blocked IP, means `congressional_trades()` can fail outright.

## How Symbol Lookup Works

`efdsearch.senate.gov` has no query API of its own; search results and each filing's transaction table render client-side. Like the House adapter, there's no per-symbol search either, only per-filer-type and per-date, so a symbol lookup works by:

1. Launching a fresh headless Chromium instance, navigating to the search page, accepting the disclaimer, and filtering to Senator PTRs.
2. Reading the most recent 100 filer and report-link rows out of the results table, sorted newest first.
3. Opening each report page concurrently, reading its rendered transaction table, and keeping only the rows whose ticker matches the requested symbol.

The browser session is not pooled: each call to `congressional_trades()` launches its own Chromium instance and closes it at the end of that call.

!!! warning "Bounded recency window, not a full historical search"
    Only the most recent 100 Senator PTR filings are scanned per request. This is a bounded-cost recency window, not an index over the full historical archive. A symbol traded only in an older filing beyond that window won't appear.

!!! warning "Paper filings are silently skipped"
    Only filings submitted through the site's e-filing system render an HTML transaction table. Older ("paper") filings don't, and produce zero matching transactions rather than an error, since this adapter does not OCR them.

## What the Fields Mean

| `CongressionalTrade` field | Source |
|-----------------------------|--------|
| `symbol` | Ticker column of the report's transaction table |
| `first_name` / `last_name` | From the search results table |
| `office` | Always `"Senate"` for this adapter |
| `district` | Always `None`; senators represent a whole state |
| `trade_type` / `amount` / `asset_description` | Read directly from the report's transaction table |
| `transaction_date` / `disclosure_date` | `YYYY-MM-DD` |
| `link` | Direct URL to the source report page |

## Next Steps

- [House PTR](housetrades.md): the House-side counterpart
- [Filings](../filings.md): `Filings::congressional_trades()`
- [Providers Overview](index.md): routing and fallback
