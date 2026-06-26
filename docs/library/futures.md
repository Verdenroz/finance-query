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

```rust
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder()
    .route(Capability::FUTURES, &[Provider::Polygon])
    .build()
    .await?;
let contract = providers.futures("ES");
let quote = contract.quote().await?;
let chart = contract.chart(Interval::OneDay, TimeRange::OneMonth).await?;
let history = contract.history(TimeRange::OneMonth).await?;
# Ok(()) }
```

## Quote

`quote()` returns a [`FuturesQuote`](https://docs.rs/finance-query/latest/finance_query/struct.FuturesQuote.html) with the current contract price and metadata.

**`FuturesQuote` Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Contract ticker symbol (e.g., `"ESM26"`) |
| `name` | `Option<String>` | Human-readable contract name |
| `underlying` | `Option<String>` | Underlying asset (e.g., `"S&P 500"`) |
| `exchange` | `Option<String>` | Exchange where the contract trades |
| `expiration_date` | `Option<String>` | Contract expiry as `YYYY-MM-DD` |
| `price` | `Option<f64>` | Current contract price |
| `change` | `Option<f64>` | Price change |
| `change_percent` | `Option<f64>` | Price change percentage |
| `open_interest` | `Option<u64>` | Number of outstanding contracts |
| `volume` | `Option<u64>` | Trading volume |
| `timestamp` | `Option<i64>` | Unix timestamp of the last update |

## Chart

`chart(interval, range)` returns OHLCV candles for the requested interval and time range.

## History

`history(range)` is a convenience wrapper around `chart` that picks a sensible default interval for the given range via [`TimeRange::default_interval`](https://docs.rs/finance-query/latest/finance_query/enum.TimeRange.html#method.default_interval).

## Provider Reference

- [Polygon.io](providers/polygon.md) — the only provider that currently supports `Capability::FUTURES`.
