//! Yahoo Finance exchanges scraper.
//!
//! Scrapes the Yahoo Finance help page to get a list of supported exchanges
//! with their suffixes and data providers.

use crate::error::{FinanceError, Result};
use crate::models::exchanges::Exchange;
use scraper::{Html, Selector};
use tracing::info;

const EXCHANGES_URL: &str = "https://help.yahoo.com/kb/finance-for-web/SLN2310.html";

/// Scrape the Yahoo Finance help page for supported exchanges.
///
/// Returns a list of exchanges with their suffixes and data delay information.
pub async fn scrape_exchanges() -> Result<Vec<Exchange>> {
    info!("Fetching exchanges from Yahoo Finance help page");

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()
        .map_err(FinanceError::HttpError)?;

    let response = client.get(EXCHANGES_URL).send().await?;
    let html = response.text().await?;

    parse_exchanges_html(&html)
}

/// Parse the exchanges HTML table.
fn parse_exchanges_html(html: &str) -> Result<Vec<Exchange>> {
    let document = Html::parse_document(html);

    let table_selector =
        Selector::parse("table").map_err(|_| FinanceError::ResponseStructureError {
            field: "table".to_string(),
            context: "Failed to parse table selector".to_string(),
        })?;

    let row_selector = Selector::parse("tr").map_err(|_| FinanceError::ResponseStructureError {
        field: "tr".to_string(),
        context: "Failed to parse row selector".to_string(),
    })?;

    let cell_selector =
        Selector::parse("td").map_err(|_| FinanceError::ResponseStructureError {
            field: "td".to_string(),
            context: "Failed to parse cell selector".to_string(),
        })?;

    let table = document.select(&table_selector).next().ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "table".to_string(),
            context: "No table found in exchanges page".to_string(),
        }
    })?;

    let mut exchanges = Vec::new();

    for row in table.select(&row_selector) {
        let cells: Vec<_> = row.select(&cell_selector).collect();

        // Skip header rows (they use <th> not <td>)
        if cells.len() != 5 {
            continue;
        }

        let country = cells[0].text().collect::<String>().trim().to_string();
        let market = cells[1].text().collect::<String>().trim().to_string();
        let suffix = cells[2].text().collect::<String>().trim().to_string();
        let delay = cells[3].text().collect::<String>().trim().to_string();
        let data_provider = cells[4].text().collect::<String>().trim().to_string();

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
