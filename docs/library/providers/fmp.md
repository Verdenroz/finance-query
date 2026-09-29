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

`Ticker::company_profile()` supports FMP through the FUNDAMENTALS route. It returns the provider symbol, company CIK, and IPO date together. Missing identifiers and dates, including FMP's empty strings, remain absent; empty, ambiguous or mismatched profiles return errors. The profile is current company information, not a historical listing snapshot.

Pass FMP's symbol spelling, such as `BRK-B`. Applications comparing providers must retain their own canonical symbol and verify the returned identity. The crate does not infer that two symbols represent the same security.

### Delisted companies

`Discovery::listing_status(false)` reads every FMP `/stable/delisted-companies`
page until an empty page. It returns `SymbolMatch` records with `ipo_date` and
`delisted_date` preserved. Missing dates stay `None`; reused ticker symbols can
have separate records with different dates. The endpoint does not provide a
stable security identifier, so `id` remains `None`.

A failed request fails the whole call instead of returning or caching a partial
list. Repeated pages that provide no new records also return an error. The
complete result is held in memory and cached on the discovery handle.

```rust no_run feature=fmp
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .providers([Provider::Fmp])
        .route(Capability::DISCOVERY, [Provider::Fmp])
        .build()
        .await?;
    let delisted = providers.discovery().listing_status(false).await?;
    for stock in delisted {
        println!(
            "{}: IPO {:?}, delisted {:?}",
            stock.symbol, stock.ipo_date, stock.delisted_date
        );
    }
    Ok(())
}
```

### Resumable delisted pages

These page operations, the directories and bulk profiles below are library-only;
the REST, GraphQL and MCP servers do not expose them.

`Discovery::delisted_stocks_page(limit, cursor)` returns one `ProviderPage<SymbolMatch>`
of 1–100 rows. Persist its records and cursor together, then pass the cursor back
unchanged with the same `limit`. The cursor is bound to FMP, the operation and the
page size, contains no credentials, and resumes after a restart. Continue until
an empty page rather than treating a short page as the end. A repeated page or a
page with no new records is an error, distinct from the empty terminal page.
`listing_status(false)` pages through the same primitive.

`Discovery::delisted_stocks_page_at(page, limit)` fetches one independent
zero-based page, below page 10,000 with 1–100 rows, and preserves IPO and
delisting dates. Use it for bounded concurrent scans. It has none of the cursor's
progress checks, so callers must track every page, reject repeated records and
confirm the empty end page.

Responses and traversal are bounded, and exceeding a bound returns an error
rather than a truncated list. Provider lists change over time and are not
historical snapshots; keep coverage and source observations in the application.

### Stock directories and bulk profiles

`Discovery::stock_list()` fetches `/stable/stock-list`, the global directory,
including records whose trading status is unknown. `listing_status(true)` reads
`/stable/actively-trading-list`. Both can include non-US instruments and funds,
and they commonly provide only the symbol and name, so missing exchange, type and
identity fields stay absent. Use profiles or Polygon reference data to classify
the intended stock universe.

`Discovery::company_profiles_bulk(part)` fetches exactly one numbered
`/stable/profile-bulk` part. It accepts CSV (including quoted commas, newlines and
a UTF-8 BOM) or a JSON array and returns `Vec<CompanyProfile>`, mapped like
individual profiles: CIK, CUSIP, ISIN, IPO date, exchange and the active, ETF,
ADR and fund flags are kept when supplied. CIK identifies a company, not a share
class, and these are current profiles, not dated history.

```rust no_run feature=fmp
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .providers([Provider::Fmp])
        .route(Capability::DISCOVERY, [Provider::Fmp])
        .build()
        .await?;
    let directory = providers.discovery().stock_list().await?;
    let active = providers.discovery().listing_status(true).await?;
    let profiles = providers.discovery().company_profiles_bulk(0).await?;
    println!("{} listed, {} active, {} profiles", directory.len(), active.len(), profiles.len());
    Ok(())
}
```

Directories and bulk parts are slow downloads, so these calls wait at least two
and ten minutes respectively, or longer if `Providers::builder().timeout(...)`
is set higher. A bulk part is about 30 MB and FMP sends it uncompressed. Large CSV parts parse off the async worker threads. Blank bodies, malformed rows, denied access, rate limits, server and
transport failures are errors, never empty parts. A header-only CSV or `[]` is an
empty part. Bulk results are not cached, no further parts are fetched implicitly,
and an empty part does not prove the dataset is complete.

FMP asks callers to space profile-bulk calls at least 60 seconds apart
([FMP FAQ](https://site.financialmodelingprep.com/faqs)); the crate applies only
the configured account rate. The
[part documentation](https://site.financialmodelingprep.com/it/faqs?code=marketPerformance)
currently describes four parts (0 through 3), so retrieve each part you need
explicitly.

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
