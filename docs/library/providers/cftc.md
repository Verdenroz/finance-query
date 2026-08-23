# CFTC (Commitments of Traders)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["cftc"] }
    ```

Nothing in the library exposed futures positioning data before this adapter: `Capability::FUTURES` previously meant Polygon-only price quotes. The [CFTC](https://www.cftc.gov/) publishes its weekly Commitments of Traders report itself, keylessly, through `publicreporting.cftc.gov`'s Socrata API. This adapter serves the disaggregated futures-only combined report, the one most commonly meant by "COT data" for physical commodities.

## Scope

Only physical-commodity futures are covered (agriculture, energy, metals) via a curated table of benchmark contracts, or any raw `cftc_contract_market_code` passed straight through. Financial futures (equity indices, rates, currencies) are reported separately by the CFTC in the Traders in Financial Futures report, which this adapter does not serve.

The CFTC itself has no price-quote data at all, so `Provider::Cftc` only ever answers `FuturesContract::commitments_of_traders`, reporting `NotSupported` for a plain futures quote.

## Setup

```rust no_run feature=cftc
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FUTURES, [Provider::Cftc, Provider::Yahoo])
        .build()
        .await?;

    let gold = providers.futures("GC=F");
    let cot = gold.commitments_of_traders().await?;

    println!("{} ({})", cot.market_and_exchange_name, cot.cftc_contract_market_code);
    for week in cot.observations.iter().rev().take(4) {
        println!(
            "{}: open interest {:?}, managed money long/short {:?}/{:?}",
            week.report_date, week.open_interest, week.managed_money_long, week.managed_money_short
        );
    }
    Ok(())
}
```

!!! note "CFTC serves positioning only"
    `fetch_futures_quote` reports `NotSupported`, so a `FUTURES` route that includes CFTC should also include a quoting provider, as in the example above, for `quote()` to resolve.

## Symbol Resolution

`commitments_of_traders()` resolves the handle's symbol to a CFTC `cftc_contract_market_code` through a small curated table of Yahoo-style continuous futures roots:

| Symbol | Contract | CFTC code |
|--------|----------|-----------|
| `GC=F` | Gold | `088691` |
| `SI=F` | Silver | `084691` |
| `PL=F` | Platinum | `076651` |
| `HG=F` | Copper | `085692` |
| `CL=F` | WTI crude | `067651` |
| `NG=F` | Henry Hub natural gas | `03565B` |
| `ZC=F` | Corn | `002602` |
| `ZW=F` | Wheat (SRW) | `001602` |
| `ZS=F` | Soybeans | `005602` |

Anything not in this table is treated as a literal CFTC contract code already: the passthrough form, e.g. `providers.futures("067651")` for NYMEX WTI directly, or any other code from CFTC's own market list.

## What the Numbers Mean

Each weekly `CotObservation` breaks total open interest down by trader category: commercial hedgers (producer/merchant), swap dealers, managed money (large speculators), other reportables, and the nonreportable residual below CFTC's reporting thresholds. A field the report doesn't carry for that week stays `None` rather than becoming `0`.

Observations come back oldest-first, matching every other history-shaped model in the library, even though the Socrata API itself answers newest-first.

A contract code with no rows returns `SymbolNotFound` rather than an empty series. CFTC's Socrata endpoint answers an unrecognised `cftc_contract_market_code` with zero rows, and there's no way to distinguish that from a valid but quiet code, so it's treated as not found.

## Rate Limits

Socrata's anonymous tier throttles by rolling-hour request count rather than per-second. The client paces at 5 requests/second to avoid looking like a burst.

## Next Steps

- [Futures](../futures.md): the `FuturesContract` handle
- [Providers Overview](index.md): routing and fallback
