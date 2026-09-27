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

```rust no_run feature=polygon
use finance_query::format::Raw;
use finance_query::{Capability, Fetch, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await?;
    let ticker = providers.ticker("AAPL").build().await?;
    let quote = ticker.quote::<Raw>().await?;
    println!("{} quote received", quote.symbol);
    Ok(())
}
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

## Historical stock downloads

Use `discovery().stock_listings_page(&request, cursor)` with a `StockListingRequest` to fetch active or inactive stocks on a chosen date. The result preserves CIK, composite FIGI, share-class FIGI, and listing dates when supplied. `details_at(symbol, date)` returns dated details. `stock_types("us")` returns the provider's stock-type codes.

Use `market().stock_bars_page(&request, cursor)` with a `StockBarsRequest` for one-minute or daily bars. Set `PriceAdjustment::Unadjusted` or `SplitAdjusted` explicitly. The result preserves millisecond timestamps, fractional volume, and optional transaction counts. These calls do not change the existing chart API.

Save each page and its `next` cursor together. Pass that cursor with the same request to continue. A cursor can be serialized and resumed after restarting, but it cannot change provider, symbol, dates, or adjustment. An empty page with a next cursor is not a complete result. Page calls do not cache or collect the full history.

The crate bounds each page response to 32 MiB. The application must also bound concurrent requests and its retained pages. Authentication errors and malformed responses are errors, not empty history. Keep durable storage, retry scheduling, and session filtering in the application.

See [the runnable stock example](../../../examples/stock_ingestion.rs) and [the public-API tests](../../../tests/stock_ingestion_api.rs). The example performs six read-only requests using Polygon and FMP keys. Pass `--directory-only` for two small Polygon directory pages, with a serialized cursor between them.

## See Also

- [Multi-Provider Architecture](index.md) — Provider configuration and strategies
- [Ticker API](../ticker.md) — Single-symbol data access
