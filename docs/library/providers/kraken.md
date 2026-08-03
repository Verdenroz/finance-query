# Kraken (public market data)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["kraken"] }
    ```

Kraken's public endpoints (`api.kraken.com/0/public/*`) need no API key and impose **no geo-block** — which is the reason this provider exists alongside [Binance](binance.md), whose data is richer but unavailable to US retail users.

Two exchanges also give `Capability::CRYPTO` a real fallback chain:

```rust
use finance_query::{Capability, Fetch, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::CRYPTO, [Provider::Binance, Provider::Kraken, Provider::CoinGecko])
    .fetch(Fetch::Sequential)
    .build()
    .await?;
```

## Capabilities

| Capability | What Kraken serves |
|------------|-------------------|
| `CRYPTO` | 24-hour ticker per spot pair |
| `CHART` | OHLC candles |

```rust
let btc = providers.crypto("bitcoin");
let quote = btc.quote("usd").await?;
println!("BTC {:?}", quote.price);
```

## Kraken's Own Symbol Conventions

Kraken predates most ticker conventions and kept its own:

| Everyone else | Kraken |
|---------------|--------|
| `BTC` | `XBT` |
| `DOGE` | `XDG` |
| `BTC/USD` | `XXBTZUSD` (legacy `X`/`Z` asset-class prefixes) |

None of that reaches your call sites. Pass normal tickers, coin ids, or separated pairs and the adapter translates in both directions:

| You pass | Kraken pair |
|----------|-------------|
| `crypto("bitcoin").quote("usd")` | `XBTUSD` |
| `crypto("DOGE").quote("eur")` | `XDGEUR` |
| chart symbol `BTC-USD` | `XBTUSD` |
| chart symbol `SOLUSD` | `SOLUSD` |

Kraken also answers with a pair name that differs from the one requested (`XBTUSD` comes back keyed as `XXBTZUSD`), so the adapter reads the response positionally rather than looking up the name it sent.

## Response Notes

`CryptoQuote`:

- `volume_24h`, `high_24h`, `low_24h` come from the **rolling 24-hour** half of Kraken's `[today, last 24 hours]` arrays, not today's figures.
- `change_24h` and `change_percent_24h` are measured against **today's opening price** (00:00 UTC). Kraken publishes no price from exactly 24 hours ago, so early in the UTC day this is a shorter-window change than the field name suggests.
- `market_cap` and `circulating_supply` are always `None` — an exchange knows flow, not supply. Route `CRYPTO` to CoinGecko for those.

`Chart`:

- Kraken timestamps candles in seconds already, so no conversion is applied.
- `adj_close` mirrors `close` — crypto has no corporate actions.

## Intervals and History Depth

Every library interval maps to a Kraken bucket except `ThreeMonths`, which returns `NotSupported` so sequential routing falls through. `OneMonth` maps to Kraken's longest bucket, 15 days.

!!! warning "Roughly 720 candles maximum"
    Kraken's `/OHLC` endpoint returns at most ~720 candles ending at the present, and its `since` parameter only moves the window's *start* forward — there is no way to page further back. A range wider than 720 candles returns the most recent 720, not the full window. For deep history, route `CHART` to Binance (which is paged automatically) or to a keyed provider.

## Errors

Kraken answers a rejected request with **HTTP 200** and a populated `error` array, so the status code alone never means success. An unknown pair surfaces as `FinanceError::SymbolNotFound`; anything else carries Kraken's own error text.

## Rate Limits

Kraken's public counter allows roughly one call per second sustained for unauthenticated clients, and the client paces to match. That is deliberately slower than the Binance adapter — put Binance first in a chain if throughput matters and your region allows it.

## Next Steps

- [Binance](binance.md) — deeper history, geo-blocked in some regions
- [CoinGecko](coingecko.md) — aggregated market caps and supply
- [Crypto Domain](../crypto.md) — the `CryptoCoin` handle
