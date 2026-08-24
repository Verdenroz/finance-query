# Nasdaq (Market Calendars)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["nasdaq"] }
    ```

`Capability::CALENDAR` previously needed a keyed provider (FMP, Polygon, or Alpha Vantage) for earnings/IPO/dividend/split calendars. This adapter serves those four keylessly from `api.nasdaq.com`'s public calendar endpoints — the same undocumented API several third-party calendar wrappers already rely on.

## Setup

```rust no_run feature=nasdaq
use finance_query::{Capability, CalendarDetail, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::CALENDAR, [Provider::Nasdaq])
        .build()
        .await?;

    let calendar = providers.calendar();
    for entry in calendar.earnings("2026-08-24", "2026-08-24").await? {
        if let CalendarDetail::Earnings { eps_estimated, time, .. } = entry.detail {
            println!(
                "{}: est. EPS {:?} ({})",
                entry.symbol.as_deref().unwrap_or("?"),
                eps_estimated,
                time.as_deref().unwrap_or("?")
            );
        }
    }
    Ok(())
}
```

!!! note "Nasdaq serves four calendar kinds only"
    `earnings`, `ipos`, `dividends`, and `splits` are covered. `economic`,
    `holidays`, and `market_status` are `NotSupported` here — route
    `CALENDAR` to [FRED](fred.md) for economic releases, or rely on the
    always-on local holiday provider and Yahoo for the other two (see
    [Calendar Providers](index.md)).

## Date-Range Fan-Out

Nasdaq's calendar endpoints take no date-range parameter — each of `earnings`, `dividends`, and `splits` is queried one calendar day at a time, and `ipos` one calendar month at a time. A `[from, to]` request fans out into one HTTP call per day (or month) in range internally, so:

- `earnings`/`dividends`/`splits` reject ranges over 92 days with `InvalidParameter`, to keep a broad request from silently issuing hundreds of calls.
- `ipos` rejects ranges over 13 months for the same reason.
- A day (or month) that fails to fetch is dropped from the result rather than failing the whole range — a single bad day doesn't take down a multi-week query.

## Field Mapping Notes

- **Earnings**: `time` maps Nasdaq's `time-pre-market`/`time-after-hours` classes to `"bmo"`/`"amc"`; anything else comes back `None`. `revenue`/`revenue_estimated` are always `None` — Nasdaq's earnings calendar carries no revenue figures.
- **Dividends/Splits**: dates arrive as `M/D/YYYY` and are normalized to `YYYY-MM-DD`. Split ratios arrive as `"4 : 1"` and are parsed into separate `numerator`/`denominator` values.
- **IPOs**: each row is tagged with the deal-status section it came from (`"priced"`, `"expected"`, `"filed"`, `"withdrawn"`) in `actions`. `date` prefers `priced_date`, falling back through `expected_price_date`, `withdraw_date`, and `filed_date` in that order — whichever the row actually carries. `market_cap` is always `None`; Nasdaq's IPO calendar doesn't report it.

## Bot Detection

`api.nasdaq.com` drops any request whose `User-Agent` doesn't look like a browser — even a self-identifying agent gets the connection reset, confirmed against the live API. This adapter sends a plain browser-shaped `User-Agent` rather than the crate's usual self-identifying one.

## Rate Limits

Nasdaq documents no formal quota for these endpoints. The client self-paces at 2 requests/second, conservative given that a single range request can already fan out into dozens of calls.

## Next Steps

- [Market Calendars](../getting-started.md): `Providers::calendar()` and `Capability::CALENDAR` routing
- [FRED](fred.md): keyed economic-calendar alternative
- [Providers Overview](index.md): routing and fallback
