//! Options contract reference endpoints: list contracts, contract details.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// An additional underlying asset for a contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AdditionalUnderlying {
    /// The type of the additional underlying (e.g., `"equity"`, `"index"`).
    #[serde(rename = "type")]
    pub underlying_type: Option<String>,
    /// The underlying ticker or identifier.
    pub underlying: Option<String>,
    /// The number of units of the underlying per contract.
    pub amount: Option<f64>,
}

/// An options contract from the Polygon reference API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsContract {
    /// The options ticker symbol (e.g., `"O:AAPL250117C00150000"`).
    pub ticker: Option<String>,
    /// The underlying stock ticker (e.g., `"AAPL"`).
    pub underlying_ticker: Option<String>,
    /// Contract type: `"call"` or `"put"`.
    pub contract_type: Option<String>,
    /// Exercise style: `"american"` or `"european"`.
    pub exercise_style: Option<String>,
    /// Contract expiration date (`"YYYY-MM-DD"`).
    pub expiration_date: Option<String>,
    /// Strike price.
    pub strike_price: Option<f64>,
    /// CFI code for the contract.
    pub cfi: Option<String>,
    /// Number of shares per contract (typically 100).
    pub shares_per_contract: Option<u32>,
    /// Additional underlying assets, if any.
    pub additional_underlyings: Option<Vec<AdditionalUnderlying>>,
    /// Primary exchange.
    pub primary_exchange: Option<String>,
}

/// Response wrapper for a single options contract detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsContractResponse {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// The contract result.
    pub results: Option<OptionsContract>,
}

/// Fetch a list of options contracts matching the given query parameters.
///
/// # Arguments
///
/// * `params` - Query params such as `underlying_ticker`, `contract_type`,
///   `expiration_date`, `strike_price`, `expired`, `order`, `limit`, `sort`
pub async fn options_contracts(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<OptionsContract>> {
    let client = build_client()?;
    let path = "/v3/reference/options/contracts";
    client.get(path, params).await
}

/// Fetch details for a single options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
pub async fn options_contract_details(ticker: &str) -> Result<OptionsContractResponse> {
    let client = build_client()?;
    let path = format!("/v3/reference/options/contracts/{}", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "options_contract_details".to_string(),
        context: format!("Failed to parse options contract details response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_options_contracts_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/reference/options/contracts")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("underlying_ticker".into(), "AAPL".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "results": [
                        {
                            "ticker": "O:AAPL250117C00150000",
                            "underlying_ticker": "AAPL",
                            "contract_type": "call",
                            "exercise_style": "american",
                            "expiration_date": "2025-01-17",
                            "strike_price": 150.0,
                            "cfi": "OCASPS",
                            "shares_per_contract": 100,
                            "primary_exchange": "BATO"
                        },
                        {
                            "ticker": "O:AAPL250117P00150000",
                            "underlying_ticker": "AAPL",
                            "contract_type": "put",
                            "exercise_style": "american",
                            "expiration_date": "2025-01-17",
                            "strike_price": 150.0,
                            "cfi": "OPASPS",
                            "shares_per_contract": 100,
                            "primary_exchange": "BATO"
                        }
                    ],
                    "resultsCount": 2
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<OptionsContract> = client
            .get(
                "/v3/reference/options/contracts",
                &[("underlying_ticker", "AAPL")],
            )
            .await
            .unwrap();

        let results = resp.results.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].ticker.as_deref(), Some("O:AAPL250117C00150000"));
        assert_eq!(results[0].contract_type.as_deref(), Some("call"));
        assert!((results[0].strike_price.unwrap() - 150.0).abs() < 0.01);
        assert_eq!(results[1].contract_type.as_deref(), Some("put"));
    }

    #[tokio::test]
    async fn test_options_contract_details_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v3/reference/options/contracts/O:AAPL250117C00150000",
            )
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "results": {
                        "ticker": "O:AAPL250117C00150000",
                        "underlying_ticker": "AAPL",
                        "contract_type": "call",
                        "exercise_style": "american",
                        "expiration_date": "2025-01-17",
                        "strike_price": 150.0,
                        "cfi": "OCASPS",
                        "shares_per_contract": 100,
                        "primary_exchange": "BATO",
                        "additional_underlyings": [
                            { "type": "equity", "underlying": "AAPL", "amount": 100.0 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v3/reference/options/contracts/O:AAPL250117C00150000", &[])
            .await
            .unwrap();

        let resp: OptionsContractResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let contract = resp.results.unwrap();
        assert_eq!(contract.ticker.as_deref(), Some("O:AAPL250117C00150000"));
        assert_eq!(contract.exercise_style.as_deref(), Some("american"));
        assert_eq!(contract.shares_per_contract, Some(100));
        let additional = contract.additional_underlyings.unwrap();
        assert_eq!(additional.len(), 1);
        assert_eq!(additional[0].underlying.as_deref(), Some("AAPL"));
    }
}
