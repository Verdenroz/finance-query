# Filings API Reference

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — Filings](https://docs.rs/finance-query/latest/finance_query/struct.Filings.html)

The `Filings` domain handle fetches SEC filings for a given symbol. It is backed by [EDGAR](providers/edgar.md) (keyless — no API key required) with an optional Polygon fallback, and is always available with no feature gate.

## Getting a Handle

Create a `Filings` handle from a [`Providers`](getting-started.md) instance and call `.get()` to fetch the filing data:

```rust
use finance_query::Providers;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let providers = Providers::builder().build().await?;
let filings = providers.filings("AAPL");
let result = filings.get().await?;
# Ok(()) }
```

The returned [`ProviderFilings`](https://docs.rs/finance-query/latest/finance_query/models/filings/struct.ProviderFilings.html) value contains the ticker symbol and a list of individual filing entries, each with fields like `filing_type`, `filing_date`, `accession_number`, and `filing_url`.

## See Also

- [EDGAR Provider Reference](providers/edgar.md) — low-level EDGAR API (CIK resolution, submissions, XBRL company facts, full-text search)
- [Getting Started](getting-started.md) — building a `Providers` instance
