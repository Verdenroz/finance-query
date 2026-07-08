//! Options snapshot endpoints: options chain, single contract snapshot.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::Provider;
use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::options::OptionContract;
use crate::models::options::Options;
use crate::providers::build_options;

use super::super::build_client;
use super::super::models::*;

/// Greeks for an options contract snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsGreeksDTO {
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
pub struct OptionsSnapshotDetailsDTO {
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
pub struct OptionsUnderlyingAssetDTO {
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
pub struct OptionsSnapshotQuoteDTO {
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
pub struct OptionsSnapshotTradeDTO {
    /// Conditions.
    pub conditions: Option<Vec<i32>>,
    /// Exchange ID.
    pub exchange: Option<i32>,
    /// TradeDTO price.
    pub price: Option<f64>,
    /// SIP timestamp (nanoseconds).
    pub sip_timestamp: Option<i64>,
    /// TradeDTO size.
    pub size: Option<f64>,
    /// Timeframe of the trade data.
    pub timeframe: Option<String>,
}

/// A single options contract snapshot from the chain or individual lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsSnapshotDTO {
    /// Break-even price for the contract.
    pub break_even_price: Option<f64>,
    /// Current day aggregate data.
    pub day: Option<SnapshotAggDTO>,
    /// Contract details (strike, expiration, type).
    pub details: Option<OptionsSnapshotDetailsDTO>,
    /// Option greeks (delta, gamma, theta, vega).
    pub greeks: Option<OptionsGreeksDTO>,
    /// Implied volatility.
    pub implied_volatility: Option<f64>,
    /// Last quote for this contract.
    pub last_quote: Option<OptionsSnapshotQuoteDTO>,
    /// Last trade for this contract.
    pub last_trade: Option<OptionsSnapshotTradeDTO>,
    /// Open interest.
    pub open_interest: Option<u64>,
    /// Underlying asset data.
    pub underlying_asset: Option<OptionsUnderlyingAssetDTO>,
}

/// Response wrapper for a single options contract snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsContractSnapshotResponseDTO {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// The snapshot result.
    pub results: Option<OptionsSnapshotDTO>,
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
) -> Result<PaginatedResponseDTO<OptionsSnapshotDTO>> {
    let client = build_client()?;
    let path = format!("/v3/snapshot/options/{}", encode_path_segment(underlying));
    client.get(&path, params).await
}

/// Helper: parse a "YYYY-MM-DD" date string into a Unix timestamp.
fn parse_date(d: &Option<String>) -> i64 {
    d.as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .and_then(|dt| dt.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0)
}

/// Helper: convert a Unix timestamp to "YYYY-MM-DD" date string.
fn timestamp_to_date(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

/// Map a single options snapshot DTO to a canonical OptionContract.
fn map_snapshot_to_contract(s: &OptionsSnapshotDTO) -> OptionContract {
    let details = s.details.as_ref();
    let day = s.day.as_ref();
    let last_trade = s.last_trade.as_ref();
    let last_quote = s.last_quote.as_ref();
    OptionContract {
        contract_symbol: details.and_then(|d| d.ticker.clone()).unwrap_or_default(),
        strike: details.and_then(|d| d.strike_price).unwrap_or(0.0),
        currency: None,
        last_price: last_trade
            .and_then(|t| t.price)
            .or_else(|| day.and_then(|d| d.close)),
        change: last_trade
            .and_then(|t| t.price)
            .zip(day.and_then(|d| d.open))
            .map(|(price, open)| price - open),
        percent_change: None,
        volume: day.and_then(|d| d.volume).map(|v| v as i64),
        open_interest: s.open_interest.map(|v| v as i64),
        bid: last_quote.and_then(|q| q.bid),
        ask: last_quote.and_then(|q| q.ask),
        contract_size: None,
        expiration: details
            .as_ref()
            .and_then(|d| d.expiration_date.as_ref())
            .map(|s| Some(parse_date(&Some(s.clone()))))
            .unwrap_or(None),
        last_trade_date: None,
        implied_volatility: s.implied_volatility,
        in_the_money: None,
    }
}

/// Fetch options chain (canonical) for a stock ticker.
pub async fn fetch_options_response(symbol: &str, date: Option<i64>) -> Result<Options> {
    let date_str_opt = date.map(timestamp_to_date);
    let mut params: Vec<(&str, &str)> = vec![("limit", "1000")];
    if let Some(ref ds) = date_str_opt {
        params.push(("expiration_date", ds.as_str()));
    }

    let paginated = options_chain_snapshot(symbol, &params).await?;
    let snapshots = paginated.results.unwrap_or_default();

    // Collect unique expiration dates (sorted).
    let expiration_dates: Vec<i64> = snapshots
        .iter()
        .filter_map(|s| {
            let ts = s
                .details
                .as_ref()
                .and_then(|d| d.expiration_date.as_ref())
                .map(|s| parse_date(&Some(s.clone())))
                .unwrap_or(0);
            if ts > 0 { Some(ts) } else { None }
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    let calls: Vec<OptionContract> = snapshots
        .iter()
        .filter(|s| s.details.as_ref().and_then(|d| d.contract_type.as_deref()) == Some("call"))
        .map(map_snapshot_to_contract)
        .collect();

    let puts: Vec<OptionContract> = snapshots
        .iter()
        .filter(|s| s.details.as_ref().and_then(|d| d.contract_type.as_deref()) == Some("put"))
        .map(map_snapshot_to_contract)
        .collect();

    Ok(build_options(
        symbol.to_string(),
        Provider::Polygon,
        expiration_dates,
        calls,
        puts,
    ))
}

/// Fetch a snapshot for a single options contract.
///
/// * `underlying` - Underlying stock ticker (e.g., `"AAPL"`)
/// * `contract` - Options contract ticker (e.g., `"O:AAPL250117C00150000"`)
pub async fn options_contract_snapshot(
    underlying: &str,
    contract: &str,
) -> Result<OptionsContractSnapshotResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v3/snapshot/options/{}/{}",
        encode_path_segment(underlying),
        encode_path_segment(contract)
    );
    client
        .get_as(
            &path,
            &[],
            "options_contract_snapshot",
            "options contract snapshot response",
        )
        .await
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
        let resp: PaginatedResponseDTO<OptionsSnapshotDTO> =
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

        let resp: OptionsContractSnapshotResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let snap = resp.results.unwrap();
        assert!((snap.break_even_price.unwrap() - 155.30).abs() < 0.01);
        assert_eq!(snap.open_interest, Some(25000));

        let greeks = snap.greeks.unwrap();
        assert!((greeks.vega.unwrap() - 0.25).abs() < 0.01);
    }
}
