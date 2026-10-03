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

These operations are library-only; the REST, GraphQL and MCP servers do not expose them.

`discovery().stock_listings_page(&request, cursor)` returns one page of the stock directory as of a date, active or inactive. Rows keep the CIK, composite FIGI, share-class FIGI and listing dates when Polygon supplies them, along with Polygon's reported status, locale and type. `details_at(symbol, date)` returns dated ticker details, and `stock_types("us")` lists Polygon's stock-type codes. `ticker_changes(id)` lists every ticker a security has traded under, oldest first; pass a composite FIGI to follow it through renames (FB from 2012-05-18, then META from 2022-06-09). Bars are filed under the ticker in use at the time, so a download before a rename needs the old ticker.

`market().stock_bars_page(&request, cursor)` returns one page of minute or daily bars. `StockBarsRequest::new` takes an explicit `PriceAdjustment`. Bars keep millisecond timestamps, fractional volume and optional transaction counts. Minute bars include pre-market and after-hours trading. How far back they go depends on the plan; older requests fail with `FinanceError::NotEntitled` rather than `AuthenticationFailed`.

Store each page's items and its `next` cursor together, then pass the cursor back with the same request to continue. Cursors serialize with serde, so a download can resume after a restart, but a cursor cannot switch provider or change the request. An empty page with a `next` cursor is not the end. Pages are not cached; the application bounds concurrency, retries and storage.

```rust no_run feature=polygon
use finance_query::{
    Capability, Interval, PageCursor, PriceAdjustment, Provider, Providers, StockBarsRequest,
    StockListingRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .providers([Provider::Polygon])
        .route(Capability::DISCOVERY, [Provider::Polygon])
        .route(Capability::CHART, [Provider::Polygon])
        .build()
        .await?;

    let listings = StockListingRequest::new("2020-01-02", false)?.with_stock_type("CS");
    let mut cursor: Option<PageCursor> = None;
    loop {
        let page = providers
            .discovery()
            .stock_listings_page(&listings, cursor.as_ref())
            .await?;
        // Persist page.items with the serialized cursor before continuing.
        let saved = serde_json::to_string(&page.next)?;
        cursor = serde_json::from_str(&saved)?;
        if cursor.is_none() {
            break;
        }
    }

    let bars = StockBarsRequest::new(
        "AAPL",
        "2020-01-02",
        "2020-01-02",
        Interval::OneMinute,
        PriceAdjustment::Unadjusted,
    )?;
    let page = providers.market().stock_bars_page(&bars, None).await?;
    println!("{} bars, more pages: {}", page.items.len(), page.next.is_some());

    let details = providers.discovery().details_at("AAPL", "2020-01-02").await?;
    let types = providers.discovery().stock_types("us").await?;
    println!("CIK {:?}, {} stock types", details.cik, types.len());
    Ok(())
}
```

## See Also

- [Multi-Provider Architecture](index.md) — Provider configuration and strategies
- [Ticker API](../ticker.md) — Single-symbol data access
