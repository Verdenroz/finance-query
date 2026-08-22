//! FMP company information endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::discovery::reference::SymbolMatch;

// ============================================================================
// Response types
// ============================================================================

/// Stock peer from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockPeersDTO {
    /// Ticker symbol of the peer.
    pub symbol: Option<String>,
    /// Peer company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
}

/// Delisted company from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DelistedCompanyDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// IPO date.
    #[serde(rename = "ipoDate")]
    pub ipo_date: Option<String>,
    /// Delisted date.
    #[serde(rename = "delistedDate")]
    pub delisted_date: Option<String>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Convert stock peers DTOs into canonical SimilarSymbol items.
fn stock_peers_to_canonical(
    peers: Vec<StockPeersDTO>,
    limit: usize,
) -> Vec<crate::models::corporate::recommendation::SimilarSymbol> {
    let mut symbols: Vec<crate::models::corporate::recommendation::SimilarSymbol> = peers
        .into_iter()
        .filter_map(|p| p.symbol)
        .map(
            |s| crate::models::corporate::recommendation::SimilarSymbol {
                symbol: s,
                score: 0.0,
            },
        )
        .collect();
    symbols.truncate(limit);
    symbols
}

/// Fetch canonical similar symbols for a ticker.
pub async fn fetch_canonical_similar_symbols(
    symbol: &str,
    limit: u32,
) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
    let peers = stock_peers(symbol).await?;
    Ok(stock_peers_to_canonical(peers, limit as usize))
}

/// Fetch stock peers for a symbol.
pub async fn stock_peers(symbol: &str) -> Result<Vec<StockPeersDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    client
        .get("/stable/stock-peers", &[("symbol", symbol)])
        .await
}

/// Fetch delisted companies.
pub async fn delisted_companies(limit: Option<u32>) -> Result<Vec<DelistedCompanyDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    let limit_str = limit.unwrap_or(100).to_string();
    client
        .get(
            "/stable/delisted-companies",
            &[("page", "0"), ("limit", &limit_str)],
        )
        .await
}

/// Convert a delisted-company record into a canonical [`SymbolMatch`],
/// dropping entries without a symbol.
fn to_delisted_symbol_match(dto: DelistedCompanyDTO) -> Option<SymbolMatch> {
    Some(SymbolMatch {
        symbol: dto.symbol?,
        id: None,
        name: dto.company_name,
        exchange: dto.exchange,
        asset_type: None,
        currency: None,
        active: Some(false),
        market_cap_rank: None,
        thumbnail: None,
        image: None,
    })
}

/// Fetch canonical delisted-security listing status
/// (`DiscoveryProvider::fetch_listing_status(active: false)`).
pub async fn fetch_delisted_listing_status_response() -> Result<Vec<SymbolMatch>> {
    Ok(delisted_companies(None)
        .await?
        .into_iter()
        .filter_map(to_delisted_symbol_match)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_delisted_company_to_symbol_match() {
        let dto: DelistedCompanyDTO = serde_json::from_value(serde_json::json!({
            "symbol": "XYZ",
            "companyName": "XYZ Corp",
            "exchange": "NYSE",
            "ipoDate": "2001-01-01",
            "delistedDate": "2023-06-01"
        }))
        .unwrap();

        let out = to_delisted_symbol_match(dto).unwrap();
        assert_eq!(out.symbol, "XYZ");
        assert_eq!(out.name.as_deref(), Some("XYZ Corp"));
        assert_eq!(out.exchange.as_deref(), Some("NYSE"));
        assert_eq!(out.active, Some(false));
    }

    #[test]
    fn drops_delisted_entries_without_a_symbol() {
        let dto: DelistedCompanyDTO = serde_json::from_value(serde_json::json!({
            "companyName": "No Symbol Inc"
        }))
        .unwrap();
        assert!(to_delisted_symbol_match(dto).is_none());
    }

    #[tokio::test]
    async fn test_fmp_rate_limit_returns_rate_limited_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(429)
            .with_body("{}")
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::RateLimited { .. })
        ));
    }

    #[tokio::test]
    async fn test_fmp_401_returns_authentication_failed() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(401)
            .with_body("{}")
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn test_fmp_body_api_key_error_returns_authentication_failed() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"{"Error Message":"Invalid API KEY."}"#)
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn test_fmp_500_returns_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(500)
            .with_body("{}")
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/api/v3/profile/AAPL", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::ServerError { .. })
        ));
    }
}
