# Filings API Reference

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Filings](https://docs.rs/finance-query/latest/finance_query/struct.Filings.html)

The `Filings` domain handle fetches SEC filings for a given symbol. It is backed by [EDGAR](providers/edgar.md) (keyless — no API key required) with an optional Polygon fallback, and is always available with no feature gate.

## Getting a Handle

Create a `Filings` handle from a [`Providers`](getting-started.md) instance and call `.get()` to fetch the filing data:

```rust no_run covers=finance_query::models::filings::provider::ProviderFiling
use finance_query::{Providers, edgar};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // EDGAR is keyless but SEC requires a contact email in the User-Agent.
    // Initialise it once per process (or set the EDGAR_EMAIL env var).
    edgar::init("you@example.com")?;

    let providers = Providers::builder().build().await?;
    let filings = providers.filings("AAPL");
    let result = filings.get().await?;

    for f in result.filings.iter().take(5) {
        println!(
            "{} {}: {}",
            f.filing_date.as_deref().unwrap_or("?"),
            f.filing_type.as_deref().unwrap_or("?"),
            f.filing_url.as_deref().unwrap_or("-")
        );
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::de_edgar_submissions.walltime.median_ns < 2000000 -->
- Parsing a company's full submissions index (`edgar::submissions` →
  `EdgarSubmissions`, roughly a thousand filings) takes **around a
  millisecond** or less.

!!! note "EDGAR requires a contact email"
    EDGAR needs no API key, but SEC's fair-access policy requires a contact email
    in the request `User-Agent`. Call `edgar::init("you@example.com")` once before
    fetching, or set the `EDGAR_EMAIL` environment variable.

<!-- soothfast:bind finance_query::models::filings::provider::ProviderFilings -->
The returned [`ProviderFilings`](https://docs.rs/finance-query/latest/finance_query/models/filings/struct.ProviderFilings.html) value contains the ticker symbol (`symbol`) and a list of individual filing entries (`filings`).
<!-- /soothfast:bind -->

Each entry is a `ProviderFiling`:

<!-- soothfast:bind finance_query::models::filings::provider::ProviderFiling -->

| Field | Type | Description |
|-------|------|-------------|
| `accession_number` | `Option<String>` | SEC accession number (unique filing ID) |
| `filing_date` | `Option<String>` | Filing date as `YYYY-MM-DD` |
| `filing_type` | `Option<String>` | Filing type (e.g., `"10-K"`, `"10-Q"`, `"8-K"`) |
| `filing_url` | `Option<String>` | URL to the filing document |
| `company_name` | `Option<String>` | Company name at time of filing |
| `cik` | `Option<String>` | SEC CIK number |

<!-- /soothfast:bind -->

## Congressional Trades

Call `.congressional_trades()` to fetch legislator stock-trade disclosures naming this symbol, filed under the STOCK Act as Periodic Transaction Reports (PTRs):

```rust no_run feature=housetrades,senatetrades covers=finance_query::models::filings::ownership::CongressionalTrade
use finance_query::{Capability, Provider, Providers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let providers = Providers::builder()
        .route(Capability::FILINGS, [Provider::CongressTrades, Provider::Edgar])
        .build()
        .await?;

    let filings = providers.filings("AAPL");
    for trade in filings.congressional_trades().await? {
        println!(
            "{} {}: {} {} on {}",
            trade.office.as_deref().unwrap_or("?"),
            trade.last_name.as_deref().unwrap_or("?"),
            trade.trade_type.as_deref().unwrap_or("?"),
            trade.amount.as_deref().unwrap_or("?"),
            trade.transaction_date.as_deref().unwrap_or("?")
        );
    }
    Ok(())
}
```

Two keyless sources feed this: the [House Clerk](providers/housetrades.md) (`housetrades` feature) and the [Senate eFD system](providers/senatetrades.md) (`senatetrades` feature), merged by `Provider::CongressTrades` when both are compiled in. Each result row's `office` field says which chamber it came from, `"House"` or `"Senate"`. If the Senate source fails (a network error, or Akamai bot protection blocking the request; see the Senate PTR page for why that happens), the merge drops just those rows and returns House-only results rather than failing the whole call. Only both sources failing (or the single compiled source failing, when just one is enabled) surfaces an error.

<!-- soothfast:bind finance_query::models::filings::ownership::CongressionalTrade -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `Option<String>` | Ticker symbol traded |
| `first_name` | `Option<String>` | Legislator's first name |
| `last_name` | `Option<String>` | Legislator's last name |
| `office` | `Option<String>` | `"House"` or `"Senate"`, depending on which source the row came from |
| `district` | `Option<String>` | District, for House members; always `None` for Senate rows |
| `trade_type` | `Option<String>` | Transaction type (e.g. `"Purchase"`, `"Sale"`) |
| `amount` | `Option<String>` | Reported transaction amount range (e.g. `"$1,001 - $15,000"`) |
| `asset_description` | `Option<String>` | Description of the asset traded |
| `transaction_date` | `Option<String>` | Date the transaction occurred (`YYYY-MM-DD`) |
| `disclosure_date` | `Option<String>` | Date the transaction was publicly disclosed (`YYYY-MM-DD`) |
| `link` | `Option<String>` | Link to the source disclosure filing |

<!-- /soothfast:bind -->

## Fails to Deliver

Call `.fails_to_deliver()` for this symbol's SEC fails-to-deliver history, the settlement-date record of shares that a broker-dealer failed to deliver on time:

```rust no_run feature=secftd covers=finance_query::models::filings::ownership::FailToDeliver
use finance_query::{Providers, edgar};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    edgar::init("you@example.com")?;

    let providers = Providers::builder().build().await?;
    let filings = providers.filings("AAPL");

    for row in filings.fails_to_deliver().await? {
        println!(
            "{}: {:?} shares of {:?}",
            row.date.as_deref().unwrap_or("?"),
            row.quantity,
            row.name
        );
    }
    Ok(())
}
```

This is served by keyless EDGAR (`secftd` feature), which the default `FILINGS` route already puts ahead of Yahoo, so no explicit `.route()` call is required once `secftd` is compiled in. FMP also serves this operation for callers who route `Capability::FILINGS` to it instead.

<!-- soothfast:bind finance_query::models::filings::ownership::FailToDeliver -->

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `Option<String>` | Ticker symbol |
| `date` | `Option<String>` | Settlement date (`YYYY-MM-DD`) |
| `quantity` | `Option<f64>` | Number of shares that failed to deliver |
| `price` | `Option<f64>` | Closing price on the settlement date |
| `name` | `Option<String>` | Security name |
| `description` | `Option<String>` | Additional description, when reported |

<!-- /soothfast:bind -->

## See Also

- [EDGAR Provider Reference](providers/edgar.md): low-level EDGAR API (CIK resolution, submissions, XBRL company facts, full-text search)
- [House PTR](providers/housetrades.md): House-side congressional trades source
- [Senate PTR](providers/senatetrades.md): Senate-side congressional trades source
- [Getting Started](getting-started.md): building a `Providers` instance
