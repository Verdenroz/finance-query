# US Treasury FiscalData

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["fiscaldata"] }
    ```

[FiscalData](https://fiscaldata.treasury.gov/) is the US Treasury's own publishing platform for federal debt, interest rates, and the Daily Treasury Statement. It is **keyless** — no registration, no API key, no environment variable.

FRED mirrors part of this data with a lag and needs a key; FiscalData is the primary source.

## Setup

```rust no_run feature=fiscaldata
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await?;

    let debt = providers.economic("DEBT_TO_PENNY").series().await?;
    println!("{}", debt.title.unwrap_or_default());  // "Total Public Debt Outstanding"

    if let Some(latest) = debt.observations.last() {
        println!("{}: {:?}", latest.date, latest.value);
    }
    Ok(())
}
```

## Curated Series

FiscalData organises its data as ~50 datasets with dozens of columns each, so the adapter names the common single-value series directly:

| Series id | Dataset | Column | Units | Frequency |
|-----------|---------|--------|-------|-----------|
| `DEBT_TO_PENNY` | `v2/accounting/od/debt_to_penny` | `tot_pub_debt_out_amt` | US Dollars | Daily |
| `DEBT_HELD_BY_PUBLIC` | `v2/accounting/od/debt_to_penny` | `debt_held_public_amt` | US Dollars | Daily |
| `INTRAGOVERNMENTAL_HOLDINGS` | `v2/accounting/od/debt_to_penny` | `intragov_hold_amt` | US Dollars | Daily |
| `AVG_INTEREST_RATE` | `v2/accounting/od/avg_interest_rates` | `avg_interest_rate_amt` | Percent | Monthly |
| `AVG_INTEREST_RATE_MARKETABLE` | `v2/accounting/od/avg_interest_rates` | `avg_interest_rate_amt` | Percent | Monthly |
| `OPERATING_CASH_BALANCE` | `v1/accounting/dts/operating_cash_balance` | `open_today_bal` | Millions of US Dollars | Daily |

Series ids are matched case-insensitively.

The two `AVG_INTEREST_RATE*` entries read the same column with different row filters — `Total Interest-bearing Debt` and `Total Marketable` respectively — because that dataset stacks one series per security type.

!!! warning "Units differ per dataset"
    FiscalData's own metadata reports only `CURRENCY` / `PERCENTAGE`, which cannot distinguish dollars from millions of dollars. The curated table above states the real unit; Daily Treasury Statement figures (`OPERATING_CASH_BALANCE`) are in **millions**.

## Any Other Dataset

Anything outside the curated list is reachable with the passthrough form `"<dataset path>:<value column>"`:

```rust no_run feature=fiscaldata
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await?;

    let series = providers
        .economic("v2/accounting/od/debt_to_penny:debt_held_public_amt")
        .series()
        .await?;
    Ok(())
}
```

Dataset paths and column names come from the [FiscalData dataset catalogue](https://fiscaldata.treasury.gov/datasets/). Passthrough series report `units` from the column's declared type and leave `frequency` as `None` — the adapter will not guess.

Passing anything that is neither a curated id nor a `dataset:column` pair returns `FinanceError::InvalidParameter` listing the curated catalogue.

## Response Shape

Results come back as the provider-neutral [`EconomicSeries`](../economic.md):

| Field | Value from FiscalData |
|-------|----------------------|
| `series_id` | The string you passed in |
| `title` | The column's human label from the response metadata |
| `units` | From the curated table, or the column type for passthrough series |
| `frequency` | From the curated table; `None` for passthrough series |
| `observations` | Chronological (oldest first), keyed by `record_date` |

FiscalData encodes every column as a string, including numbers, and marks a missing figure with the literal string `"null"` — both are normalised, so `value` is a real `Option<f64>`.

## Pagination

Series are fetched at the API's maximum page size (10,000 rows) and pagination is followed automatically, capped at 5 pages. A dataset larger than that logs a warning rather than silently returning a truncated series.

## Rate Limits

FiscalData publishes no documented quota. The client paces itself at 5 requests/second, and a `429` surfaces as [`FinanceError::RateLimited`](../error-handling.md).

## Next Steps

- [FRED](fred.md) — 800k+ US macro series, including Treasury yield curves
- [World Bank](worldbank.md) — keyless global macro indicators
- [Providers Overview](index.md) — routing and fallback
