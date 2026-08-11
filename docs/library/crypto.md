# Crypto

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — crypto](https://docs.rs/finance-query/latest/finance_query/crypto/index.html)

!!! note "Feature flag required"
    The CoinGecko functions (`crypto::coins`, `crypto::coin`) require the `crypto` feature:
    ```toml
    [dependencies]
    finance-query = { version = "*", features = ["crypto"] }
    ```
    The `CryptoCoin` handle (`providers.crypto(id)`) is also gated on the `crypto` feature when
    using the CoinGecko provider, but the handle type itself is available with other provider
    features (`alphavantage`, `fmp`, `polygon`).

## Top Coins

Fetch the top N coins by market cap, priced in a given vs-currency:

```rust no_run feature=crypto covers=finance_query::de_crypto_coins
use finance_query::crypto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let top = crypto::coins("usd", 10).await?;
    assert!(!top.is_empty(), "should return coins");
    assert!(top.len() <= 10);

    for coin in &top {
        let price = coin.current_price.unwrap_or(0.0);
        let change = coin.price_change_percentage_24h.unwrap_or(0.0);
        let rank = coin.market_cap_rank.unwrap_or(0);
        println!(
            "#{} {} ({}): ${:.2} ({:+.2}%)",
            rank, coin.name, coin.symbol, price, change
        );
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::de_crypto_coins.walltime.median_ns < 100000 -->
Cheap enough to re-fetch and re-parse on every poll.

## Single Coin

Fetch a single coin's quote by its CoinGecko ID:

```rust no_run feature=crypto
use finance_query::crypto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let btc = crypto::coin("bitcoin", "usd").await?;
    assert_eq!(btc.id, "bitcoin");
    assert_eq!(btc.symbol.to_uppercase(), "BTC");
    let price = btc.current_price.unwrap_or(0.0);
    assert!(price > 0.0, "BTC price should be positive");
    println!("Bitcoin: ${:.2}", price);
    Ok(())
}
```

## CoinQuote Fields

`CoinQuote` is returned by both `crypto::coins` and `crypto::coin`.

<!-- soothfast:bind finance_query::models::crypto::CoinQuote -->

| Field | Type | Description |
|---|---|---|
| `id` | `String` | CoinGecko coin ID (e.g., `"bitcoin"`) |
| `symbol` | `String` | Ticker symbol in uppercase (e.g., `"BTC"`) |
| `name` | `String` | Full coin name (e.g., `"Bitcoin"`) |
| `current_price` | `Option<f64>` | Current price in the requested currency |
| `market_cap` | `Option<f64>` | Market capitalisation |
| `market_cap_rank` | `Option<u32>` | Market cap rank (1 = highest) |
| `price_change_percentage_24h` | `Option<f64>` | 24-hour price change percentage |
| `total_volume` | `Option<f64>` | 24-hour trading volume |
| `circulating_supply` | `Option<f64>` | Circulating supply |
| `image` | `Option<String>` | URL to the coin's logo image |

<!-- /soothfast:bind -->

This field-verification helper compiles as a real test, so the table above
cannot drift from the type:

```rust capture-output feature=crypto covers=finance_query::models::crypto::CoinQuote
use finance_query::crypto::CoinQuote;

// `CoinQuote` is #[non_exhaustive] outside the crate, so construct via
// serde. With live data: `crypto::coin("bitcoin", "usd").await?`.
let coin: CoinQuote = serde_json::from_value(serde_json::json!({
    "id": "bitcoin",
    "symbol": "BTC",
    "name": "Bitcoin",
    "current_price": 67234.50,
    "market_cap": 1_325_000_000_000.0,
    "market_cap_rank": 1,
    "price_change_percentage_24h": 2.35,
    "total_volume": 28_400_000_000.0,
    "circulating_supply": 19_700_000.0,
    "image": "https://assets.coingecko.com/coins/images/1/large/bitcoin.png",
}))
.unwrap();

fn verify_coin_quote_fields(c: CoinQuote) {
    let _: String = c.id;
    let _: String = c.symbol;
    let _: String = c.name;
    let _: Option<f64> = c.current_price;
    let _: Option<f64> = c.market_cap;
    let _: Option<u32> = c.market_cap_rank;
    let _: Option<f64> = c.price_change_percentage_24h;
    let _: Option<f64> = c.total_volume;
    let _: Option<f64> = c.circulating_supply;
    let _: Option<String> = c.image;
}
verify_coin_quote_fields(coin.clone());

println!("id = {}, symbol = {}", coin.id, coin.symbol);
println!("current_price = {:?}", coin.current_price);
println!("market_cap_rank = {:?}", coin.market_cap_rank);
```

```text soothfast-output
id = bitcoin, symbol = BTC
current_price = Some(67234.5)
market_cap_rank = Some(1)
```

## Coin Handle

The `CryptoCoin` handle provides a domain-oriented interface for quote, chart, and history
queries backed by your configured providers. Construct it via `Providers::crypto`.

`quote` is keyed by the **CoinGecko coin id** (e.g. `"bitcoin"`) and is keyless via CoinGecko:

```rust no_run feature=crypto
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::CoinGecko])
        .build()
        .await?;
    let btc = providers.crypto("bitcoin");
    let quote = btc.quote("usd").await?;
    println!("BTC: {:?}", quote.price);
    Ok(())
}
```

!!! note "Chart vs. quote identifiers"
    `chart`/`history` route through `Capability::CHART`, not `CRYPTO`. On the default
    Yahoo route the handle id must be the coin's **ticker** (e.g. `"BTC"`, which Yahoo
    resolves as `"BTC-USD"`) — *not* the CoinGecko id. To use the CoinGecko id for charts
    too, route `Capability::CHART` to a crypto-aware provider (Polygon, FMP, or Alpha Vantage).

```rust no_run feature=crypto
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::CoinGecko])
        .build()
        .await?;
    // Ticker id ("BTC") so the default Yahoo CHART route resolves "BTC-USD".
    let btc = providers.crypto("BTC");
    let chart = btc.chart("usd", Interval::OneDay, TimeRange::OneMonth).await?;
    let history = btc.history("usd", TimeRange::OneMonth).await?;
    println!("{} candles", chart.candles.len() + history.candles.len());
    Ok(())
}
```

<!-- soothfast:bind finance_query::models::crypto::CryptoQuote -->
`quote` returns a [`CryptoQuote`](https://docs.rs/finance-query/latest/finance_query/struct.CryptoQuote.html)
where the price field is `price` (not `current_price`):
<!-- /soothfast:bind -->

```rust capture-output feature=crypto covers=finance_query::models::crypto::CryptoQuote
use finance_query::CryptoQuote;

// `CryptoQuote` is #[non_exhaustive] outside the crate, so construct via
// serde. With live data: `providers.crypto("BTC").quote("usd").await?`.
let quote: CryptoQuote = serde_json::from_value(serde_json::json!({
    "id": "BTC",
    "symbol": "BTC",
    "name": "Bitcoin",
    "price": 67234.50,
    "market_cap": 1_325_000_000_000.0,
    "volume_24h": 28_400_000_000.0,
    "change_24h": 1540.20,
    "change_percent_24h": 2.35,
    "high_24h": 67800.0,
    "low_24h": 65500.0,
    "circulating_supply": 19_700_000.0,
}))
.unwrap();

fn verify_crypto_quote_price(q: CryptoQuote) {
    let _: Option<f64> = q.price;
}
verify_crypto_quote_price(quote.clone());

println!("symbol = {}", quote.symbol);
println!("price = {:?}", quote.price);
```

```text soothfast-output
symbol = BTC
price = Some(67234.5)
```

## Indicators & Risk

`indicators`/`indicator` (requires the `indicators` feature) and `risk`
(requires the `risk` feature) compute from the same `vs_currency`-priced
chart data as `chart`/`history` above — annualised with a 24/7 (365-day)
calendar, since crypto trades every day of the year:

```rust no_run feature=risk
use finance_query::indicators::Indicator;
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::CoinGecko])
        .build()
        .await?;
    let btc = providers.crypto("BTC");

    let summary = btc
        .indicators("usd", Interval::OneDay, TimeRange::ThreeMonths)
        .await?;
    if let Some(rsi) = summary.rsi_14 {
        println!("RSI(14): {:.2}", rsi);
    }

    let rsi_21 = btc
        .indicator(Indicator::Rsi(21), "usd", Interval::OneDay, TimeRange::ThreeMonths)
        .await?;

    let risk = btc.risk("usd", Interval::OneDay, TimeRange::OneYear).await?;
    println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
    println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
    Ok(())
}
```

`risk` takes no benchmark parameter — `beta` is always `None`, since crypto
has no natural benchmark to compare against.
