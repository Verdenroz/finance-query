# Binance (public market data)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["binance"] }
    ```

Binance's public market-data endpoints need no API key. The adapter talks to `data-api.binance.vision` — the market-data-only host, which serves the same public endpoints as `api.binance.com` but carries no account or trading routes, so there is no key to configure and none to leak.

This is the first keyless source of *exchange-grade* crypto data in the library: CoinGecko is aggregated and coarse, and the keyed providers charge for OHLCV.

!!! warning "Geo-blocked in some regions"
    Binance restricts some regions (notably US retail) and answers with HTTP 451. That surfaces as a `FinanceError::ApiError` naming [Kraken](kraken.md) as the alternative. Chain the two if you need coverage everywhere.

## Capabilities

| Capability | What Binance serves |
|------------|--------------------|
| `CRYPTO` | Rolling 24-hour quote per spot market |
| `CHART` | Arbitrary-interval OHLCV klines |

```rust
use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

let providers = Providers::builder()
    .route(Capability::CRYPTO, [Provider::Binance])
    .route(Capability::CHART, [Provider::Binance])
    .build()
    .await?;

let btc = providers.crypto("bitcoin");
let quote = btc.quote("usd").await?;
println!("BTC {:?} ({:+.2?}%)", quote.price, quote.change_percent_24h);

let chart = btc.chart("usd", Interval::OneHour, TimeRange::OneMonth).await?;
println!("{} hourly candles", chart.candles.len());
```

!!! note "Routing `CHART` is global"
    `Capability::CHART` is one route for the whole library. Pointing it at Binance means equity charts go there too — Binance answers a symbol it cannot map to a spot market with `NotSupported`, so under `Fetch::Sequential` list Yahoo after it:
    `.route(Capability::CHART, [Provider::Binance, Provider::Yahoo])`

## Symbols

Binance names a market as one concatenated string with no separator (`BTCUSDT`). Every spelling the library uses is accepted:

| You pass | Binance market |
|----------|---------------|
| `crypto("bitcoin").quote("usd")` | `BTCUSDT` |
| `crypto("BTC").quote("eur")` | `BTCEUR` |
| chart symbol `BTC-USD` | `BTCUSDT` |
| chart symbol `BTCUSDT` | `BTCUSDT` |
| chart symbol `ETH/BTC` | `ETHBTC` |

CoinGecko-style coin ids are recognised for about 25 majors so a route swap between CoinGecko and Binance does not change your call sites. Anything not in that list is treated as a ticker, which is usually right.

!!! warning "USD means USDT"
    Binance spot lists **no USD markets** — dollar pairs are quoted in the USDT stablecoin. `"usd"` is therefore mapped to `USDT`. That is what callers mean in practice, but USDT is not literally the US dollar and can trade off its peg.

## Response Notes

`CryptoQuote`:

- `volume_24h` is the **quote-asset** volume (dollars for a `*USDT` market), which is the figure comparable to other providers — not the base-asset volume Binance also returns.
- `market_cap` and `circulating_supply` are always `None`. An exchange knows what trades on it, not how much of an asset exists; route `CRYPTO` to CoinGecko for supply-side figures.
- `name` is the full coin name for the recognised majors, and the ticker otherwise.

`Chart`:

- Binance timestamps candles in milliseconds; they are converted to the library's seconds.
- `adj_close` mirrors `close` — crypto has no corporate actions to adjust for.
- `volume` is base-asset volume truncated to an integer to fit the `Candle` model, so sub-unit volumes round toward zero.

## Intervals

Every library interval maps to a Binance kline code except `ThreeMonths`, which Binance does not offer — that returns `NotSupported`, so sequential routing falls through to the next provider.

Binance caps a single kline response at 1000 candles. Longer windows are walked forward automatically, up to 10 requests (10,000 candles) per chart — enough for five years of daily or a year of hourly data.

## Rate Limits

Binance meters by request weight (6000 per minute per IP); the endpoints used here are weight 1–2. The client paces at 10 requests/second, well inside that. A `429` — or a `418`, Binance's "you ignored a 429" ban — surfaces as [`FinanceError::RateLimited`](../error-handling.md).

## Next Steps

- [Kraken](kraken.md) — the US-accessible sibling
- [CoinGecko](coingecko.md) — aggregated market caps and supply
- [Crypto Domain](../crypto.md) — the `CryptoCoin` handle
