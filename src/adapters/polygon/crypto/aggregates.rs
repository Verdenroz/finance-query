//! Crypto aggregate bar endpoints: OHLCV bars, previous close, grouped daily, daily open/close.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Daily open/close response for a crypto pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CryptoDailyOpenClose {
    /// The "from" symbol of the pair (e.g., `BTC`).
    pub symbol: Option<String>,
    /// Whether the response is adjusted.
    #[serde(rename = "isUTC")]
    pub is_utc: Option<bool>,
    /// Day of the data.
    pub day: Option<String>,
    /// Open price.
    pub open: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// Open trades.
    #[serde(rename = "openTrades")]
    pub open_trades: Option<Vec<CryptoOpenCloseTrade>>,
    /// Close trades.
    #[serde(rename = "closingTrades")]
    pub closing_trades: Option<Vec<CryptoOpenCloseTrade>>,
}

/// A single trade within a crypto daily open/close response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CryptoOpenCloseTrade {
    /// Price of the trade.
    #[serde(rename = "p")]
    pub price: Option<f64>,
    /// Size of the trade.
    #[serde(rename = "s")]
    pub size: Option<f64>,
    /// Exchange.
    #[serde(rename = "x")]
    pub exchange: Option<i32>,
    /// Conditions.
    pub conditions: Option<Vec<i32>>,
    /// Timestamp.
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
}

/// Fetch aggregate bars (OHLCV) for a crypto ticker over a date range.
///
/// # Arguments
///
/// * `ticker` - Crypto ticker symbol with `X:` prefix (e.g., `"X:BTCUSD"`)
/// * `multiplier` - Size of the timespan multiplier (e.g., `1`, `5`, `15`)
/// * `timespan` - Timespan unit (e.g., `Timespan::Day`)
/// * `from` - Start date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `to` - End date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `params` - Optional parameters (adjusted, sort, limit)
pub async fn crypto_aggregates(
    ticker: &str,
    multiplier: u32,
    timespan: Timespan,
    from: &str,
    to: &str,
    params: Option<AggregateParams>,
) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!(
        "/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
        ticker,
        multiplier,
        timespan.as_str(),
        from,
        to
    );

    let mut query_params: Vec<(&str, String)> = Vec::new();
    if let Some(ref p) = params {
        if let Some(adjusted) = p.adjusted {
            query_params.push(("adjusted", adjusted.to_string()));
        }
        if let Some(sort) = p.sort {
            query_params.push(("sort", sort.as_str().to_string()));
        }
        if let Some(limit) = p.limit {
            query_params.push(("limit", limit.to_string()));
        }
    }

    let query_refs: Vec<(&str, &str)> = query_params
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    let json = client.get_raw(&path, &query_refs).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_aggregates".to_string(),
        context: format!("Failed to parse crypto aggregate response: {e}"),
    })
}

/// Fetch the previous day's OHLCV bar for a crypto ticker.
///
/// * `ticker` - Crypto ticker symbol with `X:` prefix (e.g., `"X:BTCUSD"`)
/// * `adjusted` - Whether results are adjusted (default: true)
pub async fn crypto_previous_close(
    ticker: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!("/v2/aggs/ticker/{}/prev", ticker);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_previous_close".to_string(),
        context: format!("Failed to parse crypto previous close response: {e}"),
    })
}

/// Fetch grouped daily bars for the entire crypto market on a given date.
///
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted (default: true)
pub async fn crypto_grouped_daily(
    date: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!("/v2/aggs/grouped/locale/global/market/crypto/{}", date);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_grouped_daily".to_string(),
        context: format!("Failed to parse crypto grouped daily response: {e}"),
    })
}

/// Fetch daily open/close for a crypto pair on a specific date.
///
/// * `from` - The "from" symbol of the pair (e.g., `"BTC"`)
/// * `to` - The "to" symbol of the pair (e.g., `"USD"`)
/// * `date` - Date as `"YYYY-MM-DD"`
pub async fn crypto_daily_open_close(
    from: &str,
    to: &str,
    date: &str,
) -> Result<CryptoDailyOpenClose> {
    let client = build_client()?;
    let path = format!("/v1/open-close/crypto/{}/{}/{}", from, to, date);

    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_daily_open_close".to_string(),
        context: format!("Failed to parse crypto daily open/close response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_aggregates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/aggs/ticker/X:BTCUSD/range/1/day/2024-01-01/2024-01-31",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "X:BTCUSD",
                    "status": "OK",
                    "adjusted": true,
                    "queryCount": 1,
                    "resultsCount": 2,
                    "request_id": "abc123",
                    "results": [
                        { "o": 42000.0, "h": 43500.0, "l": 41800.0, "c": 43100.0, "v": 12345.67, "vw": 42750.0, "t": 1704067200000_i64, "n": 150000 },
                        { "o": 43100.0, "h": 44000.0, "l": 42900.0, "c": 43800.0, "v": 11000.50, "vw": 43400.0, "t": 1704153600000_i64, "n": 140000 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/aggs/ticker/X:BTCUSD/range/1/day/2024-01-01/2024-01-31",
                &[],
            )
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("X:BTCUSD"));
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 2);
        assert!((results[0].open - 42000.0).abs() < 0.01);
        assert!((results[0].close - 43100.0).abs() < 0.01);
        assert_eq!(results[0].timestamp, 1704067200000);
    }
}
