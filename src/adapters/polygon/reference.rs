//! Reference data endpoints: tickers, exchanges, conditions, market holidays, market status.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::PaginatedResponse;

/// Ticker reference entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerRef {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Security name.
    pub name: Option<String>,
    /// Market (e.g., `"stocks"`, `"crypto"`, `"fx"`).
    pub market: Option<String>,
    /// Locale (e.g., `"us"`).
    pub locale: Option<String>,
    /// Primary exchange.
    pub primary_exchange: Option<String>,
    /// Asset type.
    #[serde(rename = "type")]
    pub asset_type: Option<String>,
    /// Whether the ticker is active.
    pub active: Option<bool>,
    /// Currency name.
    pub currency_name: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Composite FIGI.
    pub composite_figi: Option<String>,
    /// Share class FIGI.
    pub share_class_figi: Option<String>,
    /// Last updated date.
    pub last_updated_utc: Option<String>,
}

/// Detailed ticker overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerDetails {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Market.
    pub market: Option<String>,
    /// Locale.
    pub locale: Option<String>,
    /// Primary exchange.
    pub primary_exchange: Option<String>,
    /// Asset type.
    #[serde(rename = "type")]
    pub asset_type: Option<String>,
    /// Active.
    pub active: Option<bool>,
    /// Currency.
    pub currency_name: Option<String>,
    /// CIK.
    pub cik: Option<String>,
    /// SIC code.
    pub sic_code: Option<String>,
    /// SIC description.
    pub sic_description: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Homepage URL.
    pub homepage_url: Option<String>,
    /// Total employees.
    pub total_employees: Option<u64>,
    /// Market cap.
    pub market_cap: Option<f64>,
    /// Phone number.
    pub phone_number: Option<String>,
    /// Address.
    pub address: Option<serde_json::Value>,
    /// Branding (logo, icon).
    pub branding: Option<serde_json::Value>,
    /// List date.
    pub list_date: Option<String>,
    /// Share class shares outstanding.
    pub share_class_shares_outstanding: Option<f64>,
    /// Weighted shares outstanding.
    pub weighted_shares_outstanding: Option<f64>,
}

/// Ticker details response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerDetailsResponse {
    /// Request ID.
    pub request_id: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Ticker details.
    pub results: Option<TickerDetails>,
}

/// Ticker type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerType {
    /// Type code.
    pub code: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Locale.
    pub locale: Option<String>,
}

/// Related ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RelatedTicker {
    /// Ticker symbol.
    pub ticker: Option<String>,
}

/// Exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Exchange {
    /// Exchange ID.
    pub id: Option<i64>,
    /// Exchange type.
    #[serde(rename = "type")]
    pub exchange_type: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Locale.
    pub locale: Option<String>,
    /// Exchange name.
    pub name: Option<String>,
    /// MIC code.
    pub mic: Option<String>,
    /// Operating MIC.
    pub operating_mic: Option<String>,
    /// Participant ID.
    pub participant_id: Option<String>,
    /// URL.
    pub url: Option<String>,
}

/// Condition code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Condition {
    /// Condition ID.
    pub id: Option<i32>,
    /// Condition type.
    #[serde(rename = "type")]
    pub condition_type: Option<String>,
    /// Name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// SIP mapping.
    pub sip_mapping: Option<serde_json::Value>,
    /// Data types.
    pub data_types: Option<Vec<String>>,
    /// Legacy flag.
    pub legacy: Option<bool>,
}

/// Market holiday.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketHoliday {
    /// Holiday name.
    pub name: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Exchange (e.g., `"NYSE"`, `"NASDAQ"`).
    pub exchange: Option<String>,
    /// Status (e.g., `"closed"`, `"early-close"`).
    pub status: Option<String>,
    /// Open time (if early close).
    pub open: Option<String>,
    /// Close time (if early close).
    pub close: Option<String>,
}

/// Market status for exchanges.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketStatusResponse {
    /// After hours trading.
    #[serde(rename = "afterHours")]
    pub after_hours: Option<bool>,
    /// Early hours trading.
    #[serde(rename = "earlyHours")]
    pub early_hours: Option<bool>,
    /// Market status (e.g., `"open"`, `"closed"`).
    pub market: Option<String>,
    /// Server time.
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
    /// Individual exchange statuses.
    pub exchanges: Option<serde_json::Value>,
    /// Individual currency statuses.
    pub currencies: Option<serde_json::Value>,
}

/// Fetch all tickers.
///
/// * `params` - Query params: `type`, `market`, `exchange`, `cusip`, `cik`, `active`, `sort`, `order`, `limit`, `search`
pub async fn all_tickers(params: &[(&str, &str)]) -> Result<PaginatedResponse<TickerRef>> {
    let client = build_client()?;
    client.get("/v3/reference/tickers", params).await
}

/// Fetch detailed ticker information.
pub async fn ticker_details(ticker: &str) -> Result<TickerDetailsResponse> {
    let client = build_client()?;
    let path = format!("/v3/reference/tickers/{}", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "ticker_details".to_string(),
        context: format!("Failed to parse ticker details: {e}"),
    })
}

/// Fetch ticker types.
pub async fn ticker_types(params: &[(&str, &str)]) -> Result<PaginatedResponse<TickerType>> {
    let client = build_client()?;
    client.get("/v3/reference/tickers/types", params).await
}

/// Fetch related tickers.
pub async fn related_tickers(ticker: &str) -> Result<PaginatedResponse<RelatedTicker>> {
    let client = build_client()?;
    let path = format!("/v1/related-companies/{}", ticker);
    client.get(&path, &[]).await
}

/// Fetch exchanges list.
pub async fn exchanges(params: &[(&str, &str)]) -> Result<PaginatedResponse<Exchange>> {
    let client = build_client()?;
    client.get("/v3/reference/exchanges", params).await
}

/// Fetch condition codes.
pub async fn condition_codes(params: &[(&str, &str)]) -> Result<PaginatedResponse<Condition>> {
    let client = build_client()?;
    client.get("/v3/reference/conditions", params).await
}

/// Fetch upcoming market holidays.
pub async fn market_holidays() -> Result<Vec<MarketHoliday>> {
    let client = build_client()?;
    let json = client.get_raw("/v1/marketstatus/upcoming", &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "market_holidays".to_string(),
        context: format!("Failed to parse market holidays: {e}"),
    })
}

/// Fetch current market status.
pub async fn market_status() -> Result<MarketStatusResponse> {
    let client = build_client()?;
    let json = client.get_raw("/v1/marketstatus/now", &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "market_status".to_string(),
        context: format!("Failed to parse market status: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ticker_details_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/reference/tickers/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(serde_json::json!({
                "request_id": "abc",
                "status": "OK",
                "results": {
                    "ticker": "AAPL",
                    "name": "Apple Inc.",
                    "market": "stocks",
                    "locale": "us",
                    "primary_exchange": "XNAS",
                    "type": "CS",
                    "active": true,
                    "currency_name": "usd",
                    "market_cap": 2850000000000.0,
                    "description": "Apple Inc. designs, manufactures, and markets smartphones..."
                }
            }).to_string())
            .create_async().await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client.get_raw("/v3/reference/tickers/AAPL", &[]).await.unwrap();
        let resp: TickerDetailsResponse = serde_json::from_value(json).unwrap();
        let details = resp.results.unwrap();
        assert_eq!(details.name.as_deref(), Some("Apple Inc."));
        assert_eq!(details.ticker.as_deref(), Some("AAPL"));
        assert!((details.market_cap.unwrap() - 2850000000000.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn test_market_status_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/marketstatus/now")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(serde_json::json!({
                "market": "open",
                "earlyHours": false,
                "afterHours": false,
                "serverTime": "2024-01-15T12:00:00-05:00"
            }).to_string())
            .create_async().await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client.get_raw("/v1/marketstatus/now", &[]).await.unwrap();
        let resp: MarketStatusResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.market.as_deref(), Some("open"));
    }
}
