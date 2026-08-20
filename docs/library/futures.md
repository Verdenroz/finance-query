# Futures

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — domains::FuturesContract](https://docs.rs/finance-query/latest/finance_query/struct.FuturesContract.html)

The `FuturesContract` handle provides quote, chart, and history data for futures contracts. It is backed by Polygon.io and requires the `polygon` feature flag plus a `POLYGON_API_KEY` environment variable.

!!! info "Feature flag required"
    Add `features = ["polygon"]` to your `Cargo.toml` dependency and set the `POLYGON_API_KEY` environment variable before calling `build()`.

    ```toml
    [dependencies]
    finance-query = { version = "2", features = ["polygon"] }
    ```

## Getting a Handle

Obtain a `FuturesContract` by routing `Capability::FUTURES` to `Provider::Polygon` and calling `providers.futures(symbol)`:

```rust no_run feature=polygon
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FUTURES, [Provider::Polygon])
        .build()
        .await?;
    let contract = providers.futures("ES");
    let quote = contract.quote().await?;
    let chart = contract.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let history = contract.history(TimeRange::OneMonth).await?;
    println!("{}: {:?} ({} candles)", quote.symbol, quote.price, chart.candles.len() + history.candles.len());
    Ok(())
}
```

## Quote

`quote()` returns a [`FuturesQuote`](https://docs.rs/finance-query/latest/finance_query/struct.FuturesQuote.html) with the current contract price and metadata.

**`FuturesQuote` Fields:**

<!-- soothfast:bind finance_query::models::futures::FuturesQuote -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Contract ticker symbol (e.g., `"ESM26"`) |
| `name` | `Option<String>` | Human-readable contract name |
| `underlying` | `Option<String>` | Underlying asset (e.g., `"S&P 500"`) |
| `exchange` | `Option<String>` | Exchange where the contract trades |
| `expiration_date` | `Option<String>` | Contract expiry as `YYYY-MM-DD` |
| `price` | `Option<f64>` | Current contract price |
| `change` | `Option<f64>` | Price change |
| `change_percent` | `Option<f64>` | Price change as a percentage (e.g. `9.62` for 9.62%) |
| `open_interest` | `Option<u64>` | Number of outstanding contracts |
| `volume` | `Option<u64>` | Session volume in contracts |
| `timestamp` | `Option<i64>` | Unix timestamp of the last update, in **seconds** |

<!-- /soothfast:bind -->

This field-verification helper compiles as a real test, so the table above
cannot drift from the type:

```rust capture-output feature=polygon covers=finance_query::models::futures::FuturesQuote
use finance_query::FuturesQuote;

// `FuturesQuote` is #[non_exhaustive] outside the crate, so construct via
// serde. With live data: `providers.futures("ES").quote().await?`.
let quote: FuturesQuote = serde_json::from_value(serde_json::json!({
    "symbol": "ESM26",
    "name": "E-mini S&P 500 Jun 2026",
    "underlying": "S&P 500",
    "exchange": "CME",
    "expiration_date": "2026-06-19",
    "price": 5432.25,
    "change": 12.50,
    "change_percent": 0.23,
    "open_interest": 1_850_000_u64,
    "volume": 920_000_u64,
    "timestamp": 1_718_000_000_i64,
}))
.unwrap();

fn verify_futures_quote_fields(q: FuturesQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.name;
    let _: Option<String> = q.underlying;
    let _: Option<String> = q.exchange;
    let _: Option<String> = q.expiration_date;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<u64> = q.open_interest;
    let _: Option<u64> = q.volume;
    let _: Option<i64> = q.timestamp;
}
verify_futures_quote_fields(quote.clone());

println!("symbol = {}", quote.symbol);
println!("price = {:?}", quote.price);
println!("open_interest = {:?}", quote.open_interest);
```

```text soothfast-output
symbol = ESM26
price = Some(5432.25)
open_interest = Some(1850000)
```

## Chart

`chart(interval, range)` returns OHLCV candles for the requested interval and time range.

## History

`history(range)` is a convenience wrapper around `chart` that picks a sensible default interval for the given range via [`TimeRange::default_interval`](https://docs.rs/finance-query/latest/finance_query/enum.TimeRange.html#method.default_interval).

## Indicators & Risk

`indicators(interval, range)` / `indicator(kind, interval, range)` (requires
the `indicators` feature) and `risk(interval, range)` (requires the `risk`
feature) compute directly from this handle's own chart data:

```rust no_run feature=risk,polygon
use finance_query::indicators::Indicator;
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FUTURES, [Provider::Polygon])
        .build()
        .await?;
    let contract = providers.futures("ES");

    let summary = contract
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;
    if let Some(rsi) = summary.rsi_14 {
        println!("RSI(14): {:.2}", rsi);
    }

    let rsi_21 = contract
        .indicator(Indicator::Rsi(21), Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    let risk = contract.risk(Interval::OneDay, TimeRange::OneYear).await?;
    println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
    println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
    Ok(())
}
```

`risk` takes no benchmark parameter — `beta` is always `None`, since futures
contracts have no natural benchmark to compare against.

## Provider Reference

- [Polygon.io](providers/polygon.md) — the only provider that currently supports `Capability::FUTURES`.
