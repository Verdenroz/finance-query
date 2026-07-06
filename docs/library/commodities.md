# Commodities

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Commodity](https://docs.rs/finance-query/latest/finance_query/struct.Commodity.html)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["fmp"] }
    ```

The `Commodity` handle lets you fetch price quotes, OHLCV charts, and historical
data for commodity symbols (gold, oil, natural gas, etc.) using FMP as the
backend provider.

## Setup

Set the FMP API key via environment variable:

```bash
export FMP_API_KEY="your-fmp-api-key"
```

Route the `COMMODITIES` capability to `Provider::Fmp` when building `Providers`:

```rust
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .route(Capability::COMMODITIES, [Provider::Fmp])
    .build()
    .await?;
let gold = providers.commodity("GCUSD");
let quote = gold.quote().await?;
let chart = gold.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let history = gold.history(TimeRange::OneMonth).await?;
# Ok(()) }
```

## Methods

### `quote()`

Fetches the current price quote for the commodity:

```rust
use finance_query::{Capability, Provider, Providers};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .route(Capability::COMMODITIES, [Provider::Fmp])
    .build()
    .await?;
let gold = providers.commodity("GCUSD");
let quote = gold.quote().await?;

println!("Symbol: {}", quote.symbol);
if let Some(name) = &quote.name {
    println!("Name: {}", name);
}
if let Some(price) = quote.price {
    println!("Price: {:.2}", price);
}
if let (Some(change), Some(pct)) = (quote.change, quote.change_percent) {
    println!("Change: {:+.2} ({:+.2}%)", change, pct);
}
# Ok(()) }
```

### `chart(interval, range)`

Fetches OHLCV candles at a specific interval over a given time range:

```rust
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .route(Capability::COMMODITIES, [Provider::Fmp])
    .build()
    .await?;
let crude = providers.commodity("CLUSD");
let chart = crude.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;

println!("Symbol: {}", chart.symbol);
println!("Candles: {}", chart.candles.len());
for candle in chart.candles.iter().take(3) {
    println!("  t={} o={:.2} h={:.2} l={:.2} c={:.2}",
        candle.timestamp, candle.open, candle.high, candle.low, candle.close);
}
# Ok(()) }
```

### `history(range)`

Fetches candles over a range using the default interval for that range
(determined by [`TimeRange::default_interval`]):

```rust
use finance_query::{Capability, Provider, Providers, TimeRange};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .route(Capability::COMMODITIES, [Provider::Fmp])
    .build()
    .await?;
let silver = providers.commodity("SIUSD");
let history = silver.history(TimeRange::SixMonths).await?;

println!("Symbol: {}", history.symbol);
println!("Candles: {}", history.candles.len());
# Ok(()) }
```

### `indicators(interval, range)` / `indicator(kind, interval, range)` / `risk(interval, range)`

Computes technical indicators or a risk summary from the commodity's own
chart data (requires the `indicators`/`risk` features respectively):

```rust
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};
use finance_query::indicators::Indicator;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .route(Capability::COMMODITIES, [Provider::Fmp])
    .build()
    .await?;
let gold = providers.commodity("GCUSD");

let summary = gold.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;
if let Some(rsi) = summary.rsi_14 {
    println!("RSI(14): {:.2}", rsi);
}

let rsi_21 = gold
    .indicator(Indicator::Rsi(21), Interval::OneDay, TimeRange::ThreeMonths)
    .await?;

let risk = gold.risk(Interval::OneDay, TimeRange::OneYear).await?;
println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
# Ok(()) }
```

`risk` takes no benchmark parameter — `beta` is always `None`, since
commodities have no natural benchmark to compare against.

## `CommodityQuote` Fields

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Commodity symbol (e.g., `"GCUSD"` for gold) |
| `name` | `Option<String>` | Human-readable name (e.g., `"Gold"`) |
| `unit` | `Option<String>` | Unit of measurement (e.g., `"troy ounce"`) |
| `price` | `Option<f64>` | Current price |
| `change` | `Option<f64>` | Price change |
| `change_percent` | `Option<f64>` | Price change percentage |
| `timestamp` | `Option<i64>` | Unix timestamp of last update |

## Common FMP Commodity Symbols

| Symbol | Commodity |
|--------|-----------|
| `GCUSD` | Gold |
| `SIUSD` | Silver |
| `CLUSD` | Crude Oil WTI |
| `NGUSD` | Natural Gas |
| `HGUSD` | Copper |
| `PLUSD` | Platinum |

## See Also

- [Provider Configuration](providers/index.md) — Routing capabilities to providers
- [FMP Provider](providers/fmp.md) — FMP setup and capabilities
- [Chart & History](ticker.md) — `Chart` and `Candle` type reference
