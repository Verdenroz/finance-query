//! Current Massive futures aggregate-bar endpoint.

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::{FinanceError, Result};

use super::super::{build_client, client::PolygonClient, models::PaginatedResponseDTO};

/// One futures OHLCV aggregate bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesAggregateDTO {
    /// Close price.
    pub close: Option<f64>,
    /// Dollar volume within the window.
    pub dollar_volume: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Open price.
    pub open: Option<f64>,
    /// Trading-session end date.
    pub session_end_date: Option<String>,
    /// Official settlement price, when available.
    pub settlement_price: Option<f64>,
    /// Contract ticker.
    pub ticker: Option<String>,
    /// Number of transactions.
    pub transactions: Option<u64>,
    /// Contract volume.
    pub volume: Option<u64>,
    /// Aggregate-window start timestamp in Unix nanoseconds.
    pub window_start: Option<i64>,
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
    fetch_aggregates(&build_client()?, ticker, resolution, params).await
}

async fn fetch_aggregates(
    client: &PolygonClient,
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
    let path = format!("/futures/v1/aggs/{}", encode_path_segment(ticker));
    let mut query = vec![("resolution", resolution)];
    query.extend_from_slice(params);
    client.get(&path, &query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn futures_aggregates_builds_the_path_and_merges_params() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/futures/v1/aggs/ES%20Z6")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("resolution".into(), "1session".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "1".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"status":"OK","results":[{"close":6052.0,"high":6060.0,"low":6000.0,"open":6010.0,"session_end_date":"2026-08-06","ticker":"ES Z6","transactions":42,"volume":1000,"window_start":1785974400000000000}]}"#)
            .create_async()
            .await;
        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let response = fetch_aggregates(&client, "ES Z6", "1session", &[("limit", "1")])
            .await
            .unwrap();
        let bar = &response.results.unwrap()[0];
        assert_eq!(bar.ticker.as_deref(), Some("ES Z6"));
        assert_eq!(bar.close, Some(6052.0));
        assert_eq!(bar.volume, Some(1000));
        assert_eq!(bar.transactions, Some(42));
        assert_eq!(bar.window_start, Some(1_785_974_400_000_000_000));
    }

    /// Intraday bars omit `settlement_price`, and a partial bar must not fail
    /// the whole page.
    #[tokio::test]
    async fn futures_aggregates_tolerates_a_sparse_bar() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/futures/v1/aggs/ESZ6")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"{"status":"OK","results":[{"close":6052.0,"volume":1000}]}"#)
            .create_async()
            .await;
        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let response = fetch_aggregates(&client, "ESZ6", "1session", &[])
            .await
            .unwrap();
        let bar = &response.results.unwrap()[0];
        assert_eq!(bar.volume, Some(1000));
        assert!(bar.ticker.is_none());
        assert!(bar.settlement_price.is_none());
    }

    #[tokio::test]
    async fn futures_aggregates_rejects_a_blank_resolution() {
        let server = mockito::Server::new_async().await;
        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let error = fetch_aggregates(&client, "ESZ6", "  ", &[])
            .await
            .unwrap_err();
        assert!(
            matches!(error, FinanceError::InvalidParameter { ref param, .. } if param == "resolution"),
            "got {error:?}"
        );
    }
}
