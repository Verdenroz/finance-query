//! ETF and mutual fund listing endpoints for Financial Modeling Prep.

use crate::error::Result;
use crate::models::discovery::reference::SymbolMatch;

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::crypto::AvailableSymbolDTO;

/// List all available ETFs.
pub async fn etf_available() -> Result<Vec<AvailableSymbolDTO>> {
    let client = build_client()?;
    client.get("/stable/etf-list", &[]).await
}

/// List all available mutual funds. FMP has not published a `/stable`
/// replacement for this endpoint.
pub async fn mutual_fund_available() -> Result<Vec<AvailableSymbolDTO>> {
    let client = build_client()?;
    client
        .get("/api/v3/symbol/available-mutual-funds", &[])
        .await
}

/// Convert a listed-fund entry into a canonical [`SymbolMatch`], tagging its
/// `asset_type`. Drops entries without a symbol.
fn to_symbol_match(dto: AvailableSymbolDTO, asset_type: &str) -> Option<SymbolMatch> {
    Some(SymbolMatch {
        symbol: dto.symbol?,
        id: None,
        name: dto.name,
        exchange: dto.exchange_short_name.or(dto.stock_exchange),
        asset_type: Some(asset_type.to_string()),
        currency: dto.currency,
        active: Some(true),
        market_cap_rank: None,
        thumbnail: None,
        image: None,
    })
}

/// Fetch canonical active listing status (every available ETF and mutual
/// fund) for `DiscoveryProvider::fetch_listing_status(active: true)`.
pub async fn fetch_active_listing_status_response() -> Result<Vec<SymbolMatch>> {
    let (etfs, funds) = tokio::try_join!(etf_available(), mutual_fund_available())?;
    let mut out: Vec<SymbolMatch> = etfs
        .into_iter()
        .filter_map(|d| to_symbol_match(d, "ETF"))
        .collect();
    out.extend(
        funds
            .into_iter()
            .filter_map(|d| to_symbol_match(d, "MUTUAL_FUND")),
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_available_fund_to_symbol_match() {
        let dto: AvailableSymbolDTO = serde_json::from_value(serde_json::json!({
            "symbol": "SPY",
            "name": "SPDR S&P 500 ETF Trust",
            "currency": "USD",
            "stockExchange": "NYSE Arca",
            "exchangeShortName": "AMEX"
        }))
        .unwrap();

        let out = to_symbol_match(dto, "ETF").unwrap();
        assert_eq!(out.symbol, "SPY");
        assert_eq!(out.asset_type.as_deref(), Some("ETF"));
        assert_eq!(out.exchange.as_deref(), Some("AMEX"));
        assert_eq!(out.active, Some(true));
    }

    #[test]
    fn drops_available_fund_entries_without_a_symbol() {
        let dto: AvailableSymbolDTO = serde_json::from_value(serde_json::json!({
            "name": "No Symbol Fund"
        }))
        .unwrap();
        assert!(to_symbol_match(dto, "ETF").is_none());
    }

    #[tokio::test]
    async fn test_etf_available_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/etf-list")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "SPY",
                        "name": "SPDR S&P 500 ETF Trust",
                        "currency": "USD",
                        "stockExchange": "NYSE Arca",
                        "exchangeShortName": "AMEX"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbolDTO> = client.get("/stable/etf-list", &[]).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("SPY"));
    }
}
