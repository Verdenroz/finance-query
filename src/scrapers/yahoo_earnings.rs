//! Yahoo Finance earnings calls scraper.
//!
//! Scrapes the quote page to get a list of available earnings call transcripts,
//! extracting event IDs needed to fetch full transcripts.

use crate::error::{FinanceError, Result};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;
use tracing::info;

/// Matches earnings call URLs for any symbol; capture groups 1 and 2 hold the
/// symbol as it appears in the `/quote/{symbol}` and `{symbol}-Q...` segments
/// respectively. The `regex` crate has no backreferences, so both are captured
/// and compared against the requested symbol in Rust instead of in the pattern.
/// Both symbol positions match any run of non-`/` characters rather than a
/// fixed class, so the pattern accepts every symbol the old
/// `regex::escape(symbol)` form did — including `^GSPC` or `ES=F` — and the
/// exact-match filter below, not the pattern, decides what is kept.
static EARNINGS_CALL_URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?:https://finance\.yahoo\.com)?/quote/([^/]+)/earnings/([^/]+?)-([Qq]\d)-(\d{4})-earnings_call-(\d+)\.html"#,
    )
    .unwrap()
});

/// Represents an earnings call with its identifiers and metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsCall {
    /// Event ID needed for fetching the transcript
    pub event_id: String,
    /// Fiscal quarter (e.g., "Q1", "Q2", "Q3", "Q4")
    pub quarter: Option<String>,
    /// Fiscal year
    pub year: Option<i32>,
    /// Title of the earnings call
    pub title: String,
    /// Full URL to the earnings call page
    pub url: String,
}

impl EarningsCall {
    fn new(
        event_id: String,
        quarter: Option<String>,
        year: Option<i32>,
        title: String,
        url: String,
    ) -> Self {
        Self {
            event_id,
            quarter,
            year,
            title,
            url,
        }
    }
}

/// Scrape the quote page for a symbol to get list of available transcripts.
///
/// # Arguments
///
/// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
///
/// # Returns
///
/// A list of `EarningsCall` objects containing event IDs and metadata needed
/// to fetch full transcripts via the `earnings_transcript` endpoint.
pub(crate) async fn scrape_earnings_calls(symbol: &str) -> Result<Vec<EarningsCall>> {
    let symbol_upper = symbol.to_uppercase();
    let url = format!("https://finance.yahoo.com/quote/{}", symbol_upper);

    info!("Fetching earnings calls from quote page for {}", symbol);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let response = client.get(&url).send().await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(FinanceError::SymbolNotFound {
            symbol: Some(symbol.to_string()),
            context: "Quote page not found".to_string(),
        });
    }

    if !response.status().is_success() {
        return Err(FinanceError::ServerError {
            status: response.status().as_u16(),
            context: format!("Failed to fetch quote page for {}", symbol),
        });
    }

    let html = response.text().await?;
    parse_earnings_calls(&html, &symbol_upper)
}

/// Parse earnings calls from HTML content.
///
/// The earnings call URLs are embedded in the page (often in JSON or inline data)
/// with the pattern: /quote/{SYMBOL}/earnings/{SYMBOL}-Q{N}-{YEAR}-earnings_call-{ID}.html
fn parse_earnings_calls(html: &str, symbol: &str) -> Result<Vec<EarningsCall>> {
    // Match earnings call URLs in the page content
    // Format: /quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-218053.html
    // or full URLs: https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-218053.html
    let mut calls = Vec::new();
    let mut seen_event_ids = HashSet::new();

    for caps in EARNINGS_CALL_URL_PATTERN.captures_iter(html) {
        // Case-sensitive, same as the old regex::escape(symbol)-embedded pattern.
        if &caps[1] != symbol || &caps[2] != symbol {
            continue;
        }

        let quarter = caps.get(3).map(|m| m.as_str().to_uppercase());
        let year = caps.get(4).and_then(|m| m.as_str().parse::<i32>().ok());
        let event_id = caps[5].to_string();

        // Skip duplicates
        if seen_event_ids.contains(&event_id) {
            continue;
        }
        seen_event_ids.insert(event_id.clone());

        // Build title
        let title = match (&quarter, year) {
            (Some(q), Some(y)) => format!("{} {} Earnings Call", q, y),
            _ => "Earnings Call".to_string(),
        };

        // Build full URL
        let full_url = format!(
            "https://finance.yahoo.com/quote/{}/earnings/{}-{}-{}-earnings_call-{}.html",
            symbol,
            symbol,
            quarter.as_deref().unwrap_or(""),
            year.map(|y| y.to_string()).unwrap_or_default(),
            event_id
        );

        calls.push(EarningsCall::new(event_id, quarter, year, title, full_url));
    }

    if calls.is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "earnings_calls".to_string(),
            context: format!("No earnings calls found for {}", symbol),
        });
    }

    // Sort by year (descending) then quarter (descending: Q4 > Q3 > Q2 > Q1)
    calls.sort_by(|a, b| b.year.cmp(&a.year).then_with(|| b.quarter.cmp(&a.quarter)));

    Ok(calls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_earnings_calls_from_json_like_content() {
        // Simulates how Yahoo embeds earnings call URLs in page content
        let html = r#"
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-218053.html","title":"AAPL Q4 2024"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q3-2024-earnings_call-190138.html","title":"AAPL Q3 2024"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q2-2024-earnings_call-123456.html","title":"AAPL Q2 2024"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q1-2024-earnings_call-111111.html","title":"AAPL Q1 2024"}
        "#;

        let calls = parse_earnings_calls(html, "AAPL").unwrap();
        assert_eq!(calls.len(), 4);

        // Should be sorted by year desc, quarter desc
        assert_eq!(calls[0].event_id, "218053");
        assert_eq!(calls[0].quarter, Some("Q4".to_string()));
        assert_eq!(calls[0].year, Some(2024));

        assert_eq!(calls[1].event_id, "190138");
        assert_eq!(calls[1].quarter, Some("Q3".to_string()));

        assert_eq!(calls[3].event_id, "111111");
        assert_eq!(calls[3].quarter, Some("Q1".to_string()));
    }

    #[test]
    fn test_parse_earnings_calls_relative_urls() {
        let html = r#"
            href="/quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-12345.html"
            href="/quote/AAPL/earnings/AAPL-Q3-2024-earnings_call-12344.html"
        "#;

        let calls = parse_earnings_calls(html, "AAPL").unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].event_id, "12345");
        assert_eq!(calls[1].event_id, "12344");
    }

    #[test]
    fn test_parse_earnings_calls_filters_other_symbols() {
        // The shared LazyLock pattern is symbol-agnostic, so it must correctly
        // exclude URLs belonging to a different symbol found in the same page.
        let html = r#"
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-1.html"}
            {"url":"https://finance.yahoo.com/quote/MSFT/earnings/MSFT-Q4-2024-earnings_call-2.html"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q3-2024-earnings_call-3.html"}
            {"url":"https://finance.yahoo.com/quote/aapl/earnings/aapl-Q2-2024-earnings_call-4.html"}
        "#;

        let calls = parse_earnings_calls(html, "AAPL").unwrap();
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.event_id != "2" && c.event_id != "4"));
        assert_eq!(calls[0].event_id, "1");
        assert_eq!(calls[1].event_id, "3");

        let msft_calls = parse_earnings_calls(html, "MSFT").unwrap();
        assert_eq!(msft_calls.len(), 1);
        assert_eq!(msft_calls[0].event_id, "2");
    }

    #[test]
    fn test_parse_earnings_calls_accepts_punctuated_symbols() {
        // The old pattern interpolated `regex::escape(symbol)`, so it matched any
        // literal symbol. These would have been dropped by a fixed
        // `[A-Za-z0-9.\-]+` class; the current pattern must still find them.
        let html = r#"
            {"url":"https://finance.yahoo.com/quote/BRK-B/earnings/BRK-B-Q4-2024-earnings_call-10.html"}
            {"url":"https://finance.yahoo.com/quote/^GSPC/earnings/^GSPC-Q4-2024-earnings_call-11.html"}
            {"url":"https://finance.yahoo.com/quote/ES=F/earnings/ES=F-Q1-2025-earnings_call-12.html"}
            {"url":"https://finance.yahoo.com/quote/7203.T/earnings/7203.T-Q2-2025-earnings_call-13.html"}
        "#;

        for (symbol, event_id) in [
            ("BRK-B", "10"),
            ("^GSPC", "11"),
            ("ES=F", "12"),
            ("7203.T", "13"),
        ] {
            let calls = parse_earnings_calls(html, symbol).unwrap();
            assert_eq!(calls.len(), 1, "symbol {symbol} was not matched");
            assert_eq!(calls[0].event_id, event_id);
        }

        // Widening the class must not weaken the exact-match filter. A prefix of
        // a real symbol matches nothing, which this function reports as an error
        // rather than an empty vec.
        assert!(parse_earnings_calls(html, "BRK").is_err());
        assert!(parse_earnings_calls(html, "GSPC").is_err());
    }

    #[test]
    fn test_parse_earnings_calls_deduplication() {
        let html = r#"
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-12345.html"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-12345.html"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q3-2024-earnings_call-12344.html"}
        "#;

        let calls = parse_earnings_calls(html, "AAPL").unwrap();
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn test_parse_earnings_calls_no_matches() {
        let html = r#"
            <html>
            <body>
                <a href="/some/other/link">Other</a>
            </body>
            </html>
        "#;

        let result = parse_earnings_calls(html, "AAPL");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_earnings_calls_multiple_years() {
        let html = r#"
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q4-2024-earnings_call-1.html"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q1-2024-earnings_call-2.html"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q4-2023-earnings_call-3.html"}
            {"url":"https://finance.yahoo.com/quote/AAPL/earnings/AAPL-Q1-2023-earnings_call-4.html"}
        "#;

        let calls = parse_earnings_calls(html, "AAPL").unwrap();
        assert_eq!(calls.len(), 4);

        // Should be sorted: Q4 2024, Q1 2024, Q4 2023, Q1 2023
        assert_eq!(calls[0].year, Some(2024));
        assert_eq!(calls[0].quarter, Some("Q4".to_string()));

        assert_eq!(calls[1].year, Some(2024));
        assert_eq!(calls[1].quarter, Some("Q1".to_string()));

        assert_eq!(calls[2].year, Some(2023));
        assert_eq!(calls[2].quarter, Some("Q4".to_string()));

        assert_eq!(calls[3].year, Some(2023));
        assert_eq!(calls[3].quarter, Some("Q1".to_string()));
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_scrape_earnings_calls() {
        let calls = scrape_earnings_calls("AAPL").await;
        assert!(calls.is_ok(), "Failed: {:?}", calls.err());
        let list = calls.unwrap();
        assert!(!list.is_empty());
        println!("Found {} earnings calls for AAPL", list.len());
        for call in list.iter().take(5) {
            println!("  {:?}", call);
        }
    }
}
