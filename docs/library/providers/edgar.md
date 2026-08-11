# EDGAR API Reference

!!! abstract "Cargo Docs"
    [docs.rs/finance-query — edgar](https://docs.rs/finance-query/latest/finance_query/edgar/index.html)

The EDGAR module provides access to SEC (Securities and Exchange Commission) EDGAR filings and XBRL financial data. All EDGAR APIs are free and public, requiring only a proper User-Agent header with a contact email. The module is always available — no feature flag needed.

!!! info "Contact Email Required"
    SEC EDGAR requires all automated requests to include a User-Agent header with a valid contact email address. Call `edgar::init(email)` once per process.

## Initialization

### Basic Initialization

```rust capture-output
use finance_query::edgar;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Required: Contact email for SEC compliance
    edgar::init("user@example.com")?;
    Ok(())
}
```

```text soothfast-output
```

### Advanced Configuration

```rust capture-output
use finance_query::edgar;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    edgar::init_with_config(
        "user@example.com",
        "my-financial-app",      // Optional: default is "finance-query"
        Duration::from_secs(60), // Optional: default is 30 seconds
    )?;
    Ok(())
}
```

```text soothfast-output
```

## Rate Limiting

The EDGAR client automatically handles SEC's rate limit of 10 requests per second. You don't need to manage rate limiting manually.

```rust no_run
use finance_query::edgar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    edgar::init("user@example.com")?;

    // These requests are automatically rate-limited
    let cik1 = edgar::resolve_cik("AAPL").await?;
    let cik2 = edgar::resolve_cik("MSFT").await?;
    let cik3 = edgar::resolve_cik("GOOGL").await?;
    // Executed at max 10 req/sec automatically
    Ok(())
}
```

## Ticker to CIK Resolution

Convert a stock ticker symbol to its SEC Central Index Key (CIK) number. The ticker-to-CIK mapping is fetched once and cached for the lifetime of the client.

```rust no_run
use finance_query::edgar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    edgar::init("user@example.com")?;

    // Resolve ticker to CIK (cached after first fetch)
    let cik = edgar::resolve_cik("AAPL").await?;
    println!("Apple CIK: {}", cik); // Output: Apple CIK: 320193

    // Subsequent lookups use the cache (no network request)
    let cik2 = edgar::resolve_cik("AAPL").await?; // Instant

    // Case-insensitive lookup
    let cik3 = edgar::resolve_cik("aapl").await?; // Also works
    Ok(())
}
```

### CIK Structure

A CIK is a unique 10-digit identifier assigned by the SEC to companies and individuals who file with the commission. Examples:

- Apple Inc.: `320193`
- Microsoft Corp.: `789019`
- Alphabet Inc.: `1652044`

## Filing History (Submissions)

Fetch filing history and company metadata from SEC EDGAR. Returns the most recent ~1000 filings inline, with references to additional history files for older filings.

```rust no_run covers=finance_query::models::filings::submissions::EdgarSubmissions
use finance_query::edgar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    edgar::init("user@example.com")?;

    // Get CIK first
    let cik = edgar::resolve_cik("AAPL").await?;

    // Fetch filing history
    let submissions = edgar::submissions(cik).await?;

    // Company information
    if let Some(name) = &submissions.name {
        println!("Company: {}", name);
    }
    if let Some(cik) = &submissions.cik {
        println!("CIK: {}", cik);
    }
    if let Some(sic) = &submissions.sic {
        println!("SIC: {}", sic);
    }
    if let Some(fiscal_year_end) = &submissions.fiscal_year_end {
        println!("Fiscal Year End: {}", fiscal_year_end);
    }

    // Recent filings
    if let Some(filings) = &submissions.filings
        && let Some(recent) = &filings.recent
    {
        for i in 0..5.min(recent.accession_number.len()) {
            let form = &recent.form[i];
            let date = &recent.filing_date[i];
            let accession = &recent.accession_number[i];

            println!("{} filed on {}: {}", form, date, accession);
        }
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::de_edgar_submissions.walltime.median_ns < 2000000 -->
- Parsing a company's full submissions JSON (~1,000 recent filings) into
  `EdgarSubmissions` takes **under 2 ms**.

### Submission Structure

<!-- soothfast:bind finance_query::models::filings::submissions::EdgarSubmissions -->

The `EdgarSubmissions` response contains:

- **Company Info**: `name`, `cik`, `sic`, `sic_description`, `fiscal_year_end`
- **Filings**: Recent filings with form types, dates, accession numbers, and document URLs
- **History**: References to additional filing history files for older filings

<!-- /soothfast:bind -->

### Common Form Types

- **10-K**: Annual report with comprehensive company information
- **10-Q**: Quarterly report
- **8-K**: Current report for major events
- **DEF 14A**: Proxy statement for shareholder meetings
- **S-1**: Registration statement for IPOs
- **4**: Insider trading report

## Company Facts (XBRL Data)

Fetch structured XBRL financial data from SEC EDGAR. Returns all extracted XBRL facts organized by taxonomy (us-gaap, ifrs, dei). This can be a large response (several MB for major companies).

```rust no_run covers=finance_query::models::filings::company_facts::CompanyFacts
use finance_query::edgar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    edgar::init("user@example.com")?;

    // Get CIK
    let cik = edgar::resolve_cik("AAPL").await?;

    // Fetch company facts
    let facts = edgar::company_facts(cik).await?;

    // Access financial data by taxonomy and concept
    if let Some(us_gaap) = facts.facts.get("us-gaap") {
        // Revenue data (FactsByTaxonomy is a tuple struct, access with .0)
        if let Some(revenue) = us_gaap.0.get("Revenues") {
            if let Some(label) = &revenue.label {
                println!("Revenue concept: {}", label);
            }
            if let Some(description) = &revenue.description {
                println!("Description: {}", description);
            }

            // Access data points by unit (e.g., USD)
            if let Some(usd_data) = revenue.units.get("USD") {
                for point in usd_data.iter().take(5) {
                    if let (Some(fy), Some(val)) = (point.fy, point.val) {
                        println!("FY {}: ${}", fy, val);
                    }
                }
            }
        }

        // Assets data
        if let Some(assets) = us_gaap.0.get("Assets")
            && let Some(usd_data) = assets.units.get("USD")
        {
            for point in usd_data.iter().take(5) {
                if let (Some(fy), Some(val)) = (point.fy, point.val) {
                    println!("FY {}: ${}", fy, val);
                }
            }
        }
    }
    Ok(())
}
```

<!-- soothfast:claim finance_query::de_edgar_facts.walltime.median_ns < 10000000 -->
- Parsing a complete XBRL company-facts payload (several MB for a large filer)
  into `CompanyFacts` takes **under 10 ms** — network transfer, not parsing,
  dominates the call.

### Available Taxonomies

<!-- soothfast:bind finance_query::models::filings::company_facts::CompanyFacts -->

The `CompanyFacts` response maps taxonomy names to their extracted concepts:

- **us-gaap**: US Generally Accepted Accounting Principles (most common)
- **ifrs**: International Financial Reporting Standards
- **dei**: Document and Entity Information (metadata)

<!-- /soothfast:bind -->

### Common XBRL Concepts

**Income Statement:**
- `Revenues` / `RevenueFromContractWithCustomerExcludingAssessedTax`
- `NetIncomeLoss`
- `OperatingIncomeLoss`
- `GrossProfit`

**Balance Sheet:**
- `Assets`
- `Liabilities`
- `StockholdersEquity`
- `Cash` / `CashAndCashEquivalentsAtCarryingValue`

**Cash Flow:**
- `NetCashProvidedByUsedInOperatingActivities`
- `NetCashProvidedByUsedInInvestingActivities`
- `NetCashProvidedByUsedInFinancingActivities`

## Full-Text Search

Search SEC EDGAR filings by text content with optional filters for form type, date range, and pagination (`from` offset and `size` limit — pass `None` for the defaults).

```rust no_run
use finance_query::edgar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    edgar::init("user@example.com")?;

    // Basic search
    let results = edgar::search(
        "artificial intelligence",
        None,    // No form filter
        None,    // No start date
        None,    // No end date
        None,    // from (pagination offset)
        None,    // size (max results)
    ).await?;

    // Display results
    if let Some(hits) = &results.hits {
        if let Some(value) = hits.total.as_ref().and_then(|t| t.value) {
            println!("Total hits: {}", value);
        }

        for hit in &hits.hits {
            if let Some(source) = &hit._source {
                let form = source.form.as_deref().unwrap_or("Unknown");
                let file_date = source.file_date.as_deref().unwrap_or("Unknown");

                println!("{} filed on {}", form, file_date);

                if !source.display_names.is_empty() {
                    println!("  Companies: {:?}", source.display_names);
                }
            }
        }
    }
    Ok(())
}
```

### Filtered Search

```rust no_run
use finance_query::edgar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Search for 10-K filings only
    let results = edgar::search(
        "machine learning",
        Some(&["10-K"]),              // Only 10-K forms
        Some("2024-01-01"),           // From Jan 1, 2024
        Some("2024-12-31"),           // To Dec 31, 2024
        None,                         // from (pagination offset)
        None,                         // size (max results)
    ).await?;
    Ok(())
}
```

### Common Form Filters

```rust no_run
use finance_query::edgar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Annual reports
    edgar::search("query", Some(&["10-K"]), None, None, None, None).await?;

    // Quarterly reports
    edgar::search("query", Some(&["10-Q"]), None, None, None, None).await?;

    // Current events
    edgar::search("query", Some(&["8-K"]), None, None, None, None).await?;

    // Multiple form types
    edgar::search("query", Some(&["10-K", "10-Q", "8-K"]), None, None, None, None).await?;
    Ok(())
}
```

### Routed Search (`Filings` handle)

The same EFTS index is reachable through the `FILINGS` capability, which returns
the flattened `FilingSearchHit` model instead of EDGAR's raw Elasticsearch
envelope — and derives a direct archive URL per hit where possible.

```rust,ignore
use finance_query::{FilingSearchFilters, Providers};

let providers = Providers::builder().build().await?;
let sec = providers.filings("AAPL");

// Scoped to AAPL — the handle's symbol is resolved to a CIK filter.
let hits = sec.search(
    "artificial intelligence",
    FilingSearchFilters::default().forms(["10-K"]).from("2024-01-01"),
).await?;

// Across every filer.
let all = sec.search_all(
    "artificial intelligence",
    FilingSearchFilters::default().forms(["10-K"]).limit(50),
).await?;

for hit in hits {
    println!("{:?} {:?} {:?}", hit.form, hit.filed_date, hit.url);
}
```

EFTS caps a page at 100 hits, so `limit` above that is clamped.

## Ownership (Forms 3/4/5 and 13F-HR)

Both are parsed straight from the filed XML — primary-source data, keyless, and
structurally typed rather than scraped.

```rust,ignore
use finance_query::Providers;

let providers = Providers::builder().build().await?;

// Insider transactions from the most recent 10 Form 3/4/5 filings.
for trade in providers.filings("AAPL").insider_trades(10).await? {
    println!(
        "{:?} {:?} {:?} shares @ {:?}",
        trade.insider_name, trade.transaction_code, trade.shares, trade.price_per_share
    );
}

// The latest 13F-HR information table filed by a listed manager.
for position in providers.filings("BRK-B").institutional_holdings().await? {
    println!("{:?} {:?}", position.issuer_name, position.shares);
}
```

Notes:

- `limit` on `insider_trades` caps how many *filings* are read, not how many
  transactions come back — one Form 4 can report several lines. Each filing
  costs an index lookup plus a document fetch, so keep it modest.
- A Form 3 states initial holdings rather than transactions, so it contributes
  no rows. A filing whose XML cannot be parsed is skipped rather than failing
  the whole call — ownership schemas vary across two decades of filings.
- `institutional_holdings` treats the handle's symbol as the *filer*, so it is
  only meaningful for listed institutional managers. An issuer that files no
  13F returns an error rather than an empty list.
- `value` on a holding is passed through unscaled: filings before 2023 report
  thousands of dollars, later ones report whole dollars.

## Complete Example

Here's a complete example combining all EDGAR features:

```rust no_run
use finance_query::edgar;

async fn analyze_company(ticker: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create EDGAR client
    edgar::init("user@example.com")?;

    // Step 1: Resolve ticker to CIK
    println!("Resolving {} to CIK...", ticker);
    let cik = edgar::resolve_cik(ticker).await?;
    println!("CIK: {}\n", cik);

    // Step 2: Get filing history
    println!("Fetching filing history...");
    let submissions = edgar::submissions(cik).await?;
    if let Some(name) = &submissions.name {
        println!("Company: {}", name);
    }
    if let Some(sic_description) = &submissions.sic_description {
        println!("Industry: {}", sic_description);
    }

    // Show recent 10-K and 10-Q filings
    if let Some(filings) = &submissions.filings
        && let Some(recent) = &filings.recent
    {
        println!("\nRecent filings:");
        for i in 0..10.min(recent.form.len()) {
            let form = &recent.form[i];
            if form == "10-K" || form == "10-Q" {
                let date = &recent.filing_date[i];
                println!("  {} filed on {}", form, date);
            }
        }
    }

    // Step 3: Get company facts (XBRL data)
    println!("\nFetching XBRL financial data...");
    let facts = edgar::company_facts(cik).await?;

    if let Some(us_gaap) = facts.facts.get("us-gaap") {
        // Show revenue trend (FactsByTaxonomy is a tuple struct, access with .0)
        if let Some(revenue) = us_gaap.0.get("Revenues")
            && let Some(usd) = revenue.units.get("USD")
        {
            println!("\nRevenue Trend:");
            for point in usd.iter().take(5) {
                if let (Some(fy), Some(val)) = (point.fy, point.val) {
                    println!("  FY {}: ${:>15}", fy, val);
                }
            }
        }

        // Show assets
        if let Some(assets) = us_gaap.0.get("Assets")
            && let Some(usd) = assets.units.get("USD")
        {
            println!("\nAssets:");
            for point in usd.iter().take(3) {
                if let (Some(fy), Some(val)) = (point.fy, point.val) {
                    println!("  FY {}: ${:>15}", fy, val);
                }
            }
        }
    }

    // Step 4: Search for AI mentions in recent filings
    println!("\nSearching for 'artificial intelligence' mentions...");
    let search_results = edgar::search(
        "artificial intelligence",
        Some(&["10-K", "10-Q"]),
        Some("2024-01-01"),
        None,
        None,
        None,
    ).await?;

    if let Some(hits) = &search_results.hits {
        let count = hits.total.as_ref().and_then(|t| t.value).unwrap_or(0);
        println!("Found {} mentions", count);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    analyze_company("AAPL").await
}
```

## Best Practices

!!! tip "EDGAR API Usage"
    - **Respect Rate Limits**: The client automatically handles the 10 req/sec limit, but avoid making thousands of requests in quick succession
    - **Handle Large Responses**: Company facts can be several MB. Consider streaming or processing incrementally for large datasets
    - **Use Specific Searches**: When searching, use form type filters to reduce result size and improve relevance
    - **Check Data Availability**: Not all companies have complete XBRL data. Always check for `None` values

## Next Steps

- [Ticker API](../ticker.md) - Yahoo Finance data for real-time quotes and charts
- [Finance Module](../finance.md) - Market-wide data (screeners, trending, news)
- [Configuration](../configuration.md) - Network and timeout settings
