//! Futures reference data endpoints: contracts, products, schedules.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::PaginatedResponse;

/// A futures contract from the reference endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesContract {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Name of the contract.
    pub name: Option<String>,
    /// Market for this contract.
    pub market: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Expiration date.
    pub expiration_date: Option<String>,
    /// First trade date.
    pub first_trade_date: Option<String>,
    /// Last trade date.
    pub last_trade_date: Option<String>,
    /// Contract size.
    pub contract_size: Option<f64>,
    /// Contract unit.
    pub contract_unit: Option<String>,
    /// Tick size.
    pub tick_size: Option<f64>,
    /// Underlying ticker.
    pub underlying_ticker: Option<String>,
}

/// A futures product from the reference endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesProduct {
    /// Product ticker.
    pub ticker: Option<String>,
    /// Product name.
    pub name: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Market.
    pub market: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// Contract size.
    pub contract_size: Option<f64>,
    /// Contract unit.
    pub contract_unit: Option<String>,
    /// Tick size.
    pub tick_size: Option<f64>,
}

/// A futures schedule from the reference endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesSchedule {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Session type.
    pub session_type: Option<String>,
    /// Start time.
    pub start_time: Option<String>,
    /// End time.
    pub end_time: Option<String>,
    /// Timezone.
    pub timezone: Option<String>,
    /// Day of week.
    pub day_of_week: Option<String>,
}

/// Fetch futures contracts reference data.
///
/// * `params` - Optional query params: `ticker`, `underlying_ticker`, `expiration_date`, `order`, `limit`, `sort`
pub async fn futures_contracts(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<FuturesContract>> {
    let client = build_client()?;
    let path = "/v3/reference/futures/contracts";
    let json = client.get_raw(path, params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "futures_contracts".to_string(),
        context: format!("Failed to parse futures contracts response: {e}"),
    })
}

/// Fetch futures products reference data.
///
/// * `params` - Optional query params: `ticker`, `asset_class`, `exchange`, `order`, `limit`, `sort`
pub async fn futures_products(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<FuturesProduct>> {
    let client = build_client()?;
    let path = "/v3/reference/futures/products";
    let json = client.get_raw(path, params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "futures_products".to_string(),
        context: format!("Failed to parse futures products response: {e}"),
    })
}

/// Fetch futures schedules reference data.
///
/// * `params` - Optional query params: `ticker`, `session_type`, `order`, `limit`, `sort`
pub async fn futures_schedules(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<FuturesSchedule>> {
    let client = build_client()?;
    let path = "/v3/reference/futures/schedules";
    let json = client.get_raw(path, params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "futures_schedules".to_string(),
        context: format!("Failed to parse futures schedules response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_futures_contracts_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/reference/futures/contracts")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc123",
                    "results": [
                        {
                            "ticker": "ESZ4",
                            "name": "E-mini S&P 500 Dec 2024",
                            "market": "futures",
                            "asset_class": "equity_index",
                            "expiration_date": "2024-12-20",
                            "first_trade_date": "2024-06-21",
                            "contract_size": 50.0,
                            "contract_unit": "USD",
                            "tick_size": 0.25,
                            "underlying_ticker": "I:SPX"
                        }
                    ],
                    "resultsCount": 1
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v3/reference/futures/contracts", &[])
            .await
            .unwrap();

        let resp: PaginatedResponse<FuturesContract> = serde_json::from_value(json).unwrap();
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ticker.as_deref(), Some("ESZ4"));
        assert_eq!(results[0].name.as_deref(), Some("E-mini S&P 500 Dec 2024"));
    }
}
