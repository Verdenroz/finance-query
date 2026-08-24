//! DISCOVERY capability: SEC's active-listing universe.
//!
//! `company_tickers.json` has no delisted-securities history, so this only
//! ever answers `active = true` queries.

use crate::adapters::edgar::build_client;
use crate::adapters::edgar::client::CompanyTickerEntry;
use crate::error::Result;
use crate::models::discovery::reference::SymbolMatch;
use crate::providers::{Operation, Provider};

fn to_symbol_match(entry: CompanyTickerEntry, exchange: Option<&String>) -> SymbolMatch {
    SymbolMatch {
        symbol: entry.ticker,
        id: Some(entry.cik.to_string()),
        name: Some(entry.title),
        exchange: exchange.cloned(),
        asset_type: None,
        currency: None,
        active: Some(true),
        market_cap_rank: None,
        thumbnail: None,
        image: None,
    }
}

/// Fetch canonical listing status. `active = false` is not supported — SEC's
/// bulk ticker files carry no delisted-securities history.
pub async fn fetch_listing_status_response(active: bool) -> Result<Vec<SymbolMatch>> {
    if !active {
        return Err(Operation::ListingStatus.not_supported(Provider::Edgar));
    }

    let client = build_client()?;
    let (tickers, exchanges) =
        tokio::try_join!(client.company_tickers(), client.company_tickers_exchange())?;

    Ok(tickers
        .into_iter()
        .map(|entry| {
            let exchange = exchanges.get(&entry.ticker);
            to_symbol_match(entry, exchange)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_a_ticker_row_with_its_exchange() {
        let entry = CompanyTickerEntry {
            cik: 320193,
            ticker: "AAPL".to_string(),
            title: "Apple Inc.".to_string(),
        };
        let exchange = "Nasdaq".to_string();

        let out = to_symbol_match(entry, Some(&exchange));
        assert_eq!(out.symbol, "AAPL");
        assert_eq!(out.id.as_deref(), Some("320193"));
        assert_eq!(out.name.as_deref(), Some("Apple Inc."));
        assert_eq!(out.exchange.as_deref(), Some("Nasdaq"));
        assert_eq!(out.active, Some(true));
    }

    #[test]
    fn missing_exchange_row_leaves_exchange_none() {
        let entry = CompanyTickerEntry {
            cik: 1,
            ticker: "ZZZ".to_string(),
            title: "Unmatched Inc.".to_string(),
        };

        let out = to_symbol_match(entry, None);
        assert_eq!(out.exchange, None);
    }

    #[tokio::test]
    async fn delisted_query_is_not_supported() {
        let err = fetch_listing_status_response(false).await.unwrap_err();
        assert!(matches!(
            err,
            crate::error::FinanceError::NotSupported { .. }
        ));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_the_full_listing_from_live_edgar() {
        let _ = crate::adapters::edgar::init("test@example.com");
        let out = fetch_listing_status_response(true).await.unwrap();
        assert!(out.len() > 5000);
        let aapl = out.iter().find(|s| s.symbol == "AAPL").unwrap();
        assert_eq!(aapl.name.as_deref(), Some("Apple Inc."));
        assert_eq!(aapl.exchange.as_deref(), Some("Nasdaq"));
    }
}
