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

<!-- soothfast:bind finance_query::providers::Provider -->

| Provider | Feature flag | Free tier | Env var |
|----------|-------------|-----------|---------|
| **Yahoo Finance** | *(always available)* | Keyless | — |
| **Polygon.io** | `polygon` | 5 req/sec | `POLYGON_API_KEY` |
| **FMP** | `fmp` | 250 req/day | `FMP_API_KEY` |
| **Alpha Vantage** | `alphavantage` | 25 req/day | `ALPHAVANTAGE_API_KEY` |
| **CoinGecko** | `crypto` | 30 req/min | *(keyless)* |
| **FRED** | `fred` | 120 req/min | `FRED_API_KEY` |
| **World Bank** | `worldbank` | Keyless | *(keyless)* |
| **US Treasury FiscalData** | `fiscaldata` | Keyless | *(keyless)* |
| **BLS** | `bls` | Keyless 25/day, keyed 500/day | `BLS_API_KEY` *(optional)* |
| **Frankfurter** | `frankfurter` | Keyless | *(keyless)* |
| **Binance** | `binance` | Keyless | *(keyless)* |
| **Kraken** | `kraken` | Keyless | *(keyless)* |
| **FINRA** | `finra` | Keyless (non-commercial) | *(keyless)* |
| **OpenFIGI** | `openfigi` | Keyless 25 req/min | `OPENFIGI_API_KEY` *(optional)* |
| **DefiLlama** | `defi` | Keyless | *(keyless)* |
| **GDELT DOC 2.0** | `gdelt` | Keyless (~1 req/5s) | *(keyless)* |
| **CFTC** | `cftc` | Keyless | *(keyless)* |
| **Congressional Trades (House/Senate)** | `housetrades`, `senatetrades` | Keyless | *(keyless)* |
| **SEC EDGAR** | *(always available)* | Keyless | *(email via `edgar::init`)* |

<!-- /soothfast:bind -->

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

```rust no_run feature=full
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        // Route quotes to Polygon first, Yahoo as fallback
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        // Route fundamentals to FMP first, Yahoo as fallback
        .route(Capability::FUNDAMENTALS, [Provider::Fmp, Provider::Yahoo])
        // Route corporate (news, recommendations) to Polygon only
        .route(Capability::CORPORATE, [Provider::Polygon])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let ticker = providers.ticker("AAPL").build().await?;
    Ok(())
}
```

If no `.route()` is set for a capability, Yahoo Finance is used by default. EDGAR is auto-injected for `FILINGS` when no other provider is configured.

### Available Capabilities

<!-- soothfast:bind finance_query::providers::Capability -->

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

<!-- /soothfast:bind -->

Capabilities are bitflags — compose them with `|` and test membership with `contains`. This example runs as a real test, no network needed:

```rust capture-output covers=finance_query::providers::Capability
use finance_query::Capability;

let market_data = Capability::QUOTE | Capability::CHART;
assert!(market_data.contains(Capability::QUOTE));
assert!(market_data.contains(Capability::CHART));
assert!(!market_data.contains(Capability::OPTIONS));
assert_eq!(Capability::QUOTE.name(), "quote");
println!("market_data = {market_data:?}");
println!("QUOTE.name() = {:?}", Capability::QUOTE.name());
```

```text soothfast-output
market_data = Capability(3)
QUOTE.name() = "quote"
```

## Fetch Strategies

`Fetch` controls how the provider list is queried:

<!-- soothfast:bind finance_query::providers::Fetch -->

| Strategy | Behavior | Best for |
|----------|----------|----------|
| `Fetch::Sequential` | Try in priority order; first success wins **(default)** | Respecting rate limits, minimizing API calls |
| `Fetch::Parallel` | Fire all concurrently; first success wins | Lowest latency for real-time data |

<!-- /soothfast:bind -->

`Fetch` and `Provider` are plain enums — constructing them never touches the network:

```rust capture-output covers=finance_query::providers::Provider
use finance_query::{Fetch, Provider};

let strategy = Fetch::Sequential;
assert_eq!(strategy, Fetch::Sequential);
assert_ne!(Fetch::Parallel, Fetch::Sequential);

// Yahoo is the default provider; EDGAR is likewise always compiled in.
assert_eq!(Provider::default(), Provider::Yahoo);
assert_eq!(Provider::Edgar.as_str(), "edgar");
println!("default provider = {:?}", Provider::default());
println!("Edgar.as_str()   = {:?}", Provider::Edgar.as_str());
```

```text soothfast-output
default provider = Yahoo
Edgar.as_str()   = "edgar"
```

```rust no_run feature=polygon covers=finance_query::providers::Fetch
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sequential: try Polygon, then Yahoo if Polygon fails
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let ticker = providers.ticker("AAPL").build().await?;

    // Parallel: race Polygon against Yahoo, use whichever responds first
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Parallel)
        .build()
        .await?;
    let ticker = providers.ticker("AAPL").build().await?;
    Ok(())
}
```

## Provider Capabilities Matrix

Capabilities supported by each provider. Providers that don't support a given capability are automatically skipped during dispatch.

<!-- soothfast:claim finance_query::dispatch_select.alloc.allocs <= 0 -->
<!-- soothfast:claim finance_query::dispatch_select.walltime.median_ns < 100 -->
- Capability dispatch over a full provider registry is branch-few bitflag
  filtering: selecting the providers for a request makes **zero allocations**
  and completes in **single-digit nanoseconds** — routing adds no measurable
  overhead to any call.

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

```rust no_run feature=full
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::AlphaVantage])
        .route(Capability::ECONOMIC, [Provider::Fred])
        .route(Capability::CRYPTO, [Provider::CoinGecko])
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
    Ok(())
}
```

Four handles are market-wide rather than symbol-scoped, so their factories take
no argument. `market()` and `snapshot()` are always available (movers are served
keylessly from Yahoo's screeners); `discovery()` and `calendar()` require at
least one of the `fmp`, `polygon`, or `alphavantage` features:

```rust,ignore
let disco = providers.discovery();   // → Discovery: symbol search, reference data, screeners
let cal   = providers.calendar();    // → MarketCalendar: earnings/IPO/dividend/split/economic calendars
let mkt   = providers.market();      // → Market: sector performance, movers
let snap  = providers.snapshot();    // → Snapshot: cross-market watchlist snapshots
let cat   = providers.economic_catalog(); // → EconomicCatalog: find macro series (needs fred/alphavantage/polygon)
```

### Domain Handle Methods

| Handle | Method | Returns |
|--------|--------|---------|
| `ForexPair` | `.quote()` · `.chart(interval, range)` · `.history(range)` | `ForexQuote` · `Chart` |
| `CryptoCoin` | `.quote(vs_currency)` · `.chart(vs_currency, interval, range)` · `.history(vs_currency, range)` | `CryptoQuote` · `Chart` |
| `EconomicIndicator` | `.series()` · `.as_of(date)` | `EconomicSeries` |
| `EconomicCatalog` | `.search(query, limit)` · `.categories(parent_id)` · `.releases()` | `Vec<EconomicSeriesMatch>` · `Vec<EconomicCategory>` · `Vec<EconomicRelease>` |
| `Index` | `.quote()` · `.chart(interval, range)` · `.history(range)` · `.constituents()` · `.constituent_changes()` | `IndexQuote` · `Chart` · `Vec<IndexConstituent>` · `Vec<IndexConstituentChange>` |
| `FuturesContract` | `.quote()` · `.chart(interval, range)` · `.history(range)` | `FuturesQuote` · `Chart` |
| `Commodity` | `.quote()` · `.chart(interval, range)` · `.history(range)` | `CommodityQuote` · `Chart` |
| `Filings` | `.get()` · `.search(query, filters)` · `.search_all(query, filters)` · `.insider_trades(limit)` · `.institutional_holdings()` · `.sections(accession, form)` · `.risk_factors()` | `ProviderFilings` · `Vec<FilingSearchHit>` · `Vec<InsiderTrade>` · `Vec<InstitutionalHolding>` · `Vec<FilingSection>` · `Vec<RiskFactor>` |
| `Discovery` | `.search(query, limit)` · `.details(symbol)` · `.exchanges()` · `.listing_status(active)` · `.screener(filters)` | `Vec<SymbolMatch>` · `SymbolDetails` · `Vec<ExchangeInfo>` · `Vec<SymbolMatch>` · `Vec<ScreenerMatch>` |
| `MarketCalendar` | `.earnings(from, to)` · `.ipos(..)` · `.dividends(..)` · `.splits(..)` · `.economic(..)` · `.holidays()` | `Vec<MarketCalendarEntry>` |
| `Market` | `.sector_performance()` · `.sector_performance_history(limit)` · `.sector_pe()` · `.industry_pe()` · `.gainers()` · `.losers()` · `.most_active()` | `Vec<SectorPerformance>` · `Vec<SectorPerformanceHistory>` · `Vec<SectorPe>` · `Vec<IndustryPe>` · `Vec<MoverQuote>` |
| `Snapshot` | `.get(symbols)` | `Vec<MarketSnapshot>` |

`Snapshot::get` takes provider-spelled symbols from any market in one list
(`"AAPL"`, `"X:BTCUSD"`, `"I:SPX"`, `"C:EURUSD"`, `"O:NCLH221014C00005000"`) and
answers them in a single request — one rate-limit unit instead of one per asset
class. Symbols the provider cannot resolve come back as rows with `error` set,
so the result stays aligned with the request. Polygon (max 250 symbols) is
currently the only provider whose snapshot endpoint spans markets.

All chart-capable handles route through `Capability::CHART` (Yahoo by default) and cache per `(symbol, interval, range)` when `.cache(ttl)` is set. `history(range)` is sugar for `chart(range.default_interval(), range)`. The handle's identifier is passed to the chart route as-is, so it must be a chart-route symbol (e.g. `^GSPC`, `NQ=F`, `GC=F`); `CryptoCoin` builds `"{ID}-{VS}"` (e.g. `"BTC-USD"`), which resolves on Yahoo only for ticker-style ids.

### Technical Indicators & Risk on Domain Handles

With the `indicators` / `risk` features, every chart-capable handle also exposes the same analytics as `Ticker`, computed over its cached chart:

| Method | Feature | Returns |
|--------|---------|---------|
| `.indicators(interval, range)` | `indicators` | `IndicatorsSummary` |
| `.indicator(Indicator, interval, range)` | `indicators` | `IndicatorResult` |
| `.risk(interval, range)` | `risk` | `RiskSummary` |

`CryptoCoin` takes a leading `vs_currency` argument on all three (e.g. `coin.indicator(Indicator::Rsi(14), "USD", interval, range)`), matching its `chart()`.

`risk()` annualizes with the handle's asset-class trading calendar — 252 days for exchange-traded (index/futures/commodity), ~260 for forex, 365 for crypto (24/7) — and intraday intervals scale by session length, so Sharpe/Sortino/Calmar are correct across asset classes and intervals. `beta` is always `None` on domain handles (no benchmark is fetched).

## Tickers and Providers

[`Tickers`](../tickers.md) supports the same multi-provider configuration as `Ticker`. Routing is configured through `Providers::builder()` and passed to `Tickers` via `providers.tickers()`:

```rust no_run feature=polygon
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let tickers = providers.tickers(["AAPL", "NVDA"]).build().await?;
    Ok(())
}
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
