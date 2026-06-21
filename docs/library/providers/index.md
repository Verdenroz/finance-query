# Multi-Provider Architecture

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — providers](https://docs.rs/finance-query/latest/finance_query/providers/index.html)

Finance Query v2.6 introduces a provider abstraction layer that lets you route each data capability (quotes, charts, fundamentals, etc.) to a different provider through a single builder API. The system automatically falls back to the next provider in the list on failure.

## Why Multiple Providers?

- **Redundancy** — if one provider fails or rate-limits you, the next one takes over
- **Capability coverage** — route each data type to the provider with the best coverage for it
- **Flexibility** — pick providers based on rate limits, data quality, and budget

## Available Providers

Yahoo Finance is always available with no configuration. All others are opt-in via feature flags:

| Provider | Feature flag | Free tier | Env var |
|----------|-------------|-----------|---------|
| **Yahoo Finance** | *(always available)* | Keyless | — |
| **Polygon.io** | `polygon` | 5 req/sec | `POLYGON_API_KEY` |
| **FMP** | `fmp` | 250 req/day | `FMP_API_KEY` |
| **Alpha Vantage** | `alphavantage` | 25 req/day | `ALPHAVANTAGE_API_KEY` |
| **CoinGecko** | `crypto` | 30 req/min | *(keyless)* |
| **FRED** | `fred` | 120 req/min | `FRED_API_KEY` |
| **SEC EDGAR** | *(always available)* | Keyless | *(email via `edgar::init`)* |

```toml
[dependencies]
finance-query = { version = "2.6", features = ["polygon", "fmp"] }
```

## Provider Initialization

API keys are read from environment variables automatically during `build()`. No manual init calls are needed:

```bash
export POLYGON_API_KEY="your-polygon-key"
export FMP_API_KEY="your-fmp-key"
export ALPHAVANTAGE_API_KEY="your-av-key"
export FRED_API_KEY="your-fred-key"
```

!!! info "EDGAR requires a one-time init"
    The SEC EDGAR module requires `edgar::init("user@example.com")?` once per process (SEC policy requires contact info for rate limiting). See [EDGAR](edgar.md).

## Capability Routing

Use `.route(Capability, &[Provider])` on `Providers::builder()` to assign providers to specific data capabilities, then create handles via `providers.ticker()`. Providers are tried in order — the first success wins.

```rust
use finance_query::{Capability, Fetch, Provider, Providers};

let providers = Providers::builder()
    // Route quotes to Polygon first, Yahoo as fallback
    .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
    // Route fundamentals to FMP first, Yahoo as fallback
    .route(Capability::FUNDAMENTALS, &[Provider::Fmp, Provider::Yahoo])
    // Route corporate (news, recommendations) to Polygon only
    .route(Capability::CORPORATE, &[Provider::Polygon])
    .fetch(Fetch::Sequential)
    .build()
    .await?;
let ticker = providers.ticker("AAPL").build().await?;
```

If no `.route()` is set for a capability, Yahoo Finance is used by default. EDGAR is auto-injected for `FILINGS` when no other provider is configured.

### Available Capabilities

| Capability | Constant | Description |
|------------|----------|-------------|
| Quote | `Capability::QUOTE` | Price, volume, market cap |
| Chart | `Capability::CHART` | Historical OHLCV data |
| Fundamentals | `Capability::FUNDAMENTALS` | Financial statements |
| Corporate | `Capability::CORPORATE` | News, recommendations, SEC metadata |
| Options | `Capability::OPTIONS` | Options chains |
| Crypto | `Capability::CRYPTO` | Cryptocurrency quotes |
| Economic | `Capability::ECONOMIC` | Macro series (GDP, CPI, etc.) |
| Forex | `Capability::FOREX` | FX currency pair rates |
| Indices | `Capability::INDICES` | Market index quotes |
| Futures | `Capability::FUTURES` | Futures contract quotes |
| Commodities | `Capability::COMMODITIES` | Commodity price quotes |
| Filings | `Capability::FILINGS` | SEC EDGAR filing data |

## Fetch Strategies

`Fetch` controls how the provider list is queried:

| Strategy | Behavior | Best for |
|----------|----------|----------|
| `Fetch::Sequential` | Try in priority order; first success wins **(default)** | Respecting rate limits, minimizing API calls |
| `Fetch::Parallel` | Fire all concurrently; first success wins | Lowest latency for real-time data |

```rust
use finance_query::{Capability, Fetch, Provider, Providers};

// Sequential: try Polygon, then Yahoo if Polygon fails
let providers = Providers::builder()
    .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build()
    .await?;
let ticker = providers.ticker("AAPL").build().await?;

// Parallel: race Polygon against Yahoo, use whichever responds first
let providers = Providers::builder()
    .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Parallel)
    .build()
    .await?;
let ticker = providers.ticker("AAPL").build().await?;
```

## Provider Capabilities Matrix

Capabilities supported by each provider. Providers that don't support a given capability are automatically skipped during dispatch.

| Capability | Yahoo | Polygon | FMP | Alpha Vantage | CoinGecko | FRED | EDGAR |
|------------|:-----:|:-------:|:---:|:-------------:|:---------:|:----:|:-----:|
| Quote | ✓ | ✓ | ✓ | ✓ | — | — | — |
| Chart | ✓ | ✓ | ✓ | ✓ | — | — | — |
| Fundamentals | ✓ | ✓ | ✓ | ✓ | — | — | — |
| Corporate | ✓ | ✓ | ✓ | ✓ | — | — | — |
| Options | ✓ | ✓ | — | ✓ | — | — | — |
| Crypto | — | ✓ | ✓ | ✓ | ✓ | — | — |
| Economic | — | ✓ | — | ✓ | — | ✓ | — |
| Forex | — | ✓ | ✓ | ✓ | — | — | — |
| Indices | — | ✓ | ✓ | — | — | — | — |
| Futures | — | ✓ | — | — | — | — | — |
| Commodities | — | — | ✓ | ✓ | — | — | — |
| Filings | — | ✓ | — | — | — | — | ✓ |
| Sentiment | — | ✓ | — | — | — | — | — |

## Providers Factory (Shared Connections)

For non-equity asset classes, use the `Providers` factory to create domain handles that share the same provider connections and configuration:

```rust
use finance_query::{Providers, Provider, Capability, Fetch};

let providers = Providers::builder()
    .route(Capability::FOREX, &[Provider::AlphaVantage])
    .route(Capability::ECONOMIC, &[Provider::Fred])
    .fetch(Fetch::Sequential)
    .build()
    .await?;

// All handles share the same provider connections
let aapl  = providers.ticker("AAPL").logo().build().await?;   // → Ticker
let pair  = providers.forex("USD", "EUR");                    // → ForexPair
let btc   = providers.crypto("bitcoin");                      // → CryptoCoin
let gdp   = providers.economic("REAL_GDP");                   // → EconomicIndicator
let spy   = providers.index("SPY");                           // → Index
let cl    = providers.futures("CL=F");                        // → FuturesContract
let wheat = providers.commodity("WHEAT");                     // → Commodity
let sec   = providers.filings("AAPL");                        // → Filings
```

### Domain Handle Methods

| Handle | Method | Returns |
|--------|--------|---------|
| `ForexPair` | `.quote()` · `.chart(interval, range)` · `.history(range)` | `ForexQuote` · `Chart` |
| `CryptoCoin` | `.quote(vs_currency)` · `.chart(vs_currency, interval, range)` · `.history(vs_currency, range)` | `CryptoQuote` · `Chart` |
| `EconomicIndicator` | `.series()` | `EconomicSeries` |
| `Index` | `.quote()` · `.chart(interval, range)` · `.history(range)` | `IndexQuote` · `Chart` |
| `FuturesContract` | `.quote()` · `.chart(interval, range)` · `.history(range)` | `FuturesQuote` · `Chart` |
| `Commodity` | `.quote()` · `.chart(interval, range)` · `.history(range)` | `CommodityQuote` · `Chart` |
| `Filings` | `.get()` | `ProviderFilings` |

All chart-capable handles route through `Capability::CHART` (Yahoo by default) and cache per `(symbol, interval, range)` when `.cache(ttl)` is set. `history(range)` is sugar for `chart(range.default_interval(), range)`. The handle's identifier is passed to the chart route as-is, so it must be a chart-route symbol (e.g. `^GSPC`, `NQ=F`, `GC=F`); `CryptoCoin` builds `"{ID}-{VS}"` (e.g. `"BTC-USD"`), which resolves on Yahoo only for ticker-style ids.

## Tickers and Providers

[`Tickers`](../tickers.md) supports the same multi-provider configuration as `Ticker`. Routing is configured through `Providers::builder()` and passed to `Tickers` via `providers.tickers()`:

```rust
use finance_query::{Capability, Fetch, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build()
    .await?;
let tickers = providers.tickers(["AAPL", "NVDA"]).build().await?;
```

!!! note "Spark is Yahoo-only"
    `spark()` uses a Yahoo-specific batch endpoint with no equivalent in other providers. It always uses the Yahoo client regardless of provider configuration.

## Provider Pages

| Provider | Documentation |
|----------|--------------|
| Polygon.io | [Polygon.io](polygon.md) |
| FMP | [Financial Modeling Prep](fmp.md) |
| Alpha Vantage | [Alpha Vantage](alphavantage.md) |
| CoinGecko | [Crypto (CoinGecko)](coingecko.md) |
| FRED | [FRED & Treasury](fred.md) |
| SEC EDGAR | [EDGAR SEC Filings](edgar.md) |
