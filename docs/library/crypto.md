# Crypto (CoinGecko)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["crypto"] }
    ```

The `crypto` module provides cryptocurrency market data via the CoinGecko public API. No API key is required. Rate limiting (30 req/min on the free tier) is handled automatically.

```rust
use finance_query::crypto;
```

## Top Coins by Market Cap

```rust
use finance_query::crypto;

// Top 10 coins in USD
let top = crypto::coins("usd", 10).await?;

for coin in &top {
    let price   = coin.current_price.unwrap_or(0.0);
    let change  = coin.price_change_percentage_24h.unwrap_or(0.0);
    let rank    = coin.market_cap_rank.unwrap_or(0);
    println!("#{} {} ({}): ${:.2} ({:+.2}%)", rank, coin.name, coin.symbol, price, change);
}
```

- `vs_currency` — Quote currency: `"usd"`, `"eur"`, `"btc"`, `"eth"`, etc.
- `count` — Number of coins to return (max 250).

## Single Coin Lookup

```rust
// Look up by CoinGecko ID
let btc = crypto::coin("bitcoin", "usd").await?;
println!("Bitcoin: ${:.2}", btc.current_price.unwrap_or(0.0));

let eth = crypto::coin("ethereum", "usd").await?;
let mktcap = eth.market_cap.unwrap_or(0.0);
println!("Ethereum market cap: ${:.2}B", mktcap / 1e9);
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

## Rate Limits

The CoinGecko free tier allows **30 requests per minute**. The client enforces this automatically — calls that would exceed the limit will wait until the window resets.

## Next Steps

- [Finance Module](finance.md) - Market-wide financial data
- [Getting Started](getting-started.md) - Feature flag setup
