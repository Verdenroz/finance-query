# Economic Indicators

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — EconomicIndicator](https://docs.rs/finance-query/latest/finance_query/struct.EconomicIndicator.html)

!!! info "Feature flag required"
    The `economic` domain requires at least one provider feature. FRED is the primary recommended provider:
    ```toml
    finance-query = { version = "...", features = ["fred"] }
    ```

The `EconomicIndicator` domain handle fetches macro-economic time series data. It is backed by [FRED](providers/fred.md) (Federal Reserve Economic Data), with optional Alpha Vantage and Polygon fallbacks, and is keyed by a series ID string.

## Getting a Handle

Create an `EconomicIndicator` handle from a [`Providers`](getting-started.md) instance by routing the `ECONOMIC` capability to FRED, then call `.series()` to fetch the data:

```rust no_run feature=fred
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::Fred])
        .build()
        .await?;
    let gdp = providers.economic("GDP");
    let series = gdp.series().await?;
    println!("{}: {} observations", series.series_id, series.observations.len());
    Ok(())
}
```

The series ID is a FRED series identifier such as `"GDP"`, `"FEDFUNDS"`, or `"CPIAUCSL"`. See the [FRED series catalog](https://fred.stlouisfed.org/categories) for the full list of 800k+ available series.

!!! note "FRED API Key"
    FRED requires an API key. Initialise it once before building your `Providers` instance:
    ```rust no_run feature=fred
    use finance_query::fred;

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        fred::init(std::env::var("FRED_API_KEY")?)?;
        Ok(())
    }
    ```
    See [FRED Provider Reference](providers/fred.md) for setup details.

<!-- soothfast:claim finance_query::de_fred_series.walltime.median_ns < 500000 -->
- Deserializing a full FRED series response (`fred::series` →
  `MacroSeries`) completes in **well under half a millisecond**.

## `EconomicSeries` Fields

The returned [`EconomicSeries`](https://docs.rs/finance-query/latest/finance_query/struct.EconomicSeries.html) value contains:

<!-- soothfast:bind finance_query::models::economic::EconomicSeries -->

| Field | Type | Description |
|-------|------|-------------|
| `series_id` | `String` | Series identifier (e.g., `"GDP"`, `"FEDFUNDS"`) |
| `title` | `Option<String>` | Human-readable series title |
| `units` | `Option<String>` | Unit of measurement (e.g., `"Billions of Dollars"`, `"Percent"`) |
| `frequency` | `Option<String>` | Reporting frequency (e.g., `"Annual"`, `"Monthly"`) |
| `observations` | `Vec<MacroObservation>` | Chronologically ordered observations |

<!-- /soothfast:bind -->

Each `MacroObservation` has:

<!-- soothfast:bind finance_query::models::economic::MacroObservation -->

| Field | Type | Description |
|-------|------|-------------|
| `date` | `String` | Observation date as `YYYY-MM-DD` |
| `value` | `Option<f64>` | Observation value; `None` when FRED reports a missing value |

<!-- /soothfast:bind -->

Both tables are backed by a compiled, real value — `EconomicSeries` and
`MacroObservation` are `#[non_exhaustive]` and have no public constructor, but
both derive `Deserialize`, so a fixture value can be built from outside the
crate with `serde_json::from_value`:

```rust capture-output feature=fred covers=finance_query::models::economic::EconomicSeries
use finance_query::EconomicSeries;
use finance_query::fred::MacroObservation;

fn main() {
    let series: EconomicSeries = serde_json::from_value(serde_json::json!({
        "series_id": "GDP",
        "title": "Gross Domestic Product",
        "units": "Billions of Dollars",
        "frequency": "Quarterly",
        "observations": [
            {"date": "2024-01-01", "value": 28624.1},
            {"date": "2024-04-01", "value": 29016.7}
        ]
    }))
    .unwrap();

    let first: &MacroObservation = &series.observations[0];
    println!("series_id = {}", series.series_id);
    println!("title = {:?}", series.title);
    println!("first observation = {} -> {:?}", first.date, first.value);
}
```

```text soothfast-output
series_id = GDP
title = Some("Gross Domestic Product")
first observation = 2024-01-01 -> Some(28624.1)
```

## Example: Inspecting GDP Data

```rust no_run feature=fred covers=finance_query::models::economic::MacroObservation
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::Fred])
        .build()
        .await?;

    let gdp = providers.economic("GDP");
    let series = gdp.series().await?;

    println!("Series:    {}", series.series_id);
    if let Some(title) = &series.title {
        println!("Title:     {}", title);
    }
    if let Some(units) = &series.units {
        println!("Units:     {}", units);
    }
    if let Some(freq) = &series.frequency {
        println!("Frequency: {}", freq);
    }
    println!("Observations: {}", series.observations.len());

    // Print the last 5 observations
    for obs in series.observations.iter().rev().take(5) {
        match obs.value {
            Some(v) => println!("{}: {:.1}", obs.date, v),
            None => println!("{}: N/A", obs.date),
        }
    }
    Ok(())
}
```

## See Also

- [FRED Provider Reference](providers/fred.md) — low-level FRED API (`fred::series`, `fred::treasury_yields`, initialisation)
- [Getting Started](getting-started.md) — building a `Providers` instance and routing capabilities
