# Crypto (CoinGecko)

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — crypto](https://docs.rs/finance-query/latest/finance_query/crypto/index.html)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["crypto"] }
    ```

The `crypto` module provides cryptocurrency market data via the CoinGecko public API. No API key is required. Rate limiting (30 req/min on the free tier) is handled automatically.

```rust feature=crypto
use finance_query::crypto;
```

## Top Coins by Market Cap

```rust no_run feature=crypto covers=finance_query::de_crypto_coins
use finance_query::crypto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Top 10 coins in USD
    let top = crypto::coins("usd", 10).await?;

    for coin in &top {
        let price   = coin.current_price.unwrap_or(0.0);
        let change  = coin.price_change_percentage_24h.unwrap_or(0.0);
        let rank    = coin.market_cap_rank.unwrap_or(0);
        println!("#{} {} ({}): ${:.2} ({:+.2}%)", rank, coin.name, coin.symbol, price, change);
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::de_crypto_coins.walltime.median_ns < 200000 -->
<!-- soothfast:claim finance_query::de_crypto_coins.perfcnt.instructions < 700000 -->
The network round-trip, not parsing, dominates every call.

- `vs_currency` — Quote currency: `"usd"`, `"eur"`, `"btc"`, `"eth"`, etc.
- `count` — Number of coins to return (max 250).

## Single Coin Lookup

```rust no_run feature=crypto covers=finance_query::models::crypto::CoinQuote
use finance_query::crypto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Look up by CoinGecko ID
    let btc = crypto::coin("bitcoin", "usd").await?;
    println!("Bitcoin: ${:.2}", btc.current_price.unwrap_or(0.0));

    let eth = crypto::coin("ethereum", "usd").await?;
    let mktcap = eth.market_cap.unwrap_or(0.0);
    println!("Ethereum market cap: ${:.2}B", mktcap / 1e9);
    Ok(())
}
```

CoinGecko IDs are lowercase, hyphenated names. Common examples:

| Name | CoinGecko ID |
|------|-------------|
| Bitcoin | `"bitcoin"` |
| Ethereum | `"ethereum"` |
| BNB | `"binancecoin"` |
| Solana | `"solana"` |
| XRP | `"ripple"` |
| USDC | `"usd-coin"` |
| Dogecoin | `"dogecoin"` |

To discover IDs programmatically, call the CoinGecko `/coins/list` endpoint.

## `CoinQuote` Fields

<!-- soothfast:bind finance_query::models::crypto::CoinQuote -->

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | CoinGecko ID (e.g., `"bitcoin"`) |
| `symbol` | `String` | Ticker symbol in uppercase (e.g., `"BTC"`) |
| `name` | `String` | Full coin name (e.g., `"Bitcoin"`) |
| `current_price` | `Option<f64>` | Current price in the requested currency |
| `market_cap` | `Option<f64>` | Market capitalisation |
| `market_cap_rank` | `Option<u32>` | Market cap rank (1 = largest) |
| `price_change_percentage_24h` | `Option<f64>` | 24-hour price change (%) |
| `total_volume` | `Option<f64>` | 24-hour trading volume |
| `circulating_supply` | `Option<f64>` | Circulating supply |
| `image` | `Option<String>` | URL to the coin's logo image |

<!-- /soothfast:bind -->

## Price History (keyless `CHART` route)

Route `Capability::CHART` to CoinGecko and `CryptoCoin` serves OHLC history by
coin id, without any API key:

```rust,ignore
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

let providers = Providers::builder()
    .route(Capability::CHART, [Provider::CoinGecko])
    .build()
    .await?;

let chart = providers
    .crypto("bitcoin")
    .history("usd", TimeRange::ThreeMonths)
    .await?;
println!("{} candles", chart.candles.len());
```

Two properties of CoinGecko's public `/ohlc` endpoint carry through:

- **`interval` is advisory.** CoinGecko selects bar granularity from the day
  span alone — 30 minutes for 1–2 days, 4 hours for 3–30 days, 4 days beyond
  that. Ranges are mapped onto its accepted spans (`1`, `7`, `30`, `90`, `180`,
  `365`, `max`); anything past a year becomes `max`.
- **Candles have no volume.** `/ohlc` does not report it, so `volume` is `0`
  rather than a figure interpolated from a differently-bucketed series. Volume
  ratio indicators (OBV, VWMA) are meaningless on these candles.

The handle's id and quote currency are recombined as `"{id}-{vs}"` and split on
the last hyphen, so hyphenated CoinGecko ids (`usd-coin`, `staked-ether`) work.

## Rate Limits

The CoinGecko free tier allows **30 requests per minute**. The client enforces this automatically — calls that would exceed the limit will wait until the window resets.

## Next Steps

- [Finance Module](../finance.md) - Market-wide financial data
- [Getting Started](../getting-started.md) - Feature flag setup
