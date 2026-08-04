# BLS (Bureau of Labor Statistics)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["bls"] }
    ```

The [BLS Public Data API](https://www.bls.gov/developers/) is the primary source for US CPI, unemployment, payrolls, average hourly earnings, and PPI. Alpha Vantage exposes a fixed handful of these, and FRED mirrors them with a lag behind a key — BLS publishes them first.

## Keyless / keyed dual mode

BLS is the only provider in the library that works both with and without a key, and it decides per call:

| | No key | `BLS_API_KEY` set |
|---|--------|-------------------|
| API version | v1 | v2 |
| Daily quota | 25 queries per IP | 500 queries |
| History per request | ~3 years | 20 years |
| Series title | not returned | returned (`catalog`) |

Nothing else differs — the same series ids, the same `EconomicSeries` response. Get a free key at [data.bls.gov/registrationEngine](https://data.bls.gov/registrationEngine/) and export it:

```bash
export BLS_API_KEY="your-bls-key"
```

The tier is resolved on every request, so exporting a key mid-process takes effect immediately.

## Setup

```rust
use finance_query::{Capability, Provider, Providers};

let providers = Providers::builder()
    .route(Capability::ECONOMIC, [Provider::Bls])
    .build()
    .await?;

let cpi = providers.economic("CUUR0000SA0").series().await?;
for obs in cpi.observations.iter().rev().take(3) {
    println!("{} {:?}", obs.date, obs.value);
}
```

## Series Identifiers

Series are addressed by their native BLS id. Common ones:

| Series id | Description | Frequency |
|-----------|-------------|-----------|
| `CUUR0000SA0` | CPI-U, all items, US city average (NSA) | Monthly |
| `CUSR0000SA0` | CPI-U, all items, US city average (SA) | Monthly |
| `CUUR0000SA0L1E` | CPI-U, all items less food and energy | Monthly |
| `LNS14000000` | Unemployment rate | Monthly |
| `LNS11300000` | Labor force participation rate | Monthly |
| `CES0000000001` | Total nonfarm payroll employment | Monthly |
| `CES0500000003` | Average hourly earnings, private | Monthly |
| `WPUFD4` | PPI, final demand | Monthly |

Full series-id structure is documented in the [BLS series-id guide](https://www.bls.gov/help/hlpforma.htm).

## Response Shape

Results come back as the provider-neutral [`EconomicSeries`](../economic.md):

| Field | Value from BLS |
|-------|---------------|
| `series_id` | The BLS id you passed in |
| `title` | The catalog series title — keyed v2 only, `None` on the keyless route |
| `units` | Always `None`; BLS returns no unit field, and the unit is implied by the series id |
| `frequency` | `"Monthly"`, `"Quarterly"`, `"Semiannual"`, or `"Annual"`, from the period codes |
| `observations` | Chronological (oldest first) |

Two BLS conventions are normalised:

- A value of `"-"` (a figure BLS could not publish — e.g. the 2025 appropriations lapse) becomes `value: None`, not a parse error.
- **Annual-aggregate rows are dropped.** BLS folds an annual average into an otherwise monthly series as period `M13` (and `Q05` / `S03` for quarterly and semiannual series). Keeping them would put two observations on the same year with incompatible meanings, so they are omitted.

## Errors

BLS answers an unknown series id with `REQUEST_SUCCEEDED`, an empty data array, and the complaint in a `message` field. That surfaces as `FinanceError::SymbolNotFound` carrying the BLS text, rather than as a silent empty series. A genuine failure status (quota exhausted, malformed request) surfaces as `FinanceError::MacroDataError`.

## Rate Limits

The BLS quota is **daily**, not per-second, so it cannot be enforced client-side; the client paces at 2 requests/second only to avoid looking abusive. Exhausting the daily quota returns a `REQUEST_NOT_PROCESSED` status, which surfaces as `MacroDataError` with the BLS explanation.

## Next Steps

- [FRED](fred.md) — the same series mirrored, plus 800k others
- [US Treasury FiscalData](fiscaldata.md) — keyless federal fiscal data
- [Providers Overview](index.md) — routing and fallback
