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

```rust
use finance_query::crypto;

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
```

## Single Coin

Fetch a single coin's quote by its CoinGecko ID:

```rust
use finance_query::crypto;

let btc = crypto::coin("bitcoin", "usd").await?;
assert_eq!(btc.id, "bitcoin");
assert_eq!(btc.symbol.to_uppercase(), "BTC");
let price = btc.current_price.unwrap_or(0.0);
assert!(price > 0.0, "BTC price should be positive");
println!("Bitcoin: ${:.2}", price);
```

## CoinQuote Fields

`CoinQuote` is returned by both `crypto::coins` and `crypto::coin`.

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

```rust
use finance_query::crypto::CoinQuote;

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
```

## Coin Handle

The `CryptoCoin` handle provides a domain-oriented interface for quote, chart, and history
queries backed by your configured providers. Construct it via `Providers::crypto`:

```rust
use finance_query::Providers;
use finance_query::{Interval, TimeRange};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder().build().await?;
let btc = providers.crypto("bitcoin");
let quote = btc.quote("usd").await?;
let chart = btc.chart("usd", Interval::OneDay, TimeRange::OneMonth).await?;
let history = btc.history("usd", TimeRange::OneMonth).await?;
# Ok(()) }
```

`quote` returns a [`CryptoQuote`](https://docs.rs/finance-query/latest/finance_query/struct.CryptoQuote.html)
where the price field is `price` (not `current_price`):

```rust
use finance_query::CryptoQuote;

fn verify_crypto_quote_price(q: CryptoQuote) {
    let _: Option<f64> = q.price;
}
```
