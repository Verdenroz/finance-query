# Commodities

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Commodity](https://docs.rs/finance-query/latest/finance_query/struct.Commodity.html)

The `Commodity` handle lets you fetch price quotes, OHLCV charts, and historical data for commodity symbols (gold, oil, natural gas, etc.). Commodity quotes are served by Yahoo Finance (keyless — no feature flag, no API key) on the default route, the same way Yahoo resolves equity quotes. FMP and Alpha Vantage remain available as alternate quote providers.

## Setup

`Providers::builder().build()` with no `.route()` call already serves commodity quotes — Yahoo is the default for every capability. Yahoo resolves commodity futures symbols the same way it resolves equities (e.g. `"GC=F"` for gold):

```rust no_run
use finance_query::{Interval, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let gold = providers.commodity("GC=F");
    let quote = gold.quote().await?;
    let chart = gold.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let history = gold.history(TimeRange::OneMonth).await?;
    println!("{}: {:?} ({} candles)", quote.symbol, quote.price, chart.candles.len() + history.candles.len());
    Ok(())
}
```

## Yahoo Commodity Symbols

Yahoo commodity futures use a root plus `=F` suffix:

| Symbol | Commodity |
|--------|-----------|
| `GC=F` | Gold |
| `SI=F` | Silver |
| `CL=F` | Crude Oil WTI |
| `NG=F` | Natural Gas |
| `HG=F` | Copper |
| `PL=F` | Platinum |

## Alternative: FMP or Alpha Vantage

Route `Capability::COMMODITIES` to `Provider::Fmp` or `Provider::AlphaVantage` for an alternate quote source, or as a fallback behind Yahoo:

```rust no_run feature=fmp
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::COMMODITIES, [Provider::Fmp, Provider::Yahoo])
        .build()
        .await?;
    let gold = providers.commodity("GCUSD");
    let quote = gold.quote().await?;
    println!("Symbol: {}", quote.symbol);
    if let Some(price) = quote.price {
        println!("Price: {:.2}", price);
    }
    Ok(())
}
```

Set `FMP_API_KEY` (or `ALPHAVANTAGE_API_KEY`) in your environment before calling `build()`. FMP's own commodity symbols don't use the `=F` suffix (e.g. `GCUSD` for gold, `CLUSD` for crude oil), and Alpha Vantage takes its function name directly as the symbol (e.g. `"WTI"`, `"NATURAL_GAS"`, `"COPPER"` — no precious-metals functions) — the symbol format is provider-specific, so switch it along with the route.

## Methods

### `quote()`

Fetches the current price quote for the commodity:

```rust no_run covers=finance_query::models::commodities::CommodityQuote
use finance_query::Providers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let gold = providers.commodity("GC=F");
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
    Ok(())
}
```

### `chart(interval, range)`

Fetches OHLCV candles at a specific interval over a given time range:

```rust no_run
use finance_query::{Interval, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let crude = providers.commodity("CL=F");
    let chart = crude.chart(Interval::OneDay, TimeRange::ThreeMonths).await?;

    println!("Symbol: {}", chart.symbol);
    println!("Candles: {}", chart.candles.len());
    for candle in chart.candles.iter().take(3) {
        println!("  t={} o={:.2} h={:.2} l={:.2} c={:.2}",
            candle.timestamp, candle.open, candle.high, candle.low, candle.close);
    }
    Ok(())
}
```

### `history(range)`

Fetches candles over a range using the default interval for that range
(determined by [`TimeRange::default_interval`]):

```rust no_run
use finance_query::{Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let silver = providers.commodity("SI=F");
    let history = silver.history(TimeRange::SixMonths).await?;

    println!("Symbol: {}", history.symbol);
    println!("Candles: {}", history.candles.len());
    Ok(())
}
```

### `indicators(interval, range)` / `indicator(kind, interval, range)` / `risk(interval, range)`

Computes technical indicators or a risk summary from the commodity's own
chart data (requires the `indicators`/`risk` features respectively):

```rust no_run feature=risk
use finance_query::indicators::Indicator;
use finance_query::{Interval, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder().build().await?;
    let gold = providers.commodity("GC=F");

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
    Ok(())
}
```

`risk` takes no benchmark parameter — `beta` is always `None`, since
commodities have no natural benchmark to compare against.

## `CommodityQuote` Fields

<!-- soothfast:bind finance_query::models::commodities::CommodityQuote -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Commodity symbol (e.g., `"GC=F"` for gold) |
| `name` | `Option<String>` | Human-readable name (e.g., `"Gold"`) |
| `unit` | `Option<String>` | Unit of measurement (e.g., `"troy ounce"`) |
| `price` | `Option<f64>` | Current price |
| `change` | `Option<f64>` | Price change |
| `change_percent` | `Option<f64>` | Price change percentage |
| `timestamp` | `Option<i64>` | Unix timestamp of last update |

<!-- /soothfast:bind -->

This field-verification helper compiles as a real test, so the table above
cannot drift from the type:

```rust capture-output covers=finance_query::models::commodities::CommodityQuote
use finance_query::CommodityQuote;

// `CommodityQuote` is #[non_exhaustive] outside the crate, so construct via
// serde. With live data: `commodity.quote().await?`.
let quote: CommodityQuote = serde_json::from_value(serde_json::json!({
    "symbol": "GC=F",
    "name": "Gold",
    "unit": "troy ounce",
    "price": 2387.50,
    "change": 12.30,
    "change_percent": 0.52,
    "timestamp": 1_718_000_000_i64,
}))
.unwrap();

fn verify_commodity_quote_fields(q: CommodityQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.name;
    let _: Option<String> = q.unit;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<i64> = q.timestamp;
}
verify_commodity_quote_fields(quote.clone());

println!("symbol = {}", quote.symbol);
println!("name = {:?}", quote.name);
println!("price = {:?}", quote.price);
```

```text soothfast-output
symbol = GC=F
name = Some("Gold")
price = Some(2387.5)
```

## Common Commodity Symbols

| Yahoo (`=F`) | FMP | Alpha Vantage | Commodity |
|--------------|-----|----------------|-----------|
| `GC=F` | `GCUSD` | — | Gold |
| `SI=F` | `SIUSD` | — | Silver |
| `CL=F` | `CLUSD` | `WTI` (or `BRENT`) | Crude Oil |
| `NG=F` | `NGUSD` | `NATURAL_GAS` | Natural Gas |
| `HG=F` | `HGUSD` | `COPPER` | Copper |
| `PL=F` | `PLUSD` | — | Platinum |

Alpha Vantage has no precious-metals (gold/silver/platinum) commodity functions; route those to Yahoo or FMP instead.

## See Also

- [Provider Configuration](providers/index.md) — Routing capabilities to providers
- [FMP Provider](providers/fmp.md) — FMP setup and capabilities
- [Alpha Vantage Provider](providers/alphavantage.md) — Alpha Vantage setup and capabilities
- [Chart & History](ticker.md) — `Chart` and `Candle` type reference
