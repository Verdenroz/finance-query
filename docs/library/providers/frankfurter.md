# Frankfurter (ECB reference rates)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["frankfurter"] }
    ```

[Frankfurter](https://frankfurter.dev/) publishes the European Central Bank's daily reference exchange rates over a keyless JSON API — no registration, no API key.

It is the **only keyless `FOREX` route** in the library. Polygon, FMP, and Alpha Vantage all require a key, which meant `providers.forex()` did nothing out of the box; Frankfurter closes the same gap for forex that Yahoo covers for quotes, CoinGecko for crypto, and EDGAR for filings.

## Setup

```rust
use finance_query::{Capability, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::FOREX, [Provider::Frankfurter])
    .build()
    .await?;

let quote = providers.forex("USD", "EUR").quote().await?;
println!("USD/EUR {:?} ({:+.2?}%)", quote.price, quote.change_percent);
```

## What ECB Reference Rates Are

!!! warning "A daily fix, not a live feed"
    The ECB publishes one set of reference rates per TARGET working day, around 16:00 CET. They are indicative — a reference for accounting and statistics, not a tradable price.

That shapes the response:

| Field | Value from Frankfurter |
|-------|-----------------------|
| `symbol` | `"USDEUR"` — base and quote concatenated |
| `base_currency` / `quote_currency` | The pair you asked for, uppercased |
| `price` | The most recently published rate |
| `bid` / `ask` | Always `None` — a reference fix has no two-way price, and inventing a spread would misrepresent it |
| `change` / `change_percent` | Against the previously published rate, which is the prior *working* day, not necessarily yesterday |
| `timestamp` | Midnight UTC of the publication date — the ECB fix carries no intraday time |

If you need intraday rates or a real bid/ask, route `FOREX` to a keyed provider first and leave Frankfurter as the fallback:

```rust
let providers = Providers::builder()
    .route(Capability::FOREX, [Provider::Polygon, Provider::Frankfurter])
    .build()
    .await?;
```

## Coverage

Roughly 30 currencies — the set the ECB publishes against the euro, with any pair among them derivable. Notably **not** included: most emerging-market currencies, and cryptocurrencies. Requesting a currency the ECB does not publish returns `FinanceError::SymbolNotFound`.

Passing the same currency twice (`forex("USD", "USD")`) is answered locally with a rate of `1.0`; Frankfurter itself rejects that pair with HTTP 422.

## Historical Rates

`ForexPair::chart()` and `history()` route through `Capability::CHART`, which Frankfurter does not serve — those still resolve on the Yahoo default route (or whichever provider you route `CHART` to). Frankfurter contributes the current-rate path only.

## Rate Limits

Frankfurter is free and unmetered but community-run. The client paces itself at 5 requests/second, and a `429` surfaces as [`FinanceError::RateLimited`](../error-handling.md).

Each quote costs exactly one request: the adapter asks for a short date range rather than the `latest` endpoint, so the current rate and the previous close needed for the day's change arrive together.

## Next Steps

- [Forex Domain](../forex.md) — the `ForexPair` handle
- [Providers Overview](index.md) — routing and fallback
