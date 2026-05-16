# Multi-Provider Architecture

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — providers](https://docs.rs/finance-query/latest/finance_query/providers/index.html)

Finance Query v2.6 introduces a multi-provider architecture that lets you swap financial data providers through a single `TickerBuilder` API. Configure priority, dispatch strategy, and result merging without changing your data access code.

## Why Multiple Providers?

- **Redundancy** — if one provider fails, the next one in line takes over
- **Enrichment** — combine data from multiple providers (e.g., Polygon for price, FMP for financials)
- **Flexibility** — choose the provider that best fits your rate limits, coverage needs, and budget

## Available Providers

All providers are optional except Yahoo Finance (always available, keyless). Enable them via feature flags:

| Provider | Feature flag | Free tier | Env var for API key |
|----------|-------------|-----------|---------------------|
| **Yahoo Finance** | *(always available)* | Keyless | — |
| **Polygon.io** | `polygon` | 5 req/sec | `POLYGON_API_KEY` |
| **FMP** | `fmp` | 250 req/day | `FMP_API_KEY` |
| **Alpha Vantage** | `alphavantage` | 25 req/day | `ALPHA_VANTAGE_API_KEY` |
| **CoinGecko** | `crypto` | 30 req/min | *(keyless)* |
| **FRED** | `fred` | 120 req/min | `FRED_API_KEY` |

Add features in `Cargo.toml`:

```toml
[dependencies]
finance-query = { version = "2.6", features = ["polygon", "fmp"] }
```

## Configuring Providers

Use `TickerBuilder` to configure which providers to use, how they're queried, and how results are combined:

```rust
use finance_query::{Ticker, Provider, Fetch, Enrich};

let ticker = Ticker::builder("AAPL")
    .providers(&[Provider::Polygon, Provider::Fmp, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .merge(Enrich)
    .build()
    .await?;

// Data now comes from your configured provider chain
let quote = ticker.quote().await?;
let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
```

### Builder Methods

| Method | Default | Description |
|--------|---------|-------------|
| `.providers(&[Provider])` | `[Yahoo]` | Providers in priority order (first = primary) |
| `.fetch(Fetch)` | `Sequential` | How providers are queried |
| `.merge(MergePolicy)` | `Prefer` | How results from multiple providers are combined |

## Fetch Strategies

`Fetch` controls how the provider chain is queried:

```rust
use finance_query::Fetch;
```

| Strategy | Behavior | Best for |
|----------|----------|----------|
| `Fetch::Sequential` | Try providers in priority order; first success wins | Minimizing API calls, respecting rate limits |
| `Fetch::Parallel` | Fire all providers concurrently; first success wins | Lowest latency for real-time data |
| `Fetch::All` | Query all providers; collect all successes | Using `Merge` policies like `Enrich` |

```rust
// Fast failover: try Polygon first, fall back to Yahoo
let ticker = Ticker::builder("AAPL")
    .providers(&[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build()
    .await?;

// Low latency: race Polygon against Yahoo, use whichever responds first
let ticker = Ticker::builder("AAPL")
    .providers(&[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Parallel)
    .build()
    .await?;
```

!!! tip "Fetch::All"
    Use `Fetch::All` with `Enrich` to fill gaps. When all providers are queried, the merge policy can backfill missing fields rather than taking the first success.

## Merge Policies

`Merge` controls how results are combined when multiple providers succeed (relevant with `Fetch::All`):

| Policy | Behavior |
|--------|----------|
| `Prefer` | Use the primary (first-listed) provider's result; discard fallbacks (default) |
| `Enrich` | Primary wins; fallbacks fill in any `None` fields the primary didn't provide |

```rust
use finance_query::{Enrich, Prefer};

// Default: first provider always wins
let ticker = Ticker::builder("AAPL")
    .providers(&[Provider::Polygon, Provider::Yahoo])
    .merge(Prefer)
    .build()
    .await?;

// Enrich: Polygon primary, Yahoo fills missing fields
let ticker = Ticker::builder("AAPL")
    .providers(&[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::All)
    .merge(Enrich)
    .build()
    .await?;
```

!!! note "Merge is only meaningful with Fetch::All"
    `Prefer` and `Enrich` behave identically unless `Fetch::All` is used. With `Sequential` or `Parallel`, the first successful result is always returned and merge is never invoked.

## Provider Initialization

API keys are read from environment variables automatically during `TickerBuilder::build()`. No manual init calls needed for providers:

```bash
export POLYGON_API_KEY="your-polygon-key"
export FMP_API_KEY="your-fmp-key"
export ALPHA_VANTAGE_API_KEY="your-av-key"
export FRED_API_KEY="your-fred-key"
```

If a provider's API key is missing, `build()` returns an error. Yahoo and CoinGecko are keyless and need no setup.

!!! info "EDGAR init still required"
    The SEC EDGAR module (`edgar::init()` / `edgar::init_with_config()`) is separate from the provider system and still requires a one-time init call. See [EDGAR](edgar.md).

## Provider Capabilities

Each provider supports a different set of data types. When you call `ticker.quote()` or `ticker.options()`, the system automatically dispatches to providers that support that operation. Providers that don't support a given operation are silently skipped during fetch.

| Capability | Yahoo | Polygon | FMP | Alpha Vantage |
|------------|-------|---------|-----|---------------|
| Quote | ✓ | ✓ | ✓ | ✓ |
| Chart | ✓ | ✓ | ✓ | ✓ |
| Fundamentals | ✓ | ✓ | ✓ | ✓ |
| Corporate | ✓ | ✓ | ✓ | ✓ |
| Options | ✓ | ✓ | — | ✓ |
| Market | ✓ | ✓ | ✓ | — |
| Discovery | ✓ | ✓ | ✓ | — |
| Indices | — | ✓ | ✓ | — |
| Commodities | — | — | ✓ | ✓ |
| Forex | — | ✓ | ✓ | ✓ |
| Crypto | — | ✓ | ✓ | ✓ |
| Futures | — | ✓ | — | — |
| Technicals | — | ✓ | ✓ | ✓ |
| Economic | — | ✓ | — | ✓ |
| Filings | — | ✓ | — | — |
| Sentiment | — | ✓ | — | — |

CoinGecko supports only Crypto. FRED supports only Economic.

## Complete Example

```rust
use finance_query::{Ticker, Provider, Fetch, Enrich, Interval, TimeRange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Polygon as primary, FMP and Yahoo as fallbacks
    let ticker = Ticker::builder("AAPL")
        .providers(&[Provider::Polygon, Provider::Fmp, Provider::Yahoo])
        .fetch(Fetch::All)
        .merge(Enrich)
        .build()
        .await?;

    let quote = ticker.quote().await?;
    println!("{}: ${:.2}", quote.symbol,
        quote.regular_market_price.as_ref().and_then(|v| v.raw).unwrap_or(0.0));

    let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    println!("{} candles from provider {:?}", chart.candles.len(), chart.provider_id);

    Ok(())
}
```

## Provider Pages

Detailed provider-specific documentation:

| Provider | Documentation |
|----------|--------------|
| Yahoo Finance | *(default, always available)* |
| Polygon.io | [Polygon.io](polygon.md) |
| FMP | [Financial Modeling Prep](fmp.md) |
| Alpha Vantage | [Alpha Vantage](alphavantage.md) |
| CoinGecko | [Crypto (CoinGecko)](coingecko.md) |
| FRED | [FRED & Treasury](fred.md) |
| SEC EDGAR | [EDGAR SEC Filings](edgar.md) |

## Tickers and Providers

[`Tickers`](../tickers.md) supports the same `.providers()`, `.fetch()`, and `.merge()` builder methods as `Ticker`. Provider dispatch applies to every batch operation (quotes, charts, financials, etc.). `spark()` is the only exception — it uses a Yahoo-specific batch endpoint with no equivalent in other providers.

## Limitations

- Custom `Merge` implementations are not possible — only the built-in `Prefer` and `Enrich` policies are available
- Provider-specific fields not in the shared intermediate types are accessible via `.extras` fields (untyped)
- `Tickers.spark()` is Yahoo-only — no `fetch_spark` in `ProviderAdapter`
