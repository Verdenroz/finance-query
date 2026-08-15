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

```rust no_run feature=alphavantage
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;

    let pair = providers.forex("EUR", "USD");
    Ok(())
}
```

## Quote

Fetch the current exchange rate for the pair:

```rust no_run feature=alphavantage
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;
    let pair = providers.forex("EUR", "USD");

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
    Ok(())
}
```

**`ForexQuote` fields:**

<!-- soothfast:bind finance_query::models::forex::ForexQuote -->

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

<!-- /soothfast:bind -->

This field-verification helper compiles as a real test, so the table above
cannot drift from the type:

```rust capture-output feature=alphavantage covers=finance_query::models::forex::ForexQuote
use finance_query::ForexQuote;

// `ForexQuote` is #[non_exhaustive] outside the crate, so construct via
// serde. With live data: `providers.forex("EUR", "USD").quote().await?`.
let quote: ForexQuote = serde_json::from_value(serde_json::json!({
    "symbol": "EURUSD",
    "base_currency": "EUR",
    "quote_currency": "USD",
    "bid": 1.084250,
    "ask": 1.084400,
    "price": 1.084325,
    "change": 0.001200,
    "change_percent": 0.1108,
    "timestamp": 1_718_000_000_i64,
}))
.unwrap();

fn verify_forex_quote_fields(q: ForexQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.base_currency;
    let _: Option<String> = q.quote_currency;
    let _: Option<f64> = q.bid;
    let _: Option<f64> = q.ask;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<i64> = q.timestamp;
}
verify_forex_quote_fields(quote.clone());

println!("symbol = {}", quote.symbol);
println!("bid = {:?}, ask = {:?}", quote.bid, quote.ask);
println!("price = {:?}", quote.price);
```

```text soothfast-output
symbol = EURUSD
bid = Some(1.08425), ask = Some(1.0844)
price = Some(1.084325)
```

## Chart

Fetch historical OHLCV candles at a given interval and range:

```rust no_run feature=alphavantage
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;
    let pair = providers.forex("EUR", "USD");

    let chart = pair.chart(Interval::OneDay, TimeRange::OneMonth).await?;

    println!("Pair: {}", chart.symbol);
    assert!(!chart.candles.is_empty());

    for candle in &chart.candles {
        println!(
            "{}: O={:.6}, H={:.6}, L={:.6}, C={:.6}",
            candle.timestamp, candle.open, candle.high, candle.low, candle.close
        );
    }
    Ok(())
}
```

The symbol used internally follows the Yahoo FX convention `"{FROM}{TO}=X"`
(e.g., `"EURUSD=X"`), but `chart.symbol` reflects this mapped form.

## History

Fetch historical candles over a range using the sensible default interval for
that range:

```rust no_run feature=alphavantage
use finance_query::{Capability, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;
    let pair = providers.forex("EUR", "USD");

    let history = pair.history(TimeRange::OneMonth).await?;

    assert!(!history.candles.is_empty());

    if let Some(last) = history.candles.last() {
        println!("Most recent close: {:.6}", last.close);
    }
    Ok(())
}
```

`history(range)` is equivalent to `chart(range.default_interval(), range)`.

## Indicators & Risk

!!! info "Feature flags required"
    ```toml
    finance-query = { version = "...", features = ["indicators", "risk"] }
    ```

Compute technical indicators or a risk summary directly from the pair's chart data:

```rust no_run feature=risk,alphavantage
use finance_query::indicators::Indicator;
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;
    let pair = providers.forex("EUR", "USD");

    let summary = pair
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await?;
    if let Some(rsi) = summary.rsi_14 {
        println!("RSI(14): {:.2}", rsi);
    }

    let rsi_21 = pair
        .indicator(Indicator::Rsi(21), Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    let risk = pair.risk(Interval::OneDay, TimeRange::OneYear).await?;
    println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
    println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
    Ok(())
}
```

`indicators`/`indicator` mirror [`Ticker`](ticker.md)'s API but compute from this
handle's own chart data. `risk` takes no benchmark parameter — `beta` is always
`None`, since non-equity handles have no natural benchmark to compare against.

## Caching

Caching is **on by default** and lasts as long as the handle lives. Use
`.cache(Duration)` to bound how long a response is reused, or `.no_cache()` to
fetch fresh on every call.

```rust no_run feature=alphavantage
use finance_query::{Capability, Provider, Providers};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .build()
        .await?;

    // Default: the first call hits the network, every later call is served
    // from cache for as long as this handle is alive.
    let pair = providers.forex("EUR", "USD");
    let _q1 = pair.quote().await?;
    let _q2 = pair.quote().await?; // served from cache

    // Bound reuse to a 60-second TTL instead.
    let pair = providers
        .forex("EUR", "USD")
        .cache(Duration::from_secs(60));

    // Or opt out entirely — every call fetches fresh.
    let pair = providers.forex("EUR", "USD").no_cache();
    let _fresh = pair.quote().await?;
    Ok(())
}
```

## See Also

- [Alpha Vantage](providers/alphavantage.md) — provider setup and capabilities
- [Ticker API](ticker.md) — single-symbol equity data
