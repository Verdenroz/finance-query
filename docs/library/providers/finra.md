# FINRA (short sale volume)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["finra"] }
    ```

FINRA publishes daily aggregated short-sale volume for every reported security through its [Query API](https://developer.finra.org/), free and without credentials.

`Ticker::short_volume()` previously needed Polygon. Routing `FUNDAMENTALS` to FINRA gives it a keyless provider reading from the primary source.

!!! warning "Free for non-commercial use"
    FINRA's Query API is free for **non-commercial** use; commercial use requires an agreement with FINRA. That is a licensing question this adapter does not change — check your use case before shipping.

## Setup

```rust no_run feature=finra
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FUNDAMENTALS, [Provider::Finra, Provider::Yahoo])
        .build()
        .await?;

    let ticker = providers.ticker("AAPL").build().await?;
    for day in ticker.short_volume().await? {
        println!(
            "{} short {:?} of {:?}",
            day.date.unwrap_or_default(),
            day.short_volume,
            day.total_volume
        );
    }
    Ok(())
}
```

!!! note "FINRA serves short volume only"
    FINRA publishes no financial statements, so `fetch_financials` reports `NotSupported` and dispatch falls through. Under `Fetch::Sequential`, list a full `FUNDAMENTALS` provider after it — as in the example above — so `financials()` still resolves.

## What the Numbers Mean

FINRA reports each symbol **once per reporting facility per day** — the Nasdaq (`Q`), NYSE (`N`), and OTC (`B`) trade reporting facilities each file separately. A listed name therefore has three raw rows for a single trading day.

The adapter sums them into one figure per date, reproducing the consolidated numbers FINRA itself publishes in its daily `CNMSshvol` file:

| `ShortVolume` field | Source |
|---------------------|--------|
| `date` | `tradeReportDate` (`YYYY-MM-DD`) |
| `short_volume` | Sum of `shortParQuantity` across facilities |
| `short_exempt_volume` | Sum of `shortExemptParQuantity` |
| `total_volume` | Sum of `totalParQuantity` |

A field no facility reported stays `None` rather than becoming `0.0`.

!!! info "Short volume is not short interest"
    Short *volume* is how many shares were sold short on a given day — much of it market-maker hedging that closes intraday. Short *interest* is the open short position at a settlement date. For the latter, use `Ticker::short_interest()`, which Yahoo and Polygon serve.

## History Depth and Ordering

FINRA keeps roughly a rolling year online, and one call requests that whole window.

Results come back oldest-first. That ordering is applied locally, because FINRA rejects server-side sorting unless every partition key is pinned with an equality filter — which a date *range* by definition does not do.

A symbol with no reportable short volume returns an **empty series, not an error**: FINRA answers such a query with HTTP 204 and no body.

## Rate Limits

FINRA's anonymous tier is metered per day rather than per second. The client paces at 2 requests/second purely to avoid looking abusive; a free developer account raises the quota but is not required.

## Next Steps

- [Polygon.io](polygon.md) — short interest history and short volume, keyed
- [Providers Overview](index.md) — routing and fallback
