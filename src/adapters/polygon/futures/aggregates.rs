//! Current Massive futures aggregate-bar endpoint.

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::{FinanceError, Result};

use super::super::{build_client, models::PaginatedResponseDTO};

/// One futures OHLCV aggregate bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesAggregateDTO {
    /// Close price.
    pub close: f64,
    /// Dollar volume within the window.
    pub dollar_volume: Option<f64>,
    /// High price.
    pub high: f64,
    /// Low price.
    pub low: f64,
    /// Open price.
    pub open: f64,
    /// Trading-session end date.
    pub session_end_date: String,
    /// Official settlement price, when available.
    pub settlement_price: Option<f64>,
    /// Contract ticker.
    pub ticker: String,
    /// Number of transactions.
    pub transactions: u64,
    /// Contract volume.
    pub volume: f64,
    /// Aggregate-window start timestamp in Unix nanoseconds.
    pub window_start: i64,
}

/// Fetch futures OHLCV bars from the current `/futures/v1/aggs` API.
///
/// `resolution` is a Massive resolution such as `1min`, `1hour`,
/// `1session`, `1week`, `1month`, `1quarter`, or `1year`. Use `params` for
/// `window_start` comparisons, `limit`, and `sort`.
pub async fn futures_aggregates(
    ticker: &str,
    resolution: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesAggregateDTO>> {
    if resolution.trim().is_empty() {
        return Err(FinanceError::InvalidParameter {
            param: "resolution".to_string(),
            reason: "resolution must not be empty".to_string(),
        });
    }
    let client = build_client()?;
    let path = format!("/futures/v1/aggs/{}", encode_path_segment(ticker));
    let mut query = vec![("resolution", resolution)];
    query.extend_from_slice(params);
    client.get(&path, &query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn current_futures_aggregate_shape_deserializes() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/futures/v1/aggs/ESZ6")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("resolution".into(), "1session".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "1".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"status":"OK","results":[{"close":6052.0,"high":6060.0,"low":6000.0,"open":6010.0,"session_end_date":"2026-08-06","ticker":"ESZ6","transactions":42,"volume":1000,"window_start":1785974400000000000}]}"#)
            .create_async()
            .await;
        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let response: PaginatedResponseDTO<FuturesAggregateDTO> = client
            .get(
                "/futures/v1/aggs/ESZ6",
                &[("resolution", "1session"), ("limit", "1")],
            )
            .await
            .unwrap();
        assert_eq!(response.results.unwrap()[0].ticker, "ESZ6");
    }
}
