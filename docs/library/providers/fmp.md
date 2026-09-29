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

`Ticker::company_profile()` supports FMP through the FUNDAMENTALS route. It returns the provider symbol, company CIK, and IPO date together. Missing identifiers and dates remain absent; empty, ambiguous or mismatched profiles return errors. The profile is current company information, not a historical listing snapshot.

Pass FMP's symbol spelling, such as `BRK-B`. Applications comparing providers must retain their own canonical symbol and verify the returned identity. The crate does not infer that two symbols represent the same security.

The profile response is bounded to 1 MiB. See [the runnable stock example](../../../examples/stock_ingestion.rs) for a call through the public API.

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

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .providers([Provider::Fmp])
    .route(Capability::DISCOVERY, [Provider::Fmp])
    .build().await?;
let delisted = providers.discovery().listing_status(false).await?;
for stock in delisted {
    println!("{}: IPO {:?}, delisted {:?}", stock.symbol, stock.ipo_date, stock.delisted_date);
}
# Ok(())
# }
```

### Resumable delisted pages

`Discovery::delisted_stocks_page_at(page, limit)` fetches an independent numbered
page through the same configured provider transport and account limiter. Page
numbers are zero-based and below 10,000; limits are 1–100. This method makes one
request and preserves IPO/delisting dates. It rejects malformed rows and responses
larger than the requested limit. Use it for bounded concurrent scans; callers must
track every page, reject repeated/nonadvancing records, and verify the empty end
page. It does not provide the sequential cursor's scan-progress checks.

`Discovery::delisted_stocks_page(limit, cursor)` returns one `ProviderPage<SymbolMatch>`.
Use a page size of 1–1000. Persist its records and cursor together, then supply
the cursor unchanged to the next call. The cursor binds the provider, operation
and page size and contains no credentials. The aggregate `listing_status(false)`
uses the same paging primitive and remains compatible.

FMP can cap the requested page size: the bounded account probe returned 100
records even for `limit=1000`. Continue until an empty terminal page rather than
assuming a short response is the end.

Empty terminal pages, failures and repeated/nonadvancing pages are distinct.
Each response is bounded to 8 MiB; traversal is bounded to 10,000 nonempty pages
and a 16 MiB cursor. Exceeding a bound returns an error, never a truncated
successful list. Record fingerprints use a fixed algorithm so continuations
survive process reconstruction. Mutable provider lists are not historical
snapshots; retain coverage and source observations in the consuming application.

### Stock directories and bulk profiles

`Discovery::stock_list()` fetches `/stable/stock-list`, the global directory,
including records whose trading status is unknown. `listing_status(true)` now
uses `/stable/actively-trading-list` instead of combining ETF and mutual-fund
lists. Both endpoints can include non-US instruments and funds. They commonly
provide only symbol and name; missing exchange/type/identity fields stay absent.
Use profile or Polygon reference data to classify the intended stock universe.

`Discovery::company_profiles_bulk(part)` fetches exactly one explicit numbered
`/stable/profile-bulk` part. It accepts CSV (including quoted commas/newlines and
UTF-8 BOM) or JSON arrays and returns `Vec<CompanyProfile>`. The same profile
mapping serves individual requests: CIK, CUSIP, ISIN, IPO date, exchange and
active/ETF/ADR/fund flags are preserved when supplied. CIK identifies a company,
not a distinct share class. These are current profiles, not dated history.

```rust no_run feature=fmp
# async fn example(providers: &finance_query::Providers) -> finance_query::Result<()> {
let directory = providers.discovery().stock_list().await?;
let active = providers.discovery().listing_status(true).await?;
let profiles = providers.discovery().company_profiles_bulk(0).await?;
# Ok(())
# }
```

Directory responses are bounded to 32 MiB and bulk parts to 128 MiB (decoded
bytes). Parsing large CSV files runs off the async worker threads. Configure
`Providers::builder().timeout(...)` for large transfers; the default is 30 seconds.
Blank bodies, malformed rows, denied access, rate limits, server and transport
failures are errors, never successful empty parts. Header-only CSV or JSON `[]`
is an empty part. Bulk results are uncached and no further parts are fetched
implicitly; an empty part alone is not a guarantee of provider-wide completeness.

The crate applies the configured shared account rate; callers must additionally
space profile-bulk calls at least 60 seconds apart, as specified in the
[FMP FAQ](https://site.financialmodelingprep.com/faqs). Its
[part documentation](https://site.financialmodelingprep.com/it/faqs?code=marketPerformance)
currently describes four parts (0 through 3). Retrieve each required part
explicitly; part 0 alone is not the whole dataset. Verify account access and
field coverage independently.

For a fixed 24-symbol comparison, run the `fmp_catalog_benchmark` example with
`--features 'polygon fmp'`. Modes are `single`, `directories`, `bulk <part>`, and
`polygon <YYYY-MM-DD>`. Export the relevant key before starting. It reports
request count (no retries), elapsed time, returned fields and missing symbols;
the bulk mode makes only one request with a 180-second timeout. The two delisted
symbols in its fixture intentionally test missing-current-profile behavior.

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
