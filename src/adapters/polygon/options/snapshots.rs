//! Options snapshot endpoints: options chain, single contract snapshot.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Greeks for an options contract snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsGreeks {
    /// Delta: rate of change of the option price with respect to the underlying.
    pub delta: Option<f64>,
    /// Gamma: rate of change of delta with respect to the underlying.
    pub gamma: Option<f64>,
    /// Theta: rate of change of the option price with respect to time.
    pub theta: Option<f64>,
    /// Vega: rate of change of the option price with respect to volatility.
    pub vega: Option<f64>,
}

/// Contract details within an options snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsSnapshotDetails {
    /// Contract type: `"call"` or `"put"`.
    pub contract_type: Option<String>,
    /// Exercise style: `"american"` or `"european"`.
    pub exercise_style: Option<String>,
    /// Expiration date (`"YYYY-MM-DD"`).
    pub expiration_date: Option<String>,
    /// Number of shares per contract.
    pub shares_per_contract: Option<u32>,
    /// Strike price.
    pub strike_price: Option<f64>,
    /// Options ticker symbol.
    pub ticker: Option<String>,
}

/// Underlying asset data within an options snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsUnderlyingAsset {
    /// Change in price since previous close.
    pub change_to_break_even: Option<f64>,
    /// Last updated timestamp (nanoseconds).
    pub last_updated: Option<i64>,
    /// Current price of the underlying.
    pub price: Option<f64>,
    /// Underlying ticker symbol.
    pub ticker: Option<String>,
    /// Timeframe of the underlying data.
    pub timeframe: Option<String>,
}

/// Last quote data within an options snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsSnapshotQuote {
    /// Ask price.
    pub ask: Option<f64>,
    /// Ask size.
    pub ask_size: Option<f64>,
    /// Bid price.
    pub bid: Option<f64>,
    /// Bid size.
    pub bid_size: Option<f64>,
    /// Last updated timestamp (nanoseconds).
    pub last_updated: Option<i64>,
    /// Midpoint price.
    pub midpoint: Option<f64>,
    /// Timeframe of the quote data.
    pub timeframe: Option<String>,
}

/// Last trade data within an options snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsSnapshotTrade {
    /// Conditions.
    pub conditions: Option<Vec<i32>>,
    /// Exchange ID.
    pub exchange: Option<i32>,
    /// Trade price.
    pub price: Option<f64>,
    /// SIP timestamp (nanoseconds).
    pub sip_timestamp: Option<i64>,
    /// Trade size.
    pub size: Option<f64>,
    /// Timeframe of the trade data.
    pub timeframe: Option<String>,
}

/// A single options contract snapshot from the chain or individual lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsSnapshot {
    /// Break-even price for the contract.
    pub break_even_price: Option<f64>,
    /// Current day aggregate data.
    pub day: Option<SnapshotAgg>,
    /// Contract details (strike, expiration, type).
    pub details: Option<OptionsSnapshotDetails>,
    /// Option greeks (delta, gamma, theta, vega).
    pub greeks: Option<OptionsGreeks>,
    /// Implied volatility.
    pub implied_volatility: Option<f64>,
    /// Last quote for this contract.
    pub last_quote: Option<OptionsSnapshotQuote>,
    /// Last trade for this contract.
    pub last_trade: Option<OptionsSnapshotTrade>,
    /// Open interest.
    pub open_interest: Option<u64>,
    /// Underlying asset data.
    pub underlying_asset: Option<OptionsUnderlyingAsset>,
}

/// Response wrapper for a single options contract snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsContractSnapshotResponse {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// The snapshot result.
    pub results: Option<OptionsSnapshot>,
}

/// Fetch the options chain snapshot for an underlying ticker.
///
/// Returns a paginated list of options contract snapshots.
///
/// # Arguments
///
/// * `underlying` - Underlying stock ticker (e.g., `"AAPL"`)
/// * `params` - Query params such as `strike_price`, `expiration_date`,
///   `contract_type`, `order`, `limit`, `sort`
pub async fn options_chain_snapshot(
    underlying: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<OptionsSnapshot>> {
    let client = build_client()?;
    let path = format!("/v3/snapshot/options/{}", underlying);
    client.get(&path, params).await
}

/// Fetch a snapshot for a single options contract.
///
/// * `underlying` - Underlying stock ticker (e.g., `"AAPL"`)
/// * `contract` - Options contract ticker (e.g., `"O:AAPL250117C00150000"`)
pub async fn options_contract_snapshot(
    underlying: &str,
    contract: &str,
) -> Result<OptionsContractSnapshotResponse> {
    let client = build_client()?;
    let path = format!("/v3/snapshot/options/{}/{}", underlying, contract);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "options_contract_snapshot".to_string(),
        context: format!("Failed to parse options contract snapshot response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_options_chain_snapshot_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/snapshot/options/AAPL")
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
                    "results": [
                        {
                            "break_even_price": 155.30,
                            "day": { "o": 5.10, "h": 5.50, "l": 4.90, "c": 5.30, "v": 1200.0 },
                            "details": {
                                "contract_type": "call",
                                "exercise_style": "american",
                                "expiration_date": "2025-01-17",
                                "shares_per_contract": 100,
                                "strike_price": 150.0,
                                "ticker": "O:AAPL250117C00150000"
                            },
                            "greeks": {
                                "delta": 0.65,
                                "gamma": 0.03,
                                "theta": -0.05,
                                "vega": 0.25
                            },
                            "implied_volatility": 0.32,
                            "last_quote": {
                                "ask": 5.40,
                                "ask_size": 10.0,
                                "bid": 5.20,
                                "bid_size": 15.0,
                                "last_updated": 1705363200000000000_i64,
                                "midpoint": 5.30
                            },
                            "last_trade": {
                                "price": 5.30,
                                "size": 5.0,
                                "exchange": 4,
                                "sip_timestamp": 1705363200000000000_i64
                            },
                            "open_interest": 25000,
                            "underlying_asset": {
                                "change_to_break_even": 5.30,
                                "last_updated": 1705363200000000000_i64,
                                "price": 150.00,
                                "ticker": "AAPL",
                                "timeframe": "2024-01-15"
                            }
                        }
                    ],
                    "resultsCount": 1
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<OptionsSnapshot> =
            client.get("/v3/snapshot/options/AAPL", &[]).await.unwrap();

        let results = resp.results.unwrap();
        assert_eq!(results.len(), 1);
        assert!((results[0].break_even_price.unwrap() - 155.30).abs() < 0.01);
        assert!((results[0].implied_volatility.unwrap() - 0.32).abs() < 0.01);

        let greeks = results[0].greeks.as_ref().unwrap();
        assert!((greeks.delta.unwrap() - 0.65).abs() < 0.01);
        assert!((greeks.theta.unwrap() - (-0.05)).abs() < 0.01);

        let details = results[0].details.as_ref().unwrap();
        assert_eq!(details.contract_type.as_deref(), Some("call"));
        assert!((details.strike_price.unwrap() - 150.0).abs() < 0.01);

        assert_eq!(results[0].open_interest, Some(25000));
    }

    #[tokio::test]
    async fn test_options_contract_snapshot_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/snapshot/options/AAPL/O:AAPL250117C00150000")
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
                        "break_even_price": 155.30,
                        "day": { "o": 5.10, "h": 5.50, "l": 4.90, "c": 5.30, "v": 1200.0 },
                        "details": {
                            "contract_type": "call",
                            "expiration_date": "2025-01-17",
                            "strike_price": 150.0,
                            "ticker": "O:AAPL250117C00150000"
                        },
                        "greeks": {
                            "delta": 0.65,
                            "gamma": 0.03,
                            "theta": -0.05,
                            "vega": 0.25
                        },
                        "implied_volatility": 0.32,
                        "open_interest": 25000,
                        "underlying_asset": {
                            "price": 150.00,
                            "ticker": "AAPL"
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v3/snapshot/options/AAPL/O:AAPL250117C00150000", &[])
            .await
            .unwrap();

        let resp: OptionsContractSnapshotResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let snap = resp.results.unwrap();
        assert!((snap.break_even_price.unwrap() - 155.30).abs() < 0.01);
        assert_eq!(snap.open_interest, Some(25000));

        let greeks = snap.greeks.unwrap();
        assert!((greeks.vega.unwrap() - 0.25).abs() < 0.01);
    }
}
