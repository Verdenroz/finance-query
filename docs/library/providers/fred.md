# FRED & Treasury Yields

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — fred](https://docs.rs/finance-query/latest/finance_query/fred/index.html)

!!! info "Feature flag required"
    Add `fred = ["dep:csv"]` to your `Cargo.toml` features to enable this module.
    ```toml
    finance-query = { version = "...", features = ["fred"] }
    ```

The `fred` module provides two macro-economic data sources:

- **FRED** (Federal Reserve Economic Data) — 800k+ time series including CPI, GDP, unemployment, and monetary indicators. Requires a free API key.
- **US Treasury yields** — Daily yield curve data from the US Treasury Department. No API key required.

## FRED Setup

Get a free API key at [fred.stlouisfed.org](https://fred.stlouisfed.org/docs/api/api_key.html), then call `fred::init` once at application startup:

```rust no_run feature=fred
use finance_query::fred;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with API key
    fred::init("your-fred-api-key")?;

    // Or, instead: initialize with a custom timeout (pick exactly one)
    fred::init_with_timeout("your-fred-api-key", Duration::from_secs(60))?;
    Ok(())
}
```

!!! warning
    Calling `init` more than once returns an error. Call it exactly once per process, typically at startup.

The client is a process-wide singleton, so the second call always fails. This example runs as a real test:

```rust capture-output feature=fred
use finance_query::fred;

let _ = fred::init("api-key");
let second_init = fred::init("another-api-key");
assert!(second_init.is_err());
println!("second init is_err = {}", second_init.is_err());
```

```text soothfast-output
second init is_err = true
```

## Fetching FRED Series

```rust no_run feature=fred covers=finance_query::models::economic::MacroSeries
use finance_query::fred;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fred::init("your-fred-api-key")?;

    // Fetch all observations for a series
    let cpi = fred::series("CPIAUCSL").await?;

    println!("Series: {}", cpi.id);
    println!("Observations: {}", cpi.observations.len());

    // Print the last 5 observations
    for obs in cpi.observations.iter().rev().take(5) {
        match obs.value {
            Some(v) => println!("{}: {:.2}", obs.date, v),
            None    => println!("{}: N/A", obs.date),
        }
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::de_fred_series.walltime.median_ns < 300000 -->
- Parsing a full FRED series response (decades of observations) into
  `MacroSeries` takes **under 300 µs**.

**Common FRED Series IDs:**

| Series ID | Description |
|-----------|-------------|
| `"FEDFUNDS"` | Federal Funds Effective Rate |
| `"CPIAUCSL"` | Consumer Price Index (all urban, seasonally adjusted) |
| `"CPILFESL"` | Core CPI (less food and energy) |
| `"UNRATE"` | Unemployment Rate |
| `"GDP"` | Gross Domestic Product |
| `"M2SL"` | M2 Money Supply |
| `"DGS10"` | 10-Year Treasury Constant Maturity Rate |
| `"DGS2"` | 2-Year Treasury Constant Maturity Rate |
| `"T10Y2Y"` | 10-Year minus 2-Year Treasury spread |
| `"INDPRO"` | Industrial Production Index |
| `"HOUST"` | Housing Starts |
| `"PAYEMS"` | Total Nonfarm Payrolls |
| `"PCE"` | Personal Consumption Expenditures |

<!-- soothfast:bind finance_query::models::economic::MacroSeries -->

**`MacroSeries` fields:**

- `id: String` — the FRED series ID
- `observations: Vec<MacroObservation>` — chronologically ordered data points

<!-- /soothfast:bind -->

<!-- soothfast:bind finance_query::models::economic::MacroObservation -->

**`MacroObservation` fields:**

- `date: String` — date as `YYYY-MM-DD`
- `value: Option<f64>` — `None` when FRED reports a missing value

<!-- /soothfast:bind -->

**Rate limit:** 2 requests/second (enforced automatically).

## Finding Series (`EconomicCatalog`)

`fred::series(id)` and `providers.economic(id)` both require an id you already
know. `providers.economic_catalog()` is how you find one:

```rust,ignore
use finance_query::Providers;

let providers = Providers::builder().build().await?;
let catalog = providers.economic_catalog();

// Free-text search, most popular first.
for hit in catalog.search("real gross domestic product", 10).await? {
    println!("{} — {:?} ({:?})", hit.id, hit.title, hit.frequency);
}

// Browse the category tree; 0 is the root.
for cat in catalog.categories(0).await? {
    println!("{} {:?}", cat.id, cat.name);
}

// Every scheduled release FRED publishes.
let releases = catalog.releases().await?;
```

## Point-in-Time Data (ALFRED vintages)

FRED revises macro data after publication, so backtesting a rule against
today's `GDPC1` is look-ahead bias — the values it trades on were not knowable
at the time. `.as_of(date)` asks for the vintage that was actually published:

```rust,ignore
let gdp = providers.economic("GDPC1");

let revised = gdp.series().await?;              // as currently revised
let vintage = gdp.as_of("2020-06-30").await?;   // as published on that date
```

Both realtime bounds are pinned to `date`, so the response contains exactly the
values in force that day rather than a range of revisions. Results are cached
per date.

## US Treasury Yields

No initialization required. Fetches directly from the US Treasury Department:

```rust no_run feature=fred covers=finance_query::models::economic::TreasuryYield
use finance_query::fred;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch the full yield curve for a given year
    let yields = fred::treasury_yields(2025).await?;

    // Print the most recent day
    if let Some(latest) = yields.last() {
        println!("Date: {}", latest.date);
        println!("2Y:  {:?}%", latest.y2);
        println!("5Y:  {:?}%", latest.y5);
        println!("10Y: {:?}%", latest.y10);
        println!("30Y: {:?}%", latest.y30);
    }
    Ok(())
}
```

<!-- soothfast:bind finance_query::models::economic::TreasuryYield -->

**`TreasuryYield` fields** (all yields are `Option<f64>` in %):

| Field | Maturity |
|-------|---------|
| `y1m` | 1 month |
| `y2m` | 2 months |
| `y3m` | 3 months |
| `y4m` | 4 months |
| `y6m` | 6 months |
| `y1` | 1 year |
| `y2` | 2 years |
| `y3` | 3 years |
| `y5` | 5 years |
| `y7` | 7 years |
| `y10` | 10 years |
| `y20` | 20 years |
| `y30` | 30 years |

Dates are formatted as `MM/DD/YYYY` (the Treasury's native format). Fields are `None` on days when that maturity is not published.

<!-- /soothfast:bind -->

## Example: Yield Curve Inversion Check

```rust no_run feature=fred covers=finance_query::de_treasury_yields
use finance_query::fred;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yields = fred::treasury_yields(2025).await?;

    for y in yields.iter().rev().take(5) {
        if let (Some(y2), Some(y10)) = (y.y2, y.y10) {
            let spread = y10 - y2;
            let label = if spread < 0.0 { "INVERTED" } else { "normal" };
            println!("{}: 10Y-2Y spread = {:.2}bps ({})", y.date, spread * 100.0, label);
        }
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::de_treasury_yields.walltime.median_ns < 200000 -->
- Parsing a daily Treasury yield-curve payload (`fred::treasury_yields`)
  takes **under 200 µs**.

## Next Steps

- [Finance Module](../finance.md) - Market-wide data functions
- [Getting Started](../getting-started.md) - Feature flag setup
