# Wikipedia (Index Constituents)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["wikipedia"] }
    ```

`Capability::INDICES`' `IndexConstituents` operation previously needed a keyed provider (FMP or Polygon). This adapter table-scrapes ["List of S&P 500 companies"](https://en.wikipedia.org/wiki/List_of_S%26P_500_companies), keylessly, for the S&P 500's member list.

## Scope

Only `MajorIndex::Sp500` is served. The Nasdaq-100 and Dow Jones Wikipedia articles list their components only inside a navbox template at the bottom of the page (ticker links grouped by sector, no headquarters/CIK/founding-year columns) — a meaningfully thinner shape not worth a bespoke parser for. Both return `NotSupported` on this provider; route to [FMP](fmp.md) for those two instead.

Wikipedia carries no constituent-*changes* history table for any index in this set either, so `IndexConstituentChanges` is unrouted here regardless of index — FMP is the only source for that operation.

`fetch_indices_quote` also reports `NotSupported`: Wikipedia has no price data at all, so an `INDICES` route that includes it should also include a quoting provider (Yahoo, the default, already covers this).

## Setup

```rust no_run feature=wikipedia
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Wikipedia])
        .build()
        .await?;

    let spx = providers.index("^GSPC");
    for member in spx.constituents().await?.iter().take(5) {
        println!(
            "{}: {} ({})",
            member.symbol,
            member.name.as_deref().unwrap_or("?"),
            member.sector.as_deref().unwrap_or("?")
        );
    }
    Ok(())
}
```

## What the Fields Mean

`IndexConstituent`'s eight columns come straight off the Wikipedia table's eight columns, in order — symbol, company name, GICS sector, GICS sub-industry, headquarters location, date added, CIK, and founding year. See [Indices](../indices.md#indexconstituent-fields) for the full field table.

## Rate Limits

Wikipedia documents no formal quota for article fetches. These are static, rarely-changing pages, so the client self-paces conservatively at 1 request/second regardless.

## Next Steps

- [Indices](../indices.md): the `Index` handle, `.constituents()`, and the full field reference
- [FMP Provider](fmp.md): keyed alternative covering all three major indices plus constituent-change history
- [Providers Overview](index.md): routing and fallback
