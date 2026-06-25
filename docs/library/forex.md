# Forex

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — ForexPair](https://docs.rs/finance-query/latest/finance_query/struct.ForexPair.html)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["alphavantage"] }
    ```

The `ForexPair` struct provides real-time quotes and historical OHLCV data for
foreign-exchange currency pairs. It requires a keyed provider — Alpha Vantage is
the canonical choice. See [Alpha Vantage](providers/alphavantage.md) for setup.

!!! note "API key required"
    Set your Alpha Vantage key in the environment before calling any method:

    ```bash
    export ALPHAVANTAGE_API_KEY="your-alphavantage-key"
    ```

## Getting a Handle

Route `Capability::FOREX` to `Provider::AlphaVantage` and call
`providers.forex(from, to)`:

```rust
use finance_query::{Capability, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::FOREX, &[Provider::AlphaVantage])
    .build()
    .await?;

let pair = providers.forex("EUR", "USD");
```

## Quote

Fetch the current exchange rate for the pair:

```rust
let quote = pair.quote().await?;

println!("Symbol: {}", quote.symbol);
if let Some(price) = quote.price {
    println!("Rate: {:.6}", price);
}
if let Some(bid) = quote.bid {
    println!("Bid:  {:.6}", bid);
}
if let Some(ask) = quote.ask {
    println!("Ask:  {:.6}", ask);
}
if let (Some(chg), Some(pct)) = (quote.change, quote.change_percent) {
    println!("Change: {:+.6} ({:+.4}%)", chg, pct);
}
```

**`ForexQuote` fields:**

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Currency pair symbol (e.g., `"EURUSD"`) |
| `base_currency` | `Option<String>` | Base currency code (e.g., `"EUR"`) |
| `quote_currency` | `Option<String>` | Quote currency code (e.g., `"USD"`) |
| `bid` | `Option<f64>` | Bid price |
| `ask` | `Option<f64>` | Ask price |
| `price` | `Option<f64>` | Midpoint or last traded price |
| `change` | `Option<f64>` | Price change |
| `change_percent` | `Option<f64>` | Price change percentage |
| `timestamp` | `Option<i64>` | Unix timestamp of the last update |

## Chart

Fetch historical OHLCV candles at a given interval and range:

```rust
use finance_query::{Interval, TimeRange};

let chart = pair.chart(Interval::OneDay, TimeRange::OneMonth).await?;

println!("Pair: {}", chart.symbol);
assert!(!chart.candles.is_empty());

for candle in &chart.candles {
    println!(
        "{}: O={:.6}, H={:.6}, L={:.6}, C={:.6}",
        candle.timestamp, candle.open, candle.high, candle.low, candle.close
    );
}
```

The symbol used internally follows the Yahoo FX convention `"{FROM}{TO}=X"`
(e.g., `"EURUSD=X"`), but `chart.symbol` reflects this mapped form.

## History

Fetch historical candles over a range using the sensible default interval for
that range:

```rust
use finance_query::TimeRange;

let history = pair.history(TimeRange::OneMonth).await?;

assert!(!history.candles.is_empty());

if let Some(last) = history.candles.last() {
    println!("Most recent close: {:.6}", last.close);
}
```

`history(range)` is equivalent to `chart(range.default_interval(), range)`.

## Caching

Enable in-memory caching with a TTL to avoid redundant network requests:

```rust
use finance_query::{Capability, Provider, Providers};
use std::time::Duration;

let providers = Providers::builder()
    .route(Capability::FOREX, &[Provider::AlphaVantage])
    .build()
    .await?;

let pair = providers
    .forex("EUR", "USD")
    .cache(Duration::from_secs(60));

// First call hits the network; subsequent calls within 60 s are cached.
let _q1 = pair.quote().await?;
let _q2 = pair.quote().await?; // served from cache
```

## See Also

- [Alpha Vantage](providers/alphavantage.md) — provider setup and capabilities
- [Ticker API](ticker.md) — single-symbol equity data
