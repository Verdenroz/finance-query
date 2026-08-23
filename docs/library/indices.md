# Indices

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Index](https://docs.rs/finance-query/latest/finance_query/struct.Index.html)

The `Index` handle provides access to stock market index data (quotes, charts, constituents) for major indices like the S&P 500, Nasdaq-100, and Dow Jones. Index quotes are served by Yahoo Finance (keyless — no feature flag, no API key) on the default route. Polygon and FMP remain available as alternate quote providers, and FMP/Wikipedia serve constituent data.

## Setup

`Providers::builder().build()` with no `.route()` call already serves index quotes — Yahoo is the default for every capability. Yahoo resolves index symbols the same way it resolves equities (the caret-prefixed form, e.g. `"^GSPC"` for the S&P 500):

```rust no_run
use finance_query::{Interval, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let spx = providers.index("^GSPC");
    let quote = spx.quote().await?;
    let chart = spx.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let history = spx.history(TimeRange::OneMonth).await?;
    println!("{}: {:?} ({} candles)", quote.symbol, quote.price, chart.candles.len() + history.candles.len());
    Ok(())
}
```

## Yahoo Index Symbols

Yahoo uses the caret-prefixed ticker form:

| Symbol | Index |
|--------|-------|
| `^GSPC` | S&P 500 |
| `^NDX` | NASDAQ-100 |
| `^DJI` | Dow Jones Industrial Average |
| `^RUT` | Russell 2000 |
| `^VIX` | CBOE Volatility Index |

## Alternative: Polygon or FMP

Route `Capability::INDICES` to `Provider::Polygon` or `Provider::Fmp` for an alternate quote source, or as a fallback behind Yahoo:

```rust no_run feature=polygon
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Polygon, Provider::Yahoo])
        .build()
        .await?;
    let spx = providers.index("I:SPX");
    let quote = spx.quote().await?;
    println!("S&P 500: {:?}", quote.price);
    Ok(())
}
```

Set `POLYGON_API_KEY` in your environment before calling `build()`.

Polygon indices use an `I:<TICKER>` prefix instead of Yahoo's caret form:

| Symbol | Index |
|--------|-------|
| `I:SPX` | S&P 500 |
| `I:NDX` | NASDAQ-100 |
| `I:DJI` | Dow Jones Industrial Average |
| `I:RUT` | Russell 2000 |
| `I:VIX` | CBOE Volatility Index |

!!! note "`chart`/`history` route through `Capability::CHART`, not `INDICES`"
    `quote()` follows the `INDICES` route, but `chart()`/`history()` always
    dispatch through `Capability::CHART` (Yahoo by default). If you route
    `INDICES` to Polygon for its `I:SPX`-style quotes, either keep using a
    Yahoo-style symbol (`^GSPC`) for `chart()`/`history()`, or also route
    `CHART` to Polygon if you want candles in the same symbol space.

## Methods

### `quote()`

Fetch the current snapshot for the index.

```rust no_run covers=finance_query::models::indices::IndexQuote
use finance_query::Providers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let spx = providers.index("^GSPC");
    let quote = spx.quote().await?;
    println!("S&P 500: {:?}", quote.price);
    Ok(())
}
```

### `chart(interval, range)`

Fetch OHLCV candles for a specific `Interval` and `TimeRange`.

```rust no_run covers=finance_query::models::chart::data::Chart
use finance_query::{Interval, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let spx = providers.index("^GSPC");
    let chart = spx.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    println!("Candles: {}", chart.candles.len());
    Ok(())
}
```

### `history(range)`

Shorthand for `chart` using the default interval for the given `TimeRange`.

```rust no_run
use finance_query::{Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let spx = providers.index("^GSPC");
    let history = spx.history(TimeRange::OneMonth).await?;
    println!("Candles: {}", history.candles.len());
    Ok(())
}
```

### `constituents()`

Fetch the current member list of a major index (S&P 500, Nasdaq-100, or Dow Jones — derived from the handle's symbol via `MajorIndex::from_symbol`). Two providers serve this:

- **Wikipedia** (`wikipedia` feature, keyless) — S&P 500 only. The Nasdaq-100 and Dow Jones Wikipedia articles list constituents only inside a navbox template rather than a proper table, a thinner shape not worth a bespoke parser for, so those two stay `NotSupported` on this provider.
- **FMP** (`fmp` feature, keyed) — all three major indices, plus [`constituent_changes()`](#constituent_changes) history that Wikipedia doesn't serve at all.

```rust no_run feature=wikipedia covers=finance_query::models::indices::IndexConstituent
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Wikipedia])
        .build()
        .await?;
    let spx = providers.index("^GSPC");
    for member in spx.constituents().await?.iter().take(5) {
        println!("{}: {}", member.symbol, member.name.as_deref().unwrap_or("?"));
    }
    Ok(())
}
```

For Nasdaq-100 or Dow Jones constituents, route to FMP instead:

```rust no_run feature=fmp
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Fmp])
        .build()
        .await?;
    let ndx = providers.index("^NDX");
    let members = ndx.constituents().await?;
    println!("Nasdaq-100 constituents: {}", members.len());
    Ok(())
}
```

### `constituent_changes()`

Fetch historical additions/removals for a major index. FMP only (`fmp` feature) — Wikipedia carries no constituent-change history table for any index in this set.

```rust no_run feature=fmp covers=finance_query::models::indices::IndexConstituentChange
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Fmp])
        .build()
        .await?;
    let spx = providers.index("^GSPC");
    for change in spx.constituent_changes().await?.iter().take(5) {
        println!(
            "{}: +{} -{}",
            change.date.as_deref().unwrap_or("?"),
            change.added_security.as_deref().unwrap_or("-"),
            change.removed_ticker.as_deref().unwrap_or("-")
        );
    }
    Ok(())
}
```

### `indicators(interval, range)` / `indicator(kind, interval, range)` / `risk(interval, range)`

Compute technical indicators or a risk summary from this index's own chart
data (requires the `indicators`/`risk` features respectively).

```rust no_run feature=risk
use finance_query::indicators::Indicator;
use finance_query::{Interval, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let spx = providers.index("^GSPC");
    let summary = spx.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;
    if let Some(rsi) = summary.rsi_14 {
        println!("RSI(14): {:.2}", rsi);
    }

    let rsi_21 = spx
        .indicator(Indicator::Rsi(21), Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    let risk = spx.risk(Interval::OneDay, TimeRange::OneYear).await?;
    println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
    println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
    Ok(())
}
```

`risk` takes no benchmark parameter — `beta` is always `None`, since indices
have no natural benchmark to compare against.

## `IndexQuote` Fields

<!-- soothfast:bind finance_query::models::indices::IndexQuote -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Index ticker symbol (e.g., `"^GSPC"`) |
| `name` | `Option<String>` | Human-readable index name (e.g., `"S&P 500"`) |
| `price` | `Option<f64>` | Current index value |
| `change` | `Option<f64>` | Absolute price change |
| `change_percent` | `Option<f64>` | Percentage price change |
| `timestamp` | `Option<i64>` | Unix timestamp of last update |

<!-- /soothfast:bind -->

## `Chart` Fields

<!-- soothfast:bind finance_query::models::chart::data::Chart -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Index symbol |
| `candles` | `Vec<Candle>` | OHLCV candle data |
| `interval` | `Option<Interval>` | Candle interval (if set) |
| `range` | `Option<TimeRange>` | Time range (if set) |

<!-- /soothfast:bind -->

Both tables are backed by a compiled field-verification test:

```rust capture-output
use finance_query::{Chart, IndexQuote};

// `IndexQuote` and `Chart` are #[non_exhaustive] with no public constructor
// outside the crate, but both derive `Deserialize` — build real instances via serde.
fn main() {
    let quote: IndexQuote = serde_json::from_value(serde_json::json!({
        "symbol": "^GSPC",
        "name": "S&P 500",
        "price": 5123.45,
        "change": 12.34,
        "change_percent": 0.24,
        "timestamp": 1_700_000_000_i64
    }))
    .unwrap();

    let chart: Chart = serde_json::from_value(serde_json::json!({
        "symbol": "^GSPC",
        "meta": { "symbol": "^GSPC" },
        "candles": [
            {"timestamp": 1_700_000_000_i64, "open": 5100.0, "high": 5150.0, "low": 5080.0, "close": 5123.45, "volume": 2_400_000_000_i64}
        ]
    }))
    .unwrap();

    println!("quote: {} = {:?}", quote.symbol, quote.price);
    println!("candles: {}", chart.candles.len());
    println!("close: {:.2}", chart.candles[0].close);
}
```

```text soothfast-output
quote: ^GSPC = Some(5123.45)
candles: 1
close: 5123.45
```

## `IndexConstituent` Fields

<!-- soothfast:bind finance_query::models::indices::IndexConstituent -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Ticker symbol of the constituent company |
| `name` | `Option<String>` | Company name |
| `sector` | `Option<String>` | Sector classification |
| `sub_sector` | `Option<String>` | Sub-sector classification |
| `headquarters` | `Option<String>` | Headquarters location |
| `date_first_added` | `Option<String>` | Date first added to the index (`YYYY-MM-DD`) |
| `cik` | `Option<String>` | SEC CIK number |
| `founded` | `Option<String>` | Year the company was founded |

<!-- /soothfast:bind -->

## `IndexConstituentChange` Fields

<!-- soothfast:bind finance_query::models::indices::IndexConstituentChange -->

| Field | Type | Description |
|-------|------|-------------|
| `date` | `Option<String>` | Date of the change (`YYYY-MM-DD`) |
| `symbol` | `Option<String>` | Ticker symbol the change concerns |
| `added_security` | `Option<String>` | Security that was added |
| `removed_ticker` | `Option<String>` | Ticker that was removed |
| `removed_security` | `Option<String>` | Security that was removed |
| `reason` | `Option<String>` | Reason for the change |

<!-- /soothfast:bind -->

See [`Interval`](configuration.md) and [`TimeRange`](configuration.md) for available values.

## See Also

- [Provider Configuration](providers/index.md) — Routing capabilities to providers
- [Wikipedia Provider](providers/wikipedia.md) — keyless S&P 500 constituents
- [FMP Provider](providers/fmp.md) — constituents for all three major indices, plus constituent-change history
- [Polygon.io Provider](providers/polygon.md) — alternate index quote source
