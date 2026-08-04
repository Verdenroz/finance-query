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
use finance_query::{Capability, Fetch, Provider, Providers, Raw};

let providers = Providers::builder()
    .route(Capability::QUOTE, [Provider::AlphaVantage, Provider::Yahoo])
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
| Indices | — |
| Commodities | ✓ |
| Forex | ✓ |
| Crypto | ✓ |
| Futures | — |
| Technicals | ✓ |
| Economic | ✓ |
| Filings | — |
| Sentiment | — |

## Alpha Vantage-only methods

ETF composition has no other wired source; symbol search and the listing
universe route through `Capability::DISCOVERY`.

```rust,ignore
use finance_query::{Capability, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::FUNDAMENTALS, [Provider::AlphaVantage, Provider::Yahoo])
    .route(Capability::DISCOVERY, [Provider::AlphaVantage])
    .build()
    .await?;

// Fund profile plus portfolio holdings, heaviest first.
let etf = providers.ticker("QQQ").build().await?.etf_profile().await?;
for h in etf.holdings.iter().take(10) {
    println!("{:?} {:?}", h.symbol, h.weight);
}

let disco = providers.discovery();
let hits = disco.search("tesco", 10).await?;
let listed = disco.listing_status(true).await?;   // every active listing
let delisted = disco.listing_status(false).await?;
```

| Method | Returns | Alpha Vantage function |
|--------|---------|------------------------|
| `Ticker::etf_profile()` | `EtfProfile` | `ETF_PROFILE` |
| `Discovery::search(query, limit)` | `Vec<SymbolMatch>` | `SYMBOL_SEARCH` |
| `Discovery::listing_status(active)` | `Vec<SymbolMatch>` | `LISTING_STATUS` |
| `Discovery::exchanges()` | `Vec<ExchangeInfo>` | `LISTING_STATUS` (derived) |

`listing_status` is an unfiltered dump of the whole universe — thousands of
rows in one response — so prefer `search` when you have a query. Alpha Vantage
has no exchange endpoint, so `exchanges()` derives the distinct venue list from
that same CSV and can only populate `name`.

## See Also

- [Multi-Provider Architecture](index.md) — Provider configuration and strategies
- [Ticker API](../ticker.md) — Single-symbol data access
