# FRED & Treasury Yields

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

```rust
use finance_query::fred;

// Initialize with API key
fred::init("your-fred-api-key")?;

// Optional: initialize with a custom timeout
use std::time::Duration;
fred::init_with_timeout("your-fred-api-key", Duration::from_secs(60))?;
```

!!! warning
    Calling `init` more than once returns an error. Call it exactly once per process, typically at startup.

## Fetching FRED Series

```rust
use finance_query::fred;

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
```

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

**`MacroSeries` fields:**

- `id: String` — the FRED series ID
- `observations: Vec<MacroObservation>` — chronologically ordered data points

**`MacroObservation` fields:**

- `date: String` — date as `YYYY-MM-DD`
- `value: Option<f64>` — `None` when FRED reports a missing value

**Rate limit:** 2 requests/second (enforced automatically).

## US Treasury Yields

No initialization required. Fetches directly from the US Treasury Department:

```rust
use finance_query::fred;

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
```

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

## Example: Yield Curve Inversion Check

```rust
use finance_query::fred;

let yields = fred::treasury_yields(2025).await?;

for y in yields.iter().rev().take(5) {
    if let (Some(y2), Some(y10)) = (y.y2, y.y10) {
        let spread = y10 - y2;
        let label = if spread < 0.0 { "INVERTED" } else { "normal" };
        println!("{}: 10Y-2Y spread = {:.2}bps ({})", y.date, spread * 100.0, label);
    }
}
```

## Next Steps

- [Finance Module](finance.md) - Market-wide data functions
- [Getting Started](getting-started.md) - Feature flag setup
