# Financial Modeling Prep (FMP)

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Provider::Fmp](https://docs.rs/finance-query/latest/finance_query/providers/enum.Provider.html#variant.Fmp)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["fmp"] }
    ```

Financial Modeling Prep provides fundamentals, historical prices, insider trading data, institutional holdings, and screening. Free tier: 250 requests per day.

## Setup

Set the API key via environment variable:

```bash
export FMP_API_KEY="your-fmp-api-key"
```

No manual init call needed — the provider reads the key during `TickerBuilder::build()`.

## Usage

```rust
use finance_query::{Ticker, Provider, Fetch};

let ticker = Ticker::builder("AAPL")
    .providers(&[Provider::Fmp, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build()
    .await?;

let quote = ticker.quote().await?;
```

## Capabilities

| Data type | Support |
|-----------|---------|
| Quote | ✓ |
| Chart | ✓ |
| Fundamentals | ✓ |
| Corporate | ✓ |
| Options | — |
| Market | ✓ |
| Discovery | ✓ |
| Indices | ✓ |
| Commodities | ✓ |
| Forex | ✓ |
| Crypto | ✓ |
| Futures | — |
| Technicals | ✓ |
| Economic | — |
| Filings | — |
| Sentiment | — |

## See Also

- [Multi-Provider Architecture](index.md) — Provider configuration and strategies
- [Ticker API](../ticker.md) — Single-symbol data access
