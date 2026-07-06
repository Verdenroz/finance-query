# Polygon.io

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Provider::Polygon](https://docs.rs/finance-query/latest/finance_query/providers/enum.Provider.html#variant.Polygon)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["polygon"] }
    ```

Polygon.io provides real-time and historical market data for stocks, options, forex, crypto, indices, and futures. Free tier: 5 requests per second.

## Setup

Set the API key via environment variable:

```bash
export POLYGON_API_KEY="your-polygon-api-key"
```

No manual init call needed — the provider reads the key during `TickerBuilder::build()`.

## Usage

```rust
use finance_query::{Capability, Fetch, Provider, Providers, Raw};

let providers = Providers::builder()
    .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build()
    .await?;
let ticker = providers.ticker("AAPL").build().await?;
let quote = ticker.quote::<Raw>().await?;
```

## Capabilities

| Data type | Support |
|-----------|---------|
| Quote | ✓ |
| Chart | ✓ |
| Fundamentals | ✓ |
| Corporate | ✓ |
| Options | ✓ |
| Market | ✓ |
| Discovery | ✓ |
| Indices | ✓ |
| Commodities | — |
| Forex | ✓ |
| Crypto | ✓ |
| Futures | ✓ |
| Technicals | ✓ |
| Economic | ✓ |
| Filings | ✓ |
| Sentiment | ✓ |

## See Also

- [Multi-Provider Architecture](index.md) — Provider configuration and strategies
- [Ticker API](../ticker.md) — Single-symbol data access
