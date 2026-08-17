# Financial Modeling Prep (FMP)

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Provider::Fmp](https://docs.rs/finance-query/latest/finance_query/providers/enum.Provider.html#variant.Fmp)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["fmp"] }
    ```

Financial Modeling Prep provides fundamentals, historical prices, insider trading data, institutional holdings, and screening. Free tier: 250 requests per day.

## Setup

Set the API key via environment variable:

```bash
export FMP_API_KEY="your-fmp-api-key"
```

No manual init call needed — the provider reads the key during `TickerBuilder::build()`.

## Usage

```rust no_run feature=fmp
use finance_query::format::Raw;
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Fmp, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let ticker = providers.ticker("AAPL").build().await?;
    let quote = ticker.quote::<Raw>().await?;
    println!("{} quote received", quote.symbol);
    Ok(())
}
```

## Capabilities

| Data type | Support |
|-----------|---------|
| Quote | ✓ |
| Chart | ✓ |
| Fundamentals | ✓ |
| Corporate | ✓ |
| Options | — |
| Market | ✓ |
| Discovery | ✓ |
| Indices | ✓ |
| Commodities | ✓ |
| Forex | ✓ |
| Crypto | ✓ |
| Futures | — |
| Technicals | ✓ |
| Economic | — |
| Filings | — |
| Sentiment | — |

## FMP-only `Ticker` methods

These route through `Capability::FUNDAMENTALS`, so FMP must be first in that
route for them to resolve — no other wired provider serves them.

```rust,ignore
let providers = Providers::builder()
    .route(Capability::FUNDAMENTALS, [Provider::Fmp, Provider::Yahoo])
    .build()
    .await?;
let ticker = providers.ticker("AAPL").build().await?;

let target = ticker.price_target_consensus().await?;  // high / low / mean / median
let activity = ticker.price_target_summary().await?;  // targets published per window
let rating = ticker.rating_consensus().await?;        // grade distribution + label

let metrics = ticker.key_metrics_ttm().await?;        // current TTM valuation/returns
let ratios = ticker.ratios_ttm().await?;              // current TTM margins/per-share
```

| Method | Returns | FMP endpoint |
|--------|---------|--------------|
| `price_target_consensus()` | `PriceTargetConsensus` | `/stable/price-target-consensus` |
| `price_target_summary()` | `PriceTargetSummary` | `/stable/price-target-summary` |
| `rating_consensus()` | `RatingConsensus` | `/stable/grades-consensus` |
| `key_metrics_ttm()` | `KeyMetricsTtm` | `/stable/key-metrics-ttm` |
| `ratios_ttm()` | `FinancialRatiosTtm` | `/stable/ratios-ttm` |
| `executive_compensation()` | `Vec<ExecutiveCompensation>` | `/stable/governance-executive-compensation` |
| `employee_count()` | `Vec<EmployeeCount>` | `/stable/historical-employee-count` |

Per-share TTM metrics are served by `ratios_ttm()`, not `key_metrics_ttm()` —
FMP's stable tier moved them between the two endpoints.

`executive_compensation()` and `employee_count()` route through
`Capability::CORPORATE` instead. FMP also serves `share_float()`, which the
default Yahoo route already covers — routing `FUNDAMENTALS` to FMP just changes
which source answers it.

The TTM snapshots are single always-current rollups; `financials(..)` remains the
period-indexed series. FMP computes the trailing window server-side, so partial
periods and restatements are handled there rather than by summing four quarters
client-side.

## See Also

- [Multi-Provider Architecture](index.md) — Provider configuration and strategies
- [Ticker API](../ticker.md) — Single-symbol data access
