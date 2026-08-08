# Indices

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Index](https://docs.rs/finance-query/latest/finance_query/struct.Index.html)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["polygon"] }
    ```

The `Index` handle provides access to stock market index data (quotes, charts, and history) via Polygon.io. Indices are keyed by provider-specific symbol strings — for Polygon, this is the `I:<NAME>` format (e.g., `"I:SPX"` for the S&P 500).

## Setup

Route the `INDICES` capability to Polygon in your `Providers` builder, then create an `Index` handle with `providers.index(symbol)`:

```rust no_run feature=polygon
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Polygon])
        .build()
        .await?;
    let spx = providers.index("I:SPX");
    let quote = spx.quote().await?;
    let chart = spx.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let history = spx.history(TimeRange::OneMonth).await?;
    println!("{}: {:?} ({} candles)", quote.symbol, quote.price, chart.candles.len() + history.candles.len());
    Ok(())
}
```

Set `POLYGON_API_KEY` in your environment before calling `build()`:

```bash
export POLYGON_API_KEY="your-polygon-key"
```

## Index Symbols (Polygon)

Polygon indices use an `I:<TICKER>` prefix:

| Symbol | Index |
|--------|-------|
| `I:SPX` | S&P 500 |
| `I:NDX` | NASDAQ-100 |
| `I:DJI` | Dow Jones Industrial Average |
| `I:RUT` | Russell 2000 |
| `I:VIX` | CBOE Volatility Index |

## Methods

### `quote()`

Fetch the current snapshot for the index.

```rust no_run feature=polygon covers=finance_query::models::indices::IndexQuote
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Polygon])
        .build()
        .await?;
    let spx = providers.index("I:SPX");
    let quote = spx.quote().await?;
    println!("S&P 500: {:?}", quote.price);
    Ok(())
}
```

### `chart(interval, range)`

Fetch OHLCV candles for a specific `Interval` and `TimeRange`.

```rust no_run feature=polygon covers=finance_query::models::chart::data::Chart
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Polygon])
        .build()
        .await?;
    let spx = providers.index("I:SPX");
    let chart = spx.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    println!("Candles: {}", chart.candles.len());
    Ok(())
}
```

### `history(range)`

Shorthand for `chart` using the default interval for the given `TimeRange`.

```rust no_run feature=polygon
use finance_query::{Capability, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Polygon])
        .build()
        .await?;
    let spx = providers.index("I:SPX");
    let history = spx.history(TimeRange::OneMonth).await?;
    println!("Candles: {}", history.candles.len());
    Ok(())
}
```

### `indicators(interval, range)` / `indicator(kind, interval, range)` / `risk(interval, range)`

Compute technical indicators or a risk summary from this index's own chart
data (requires the `indicators`/`risk` features respectively).

```rust no_run feature=risk
use finance_query::indicators::Indicator;
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::INDICES, [Provider::Polygon])
        .build()
        .await?;
    let spx = providers.index("I:SPX");
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
| `symbol` | `String` | Index ticker symbol (e.g., `"I:SPX"`) |
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

```rust capture-output feature=polygon
use finance_query::{Chart, IndexQuote};

// `IndexQuote` and `Chart` are #[non_exhaustive] with no public constructor
// outside the crate, but both derive `Deserialize` — build real instances via serde.
fn main() {
    let quote: IndexQuote = serde_json::from_value(serde_json::json!({
        "symbol": "I:SPX",
        "name": "S&P 500",
        "price": 5123.45,
        "change": 12.34,
        "change_percent": 0.24,
        "timestamp": 1_700_000_000_i64
    }))
    .unwrap();

    let chart: Chart = serde_json::from_value(serde_json::json!({
        "symbol": "I:SPX",
        "meta": { "symbol": "I:SPX" },
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
quote: I:SPX = Some(5123.45)
candles: 1
close: 5123.45
```

See [`Interval`](configuration.md) and [`TimeRange`](configuration.md) for available values.
