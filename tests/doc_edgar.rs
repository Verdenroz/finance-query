//! Compile and runtime tests for docs/library/edgar.md
//!
//! Run compile tests:
//!   cargo test --test doc_edgar
//! Run network tests:
//!   cargo test --test doc_edgar -- --ignored
//!
//! Known doc/source discrepancies corrected in tests:
//!   - edgar::search takes 6 params (query, forms, start_date, end_date, from, size);
//!     docs show only 4 (missing `from` and `size`).
//!   - EdgarSearchSource::display_names is Vec<String>, not Option<Vec<String>>;
//!     docs incorrectly show `if let Some(names) = &source.display_names`.
//!   - Complete example's `hits.total.value` skips two Option layers;
//!     corrected to use Option chaining.

use std::time::Duration;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Returns true for any connection-level error that can occur when tests run
/// concurrently: each `#[tokio::test]` gets its own runtime, so connections
/// pooled in an earlier runtime become stale (hyper `DispatchGone`) once that
/// runtime drops, or the remote host may simply be unreachable.
fn is_stale_connection(e: &reqwest::Error) -> bool {
    e.is_connect() || e.is_timeout() || format!("{e:?}").contains("DispatchGone")
}

/// Unwrap a `Result`, returning early (skipping the test) on stale-connection
/// errors. Panics on any other error so real failures still surface.
macro_rules! unwrap_or_skip {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(finance_query::FinanceError::HttpError(ref e)) if is_stale_connection(e) => {
                eprintln!("test skipped: stale connection ({})", e);
                return;
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    };
}

// ---------------------------------------------------------------------------
// Compile-time — initialization API (edgar.md "Initialization" section)
// ---------------------------------------------------------------------------

#[test]
fn test_init_compiles() {
    use finance_query::edgar;

    // From edgar.md "Basic Initialization" section
    // Returns Err if already initialized — safe to ignore in tests.
    let _ = edgar::init("user@example.com");
}

#[test]
fn test_init_with_config_compiles() {
    use finance_query::edgar;

    // From edgar.md "Advanced Configuration" section
    // Returns Err if already initialized — safe to ignore in tests.
    let _ = edgar::init_with_config(
        "user@example.com",
        "my-financial-app",      // Optional: default is "finance-query"
        Duration::from_secs(60), // Optional: default is 30 seconds
    );
}

// ---------------------------------------------------------------------------
// Network tests — Rate Limiting (edgar.md "Rate Limiting" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_resolve_cik_rate_limiting() {
    use finance_query::edgar;

    let _ = edgar::init("user@example.com");

    // From edgar.md "Rate Limiting" section
    // These requests are automatically rate-limited
    let cik1 = unwrap_or_skip!(edgar::resolve_cik("AAPL").await);
    let cik2 = unwrap_or_skip!(edgar::resolve_cik("MSFT").await);
    let cik3 = unwrap_or_skip!(edgar::resolve_cik("GOOGL").await);
    // Executed at max 10 req/sec automatically

    assert!(cik1 > 0);
    assert!(cik2 > 0);
    assert!(cik3 > 0);
}

// ---------------------------------------------------------------------------
// Network tests — CIK Resolution (edgar.md "Ticker to CIK Resolution" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_resolve_cik() {
    use finance_query::edgar;

    let _ = edgar::init("user@example.com");

    // From edgar.md "Ticker to CIK Resolution" section

    // Resolve ticker to CIK (cached after first fetch)
    let cik = unwrap_or_skip!(edgar::resolve_cik("AAPL").await);
    println!("Apple CIK: {}", cik); // Output: Apple CIK: 320193
    assert_eq!(cik, 320193);

    // Subsequent lookups use the cache (no network request)
    let cik2 = unwrap_or_skip!(edgar::resolve_cik("AAPL").await); // Instant
    assert_eq!(cik, cik2);

    // Case-insensitive lookup
    let cik3 = unwrap_or_skip!(edgar::resolve_cik("aapl").await); // Also works
    assert_eq!(cik, cik3);
}

// ---------------------------------------------------------------------------
// Network tests — Filing History (edgar.md "Filing History" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_submissions_filing_history() {
    use finance_query::edgar;

    let _ = edgar::init("user@example.com");

    // From edgar.md "Filing History (Submissions)" section

    // Get CIK first
    let cik = unwrap_or_skip!(edgar::resolve_cik("AAPL").await);

    // Fetch filing history
    let submissions = unwrap_or_skip!(edgar::submissions(cik).await);

    // Company information
    if let Some(name) = &submissions.name {
        println!("Company: {}", name);
    }
    if let Some(cik_str) = &submissions.cik {
        println!("CIK: {}", cik_str);
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
        assert!(!recent.accession_number.is_empty());
    }

    assert!(submissions.name.is_some());
}

// ---------------------------------------------------------------------------
// Network tests — Company Facts (edgar.md "Company Facts (XBRL Data)" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_company_facts_xbrl() {
    use finance_query::edgar;

    let _ = edgar::init("user@example.com");

    // From edgar.md "Company Facts (XBRL Data)" section

    // Get CIK
    let cik = unwrap_or_skip!(edgar::resolve_cik("AAPL").await);

    // Fetch company facts
    let facts = unwrap_or_skip!(edgar::company_facts(cik).await);

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

    assert!(!facts.facts.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Basic Search (edgar.md "Full-Text Search" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_search_basic() {
    use finance_query::edgar;

    let _ = edgar::init("user@example.com");

    // From edgar.md "Full-Text Search" section
    //
    // Note: edgar::search takes 6 params (query, forms, start_date, end_date, from, size).
    // Doc shows 4 params — `from` and `size` are missing from the docs.
    let results = unwrap_or_skip!(
        edgar::search(
            "artificial intelligence",
            None, // No form filter
            None, // No start date
            None, // No end date
            None, // from (offset)
            None, // size (limit)
        )
        .await
    );

    // Display results
    if let Some(hits) = &results.hits {
        if let Some(total) = &hits.total
            && let Some(value) = total.value
        {
            println!("Total hits: {}", value);
        }

        for hit in &hits.hits {
            if let Some(source) = &hit._source {
                let form = source.form.as_deref().unwrap_or("Unknown");
                let file_date = source.file_date.as_deref().unwrap_or("Unknown");

                println!("{} filed on {}", form, file_date);

                // Note: display_names is Vec<String>, not Option<Vec<String>>.
                // Doc shows `if let Some(names)` which won't compile; corrected here:
                if !source.display_names.is_empty() {
                    println!("  Companies: {:?}", source.display_names);
                }
            }
        }
    }

    assert!(results.hits.is_some());
}

// ---------------------------------------------------------------------------
// Network tests — Filtered Search (edgar.md "Filtered Search" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_search_filtered() {
    use finance_query::edgar;

    let _ = edgar::init("user@example.com");

    // From edgar.md "Filtered Search" section
    // Search for 10-K filings only
    let results = unwrap_or_skip!(
        edgar::search(
            "machine learning",
            Some(&["10-K"]),    // Only 10-K forms
            Some("2024-01-01"), // From Jan 1, 2024
            Some("2024-12-31"), // To Dec 31, 2024
            None,
            None,
        )
        .await
    );

    assert!(results.hits.is_some());
}

// ---------------------------------------------------------------------------
// Network tests — Common Form Filters (edgar.md "Common Form Filters" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_search_common_form_filters() {
    use finance_query::edgar;

    let _ = edgar::init("user@example.com");

    // From edgar.md "Common Form Filters" section

    // Annual reports
    unwrap_or_skip!(edgar::search("query", Some(&["10-K"]), None, None, None, None).await);

    // Quarterly reports
    unwrap_or_skip!(edgar::search("query", Some(&["10-Q"]), None, None, None, None).await);

    // Current events
    unwrap_or_skip!(edgar::search("query", Some(&["8-K"]), None, None, None, None).await);

    // Multiple form types
    unwrap_or_skip!(
        edgar::search(
            "query",
            Some(&["10-K", "10-Q", "8-K"]),
            None,
            None,
            None,
            None
        )
        .await
    );
}

// ---------------------------------------------------------------------------
// Network tests — Complete Example (edgar.md "Complete Example" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_complete_example() {
    use finance_query::edgar;

    // From edgar.md "Complete Example" section
    let ticker = "AAPL";

    // Create EDGAR client
    let _ = edgar::init("user@example.com");

    // Step 1: Resolve ticker to CIK
    println!("Resolving {} to CIK...", ticker);
    let cik = unwrap_or_skip!(edgar::resolve_cik(ticker).await);
    println!("CIK: {}\n", cik);

    // Step 2: Get filing history
    println!("Fetching filing history...");
    let submissions = unwrap_or_skip!(edgar::submissions(cik).await);
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
    let facts = unwrap_or_skip!(edgar::company_facts(cik).await);

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
    let search_results = unwrap_or_skip!(
        edgar::search(
            "artificial intelligence",
            Some(&["10-K", "10-Q"]),
            Some("2024-01-01"),
            None,
            None,
            None,
        )
        .await
    );
    // Note: doc shows `hits.total.value` directly, but total is Option<EdgarSearchTotal>
    // and value is Option<u64>. Corrected to use Option chaining:
    if let Some(hits) = &search_results.hits {
        let count = hits.total.as_ref().and_then(|t| t.value).unwrap_or(0);
        println!("Found {} mentions", count);
    }

    assert!(cik > 0);
    assert!(facts.facts.contains_key("us-gaap"));
}
