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

```rust
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .route(Capability::INDICES, &[Provider::Polygon])
    .build()
    .await?;
let spx = providers.index("I:SPX");
let quote = spx.quote().await?;
let chart = spx.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let history = spx.history(TimeRange::OneMonth).await?;
# Ok(()) }
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

```rust
# use finance_query::{Capability, Provider, Providers};
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let providers = Providers::builder().route(Capability::INDICES, &[Provider::Polygon]).build().await?;
let spx = providers.index("I:SPX");
let quote = spx.quote().await?;
println!("S&P 500: {:?}", quote.price);
# Ok(()) }
```

### `chart(interval, range)`

Fetch OHLCV candles for a specific `Interval` and `TimeRange`.

```rust
# use finance_query::{Capability, Interval, Provider, Providers, TimeRange};
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let providers = Providers::builder().route(Capability::INDICES, &[Provider::Polygon]).build().await?;
let spx = providers.index("I:SPX");
let chart = spx.chart(Interval::OneDay, TimeRange::OneMonth).await?;
println!("Candles: {}", chart.candles.len());
# Ok(()) }
```

### `history(range)`

Shorthand for `chart` using the default interval for the given `TimeRange`.

```rust
# use finance_query::{Capability, Provider, Providers, TimeRange};
# async fn run() -> Result<(), Box<dyn std::error::Error>> {
# let providers = Providers::builder().route(Capability::INDICES, &[Provider::Polygon]).build().await?;
let spx = providers.index("I:SPX");
let history = spx.history(TimeRange::OneMonth).await?;
println!("Candles: {}", history.candles.len());
# Ok(()) }
```

## `IndexQuote` Fields

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Index ticker symbol (e.g., `"I:SPX"`) |
| `name` | `Option<String>` | Human-readable index name (e.g., `"S&P 500"`) |
| `price` | `Option<f64>` | Current index value |
| `change` | `Option<f64>` | Absolute price change |
| `change_percent` | `Option<f64>` | Percentage price change |
| `timestamp` | `Option<i64>` | Unix timestamp of last update |

## `Chart` Fields

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Index symbol |
| `candles` | `Vec<Candle>` | OHLCV candle data |
| `interval` | `Option<Interval>` | Candle interval (if set) |
| `range` | `Option<TimeRange>` | Time range (if set) |

See [`Interval`](configuration.md) and [`TimeRange`](configuration.md) for available values.
