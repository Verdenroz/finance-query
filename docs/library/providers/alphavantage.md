# Alpha Vantage

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Provider::AlphaVantage](https://docs.rs/finance-query/latest/finance_query/providers/enum.Provider.html#variant.AlphaVantage)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["alphavantage"] }
    ```

Alpha Vantage provides financial data with best-in-class technical indicator coverage (50+ indicators), plus economic indicators, forex, crypto, and commodities. Free tier: 25 requests per day.

## Setup

Set the API key via environment variable:

```bash
export ALPHAVANTAGE_API_KEY="your-alphavantage-key"
```

No manual init call needed — the provider reads the key during `TickerBuilder::build()`.

## Usage

```rust
use finance_query::{Capability, Fetch, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::QUOTE, &[Provider::AlphaVantage, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build()
    .await?;
let ticker = providers.ticker("AAPL").build().await?;
let quote = ticker.quote().await?;
```

## Capabilities

| Data type | Support |
|-----------|---------|
| Quote | ✓ |
| Chart | ✓ |
| Fundamentals | ✓ |
| Corporate | ✓ |
| Options | ✓ |
| Market | — |
| Discovery | — |
| Indices | — |
| Commodities | ✓ |
| Forex | ✓ |
| Crypto | ✓ |
| Futures | — |
| Technicals | ✓ |
| Economic | ✓ |
| Filings | — |
| Sentiment | — |

## See Also

- [Multi-Provider Architecture](index.md) — Provider configuration and strategies
- [Ticker API](../ticker.md) — Single-symbol data access
