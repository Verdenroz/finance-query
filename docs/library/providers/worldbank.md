# World Bank Open Data

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["worldbank"] }
    ```

[World Bank Open Data](https://data.worldbank.org/) serves roughly 1,600 development and macro-economic indicators across 200+ economies. It is **keyless** — no registration, no API key, no environment variable.

It complements [FRED](fred.md), which is US-centric and needs a key: when you want GDP, inflation, population, or trade figures for economies outside the United States, route `Capability::ECONOMIC` here.

## Setup

```rust
use finance_query::{Capability, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::ECONOMIC, [Provider::WorldBank])
    .build()
    .await?;
```

## Series Identifiers

The `ECONOMIC` capability addresses a series with a single string, so a World Bank series is written as `"<COUNTRY>/<INDICATOR>"`:

```rust
let gdp = providers.economic("USA/NY.GDP.MKTP.CD").series().await?;

println!("{}", gdp.title.unwrap_or_default());   // "GDP (current US$) — United States"
for obs in &gdp.observations {
    println!("{} {:?}", obs.date, obs.value);    // "1960-01-01" Some(543300000000.0)
}
```

- **Country** — an ISO-2 (`US`) or ISO-3 (`USA`) code, a World Bank aggregate (`WLD`, `EMU`, `OED`), or `all`.
- **Indicator** — a World Bank indicator code such as `NY.GDP.MKTP.CD`. Browse them at [data.worldbank.org/indicator](https://data.worldbank.org/indicator).

Omitting the country resolves against the world aggregate:

```rust
// Equivalent to "WLD/SP.POP.TOTL" — world population
let world_pop = providers.economic("SP.POP.TOTL").series().await?;
```

### Common Indicators

| Indicator code | Description |
|----------------|-------------|
| `NY.GDP.MKTP.CD` | GDP (current US$) |
| `NY.GDP.MKTP.KD.ZG` | GDP growth (annual %) |
| `NY.GDP.PCAP.CD` | GDP per capita (current US$) |
| `FP.CPI.TOTL.ZG` | Inflation, consumer prices (annual %) |
| `SL.UEM.TOTL.ZS` | Unemployment (% of total labour force) |
| `SP.POP.TOTL` | Population, total |
| `NE.EXP.GNFS.ZS` | Exports of goods and services (% of GDP) |
| `GC.DOD.TOTL.GD.ZS` | Central government debt (% of GDP) |

## Response Shape

Results come back as the provider-neutral [`EconomicSeries`](../economic.md), the same type FRED, Alpha Vantage, and Polygon return:

| Field | Value from World Bank |
|-------|----------------------|
| `series_id` | The string you passed in |
| `title` | Indicator name and country, e.g. `"GDP (current US$) — United States"` |
| `units` | The API's `unit` field when it is populated (usually empty — most indicators state their unit in the title) |
| `frequency` | `"Annual"`, `"Quarterly"`, or `"Monthly"`, inferred from the period labels |
| `observations` | Chronological (oldest first), with `value: None` for periods the World Bank has no figure for |

World Bank period labels (`2023`, `2023Q1`, `2023M04`) are normalised to the `YYYY-MM-DD` **start** of the period so dates sort and parse like every other provider's.

## Fallback Chains

Because it is keyless, World Bank makes a good last resort behind a keyed provider:

```rust
let providers = Providers::builder()
    .route(Capability::ECONOMIC, [Provider::Fred, Provider::WorldBank])
    .build()
    .await?;
```

Note that the two use different identifier schemes — a FRED series id is not a World Bank series id — so a chain like this is useful for *availability*, not for transparently retrying the same identifier.

## Rate Limits

The World Bank publishes no documented quota. The client paces itself at 5 requests/second, and a `429` surfaces as [`FinanceError::RateLimited`](../error-handling.md).

## Next Steps

- [FRED](fred.md) — 800k+ US macro series
- [Economic Domain](../economic.md) — the `EconomicIndicator` handle
- [Providers Overview](index.md) — routing and fallback
