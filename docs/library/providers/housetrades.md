# House PTR (Congressional Trades)

!!! info "Feature flag required"
    ```toml
    finance-query = { version = "...", features = ["housetrades"] }
    ```

The STOCK Act of 2012 requires members of Congress to disclose trades over $1,000 within 45 days via a Periodic Transaction Report (PTR). The House Clerk publishes these free of charge and without credentials at `disclosures-clerk.house.gov`. This adapter reads directly from that primary source: no third-party aggregator, no API key.

Senate PTRs are a separate adapter, [`senatetrades`](senatetrades.md): `efdsearch.senate.gov` renders everything client-side behind Akamai bot protection, so it needs a real browser rather than a plain HTTP client. `Provider::CongressTrades` merges both when the corresponding features are compiled in; see [Providers Overview](index.md).

## Setup

```rust no_run feature=housetrades
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

With only `housetrades` compiled in, `Provider::CongressTrades` serves House rows exclusively, and every `CongressionalTrade.office` reads `"House"`. Enable `senatetrades` alongside it to merge in Senate rows too.

## How Symbol Lookup Works

There is no per-symbol or per-filing-type query endpoint on `disclosures-clerk.house.gov`; every individual disclosure is its own PDF. A symbol lookup works by:

1. Downloading the current year's bulk index archive (`{year}FD.zip`), a tab-delimited list of every filing that year, and filtering to Periodic Transaction Reports (`FilingType == "P"`).
2. Sorting by filing date (newest first) and taking the most recent 120 filings, topping up with the prior year's archive if the current year hasn't accumulated that many yet (early January).
3. Fetching each filing's PDF, extracting its text, and keeping only the transaction rows whose ticker matches the requested symbol.

!!! warning "Bounded recency window, not a full historical search"
    Only the most recent 120 PTR filings across all members of the House are scanned per request. This is a bounded-cost recency window, not an index over the full historical archive. A symbol traded only in an older filing beyond that window won't appear.

!!! warning "Scanned filings are silently skipped"
    Only filings typed through fd.house.gov's e-filing system carry a text layer this adapter can read. Older or hand-signed PTRs are scanned images; those produce zero matching transactions rather than an error, since this adapter does not OCR them. A 200-filing sample spread across 2021-2026 found 14.5% carrying no embedded font program at all, ranging from 10% to 20% by year.

!!! info "PDF text extraction carries no dependency"
    Text comes out of a purpose-built extractor covering the slice fd.house.gov emits: PDF 1.4, a classic cross-reference table, RC4-128 encryption under an empty user password, and CIDFontType2 subsets carrying `ToUnicode` CMaps. Column positions come from the text and graphics matrices plus the `/W` glyph advances the font already declares, so no font program is parsed. A filing outside that shape is reported rather than read as empty, since zero rows would otherwise be indistinguishable from a member disclosing no trades.

## What the Fields Mean

| `CongressionalTrade` field | Source |
|-----------------------------|--------|
| `symbol` | Ticker parsed from the PDF's asset description line |
| `first_name` / `last_name` | From the yearly filing index |
| `office` | Always `"House"` for this adapter |
| `district` | State plus district (e.g. `"AL-04"`), from the filing index |
| `trade_type` | `"Purchase"`, `"Sale"`, or `"Exchange"`, decoded from the PTR's `P`/`S`/`E` code |
| `amount` | Reported range as filed (e.g. `"$1,001 - $15,000"`), unparsed |
| `transaction_date` / `disclosure_date` | `YYYY-MM-DD` |
| `link` | Direct URL to the source PDF |

## Rate Limits

The House Clerk site publishes no documented rate limit. The client self-paces at 5 requests/second just to keep a symbol lookup's burst of PDF fetches polite.

## Next Steps

- [Senate PTR](senatetrades.md): the Senate-side counterpart
- [Filings](../filings.md): `Filings::congressional_trades()`
- [Providers Overview](index.md): routing and fallback
