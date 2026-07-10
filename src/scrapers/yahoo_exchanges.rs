//! Yahoo Finance exchanges scraper.
//!
//! Scrapes the Yahoo Finance help page to get a list of supported exchanges
//! with their suffixes and data providers.

use crate::error::{FinanceError, Result};
use crate::models::market::exchanges::Exchange;
use crate::scrapers::html;
use tracing::info;

const EXCHANGES_URL: &str = "https://help.yahoo.com/kb/finance-for-web/SLN2310.html";

/// Scrape the Yahoo Finance help page for supported exchanges.
///
/// Returns a list of exchanges with their suffixes and data delay information.
pub async fn scrape_exchanges() -> Result<Vec<Exchange>> {
    info!("Fetching exchanges from Yahoo Finance help page");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()?;

    let response = client.get(EXCHANGES_URL).send().await?;

    if !response.status().is_success() {
        return Err(FinanceError::ServerError {
            status: response.status().as_u16(),
            context: "Failed to fetch exchanges page".to_string(),
        });
    }

    let html = response.text().await?;

    parse_exchanges_html(&html)
}

/// Parse the exchanges HTML table.
fn parse_exchanges_html(document: &str) -> Result<Vec<Exchange>> {
    let table = html::find_first(document, "table").ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "table".to_string(),
            context: "No table found in exchanges page".to_string(),
        }
    })?;

    let mut exchanges = Vec::new();

    for row in html::find_all(table.inner, "tr") {
        let cells = html::find_all(row.inner, "td");

        // Skip header rows (they use <th> not <td>)
        if cells.len() != 5 {
            continue;
        }

        let country = cells[0].text().trim().to_string();
        let market = cells[1].text().trim().to_string();
        let suffix = cells[2].text().trim().to_string();
        let delay = cells[3].text().trim().to_string();
        let data_provider = cells[4].text().trim().to_string();

        exchanges.push(Exchange {
            country,
            market,
            suffix,
            delay,
            data_provider,
        });
    }

    if exchanges.is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "exchanges".to_string(),
            context: "No exchanges found in table".to_string(),
        });
    }

    info!("Found {} exchanges", exchanges.len());
    Ok(exchanges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exchanges_table_offline() {
        let html = r#"
            <html><body>
            <table>
              <tr><th>Country</th><th>Market</th><th>Suffix</th><th>Delay</th><th>Provider</th></tr>
              <tr><td>United States of America</td><td>NASDAQ</td><td></td><td>Real-time</td><td>Nasdaq</td></tr>
              <tr><td>Japan</td><td>Tokyo Stock Exchange</td><td>.T</td><td>20 min</td><td>Tokyo</td></tr>
            </table>
            </body></html>
        "#;

        let exchanges = parse_exchanges_html(html).unwrap();
        assert_eq!(exchanges.len(), 2);
        assert_eq!(exchanges[0].country, "United States of America");
        assert_eq!(exchanges[1].suffix, ".T");
    }

    #[test]
    fn errors_when_no_table_present() {
        let result = parse_exchanges_html("<html><body>no table here</body></html>");
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_scrape_exchanges() {
        let result = scrape_exchanges().await;
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let exchanges = result.unwrap();
        assert!(!exchanges.is_empty());

        // Check for some known exchanges
        let has_nyse = exchanges
            .iter()
            .any(|e| e.market.contains("Nasdaq") && e.country == "United States of America");
        assert!(has_nyse, "Should have US Nasdaq");

        let has_tokyo = exchanges
            .iter()
            .any(|e| e.market.contains("Tokyo") && e.suffix == ".T");
        assert!(has_tokyo, "Should have Tokyo Stock Exchange with .T suffix");
    }
}
