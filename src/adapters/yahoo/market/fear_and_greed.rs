//! Alternative.me Fear & Greed Index endpoint.
//!
//! No authentication required. Returns the current market sentiment index
//! (Alternative.me's index specifically reflects crypto/Bitcoin sentiment).

use std::time::Duration;

use crate::error::{FinanceError, Result};
use crate::models::sentiment::response::{FearAndGreed, FearAndGreedApiResponse};

const BASE_URL: &str = "https://api.alternative.me/fng";

/// Fetch the current Fear & Greed Index from Alternative.me.
pub(crate) async fn fetch() -> Result<FearAndGreed> {
    let raw = fetch_raw(1, BASE_URL).await?;
    FearAndGreed::from_response(raw)
}

/// Fetch `limit` historical entries (newest first) of the Fear & Greed Index
/// from Alternative.me. Alternative.me's index specifically reflects crypto
/// (Bitcoin) market sentiment.
pub(crate) async fn fetch_history(limit: u32) -> Result<Vec<FearAndGreed>> {
    let raw = fetch_raw(limit, BASE_URL).await?;
    FearAndGreed::vec_from_response(raw)
}

/// `base_url` is injectable so tests can point at a mock server instead of
/// the live API.
async fn fetch_raw(limit: u32, base_url: &str) -> Result<FearAndGreedApiResponse> {
    // Per-call client construction is intentional: this endpoint is called at most once
    // per session, so connection-pool reuse provides no measurable benefit. A static
    // OnceLock<reqwest::Client> would bind the pool to whichever tokio runtime first
    // initialises it; if that runtime later drops (e.g. in tests, or on a server
    // restart with re-init), subsequent calls on a new runtime fail with DispatchGone.
    let url = format!("{base_url}?limit={limit}&format=json");
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?
        .get(url)
        .send()
        .await?;

    let status = response.status().as_u16();
    if !response.status().is_success() {
        return Err(FinanceError::ExternalApiError {
            api: "alternative.me".to_string(),
            status,
        });
    }

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_entry_body(value: &str, classification: &str, timestamp: &str) -> String {
        serde_json::json!({
            "data": [
                {
                    "value": value,
                    "value_classification": classification,
                    "timestamp": timestamp,
                }
            ]
        })
        .to_string()
    }

    fn multi_entry_body() -> String {
        serde_json::json!({
            "data": [
                { "value": "72", "value_classification": "Greed", "timestamp": "1700000200" },
                { "value": "55", "value_classification": "Neutral", "timestamp": "1700000100" },
                { "value": "20", "value_classification": "Extreme Fear", "timestamp": "1700000000" },
            ]
        })
        .to_string()
    }

    #[tokio::test]
    async fn fetch_raw_parses_single_entry() {
        let mut server = mockito::Server::new_async().await;
        let base = format!("{}/fng", server.url());
        let _m = server
            .mock("GET", "/fng")
            .match_query(mockito::Matcher::UrlEncoded("limit".into(), "1".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(single_entry_body("20", "Extreme Fear", "1700000000"))
            .create_async()
            .await;

        let raw = fetch_raw(1, &base).await.unwrap();
        let fg = FearAndGreed::from_response(raw).unwrap();
        assert_eq!(fg.value, 20);
        assert_eq!(
            fg.classification,
            crate::models::sentiment::FearGreedLabel::ExtremeFear
        );
        assert_eq!(fg.timestamp, 1_700_000_000);
    }

    #[tokio::test]
    async fn fetch_raw_parses_multiple_entries_newest_first() {
        let mut server = mockito::Server::new_async().await;
        let base = format!("{}/fng", server.url());
        let _m = server
            .mock("GET", "/fng")
            .match_query(mockito::Matcher::UrlEncoded("limit".into(), "3".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(multi_entry_body())
            .create_async()
            .await;

        let raw = fetch_raw(3, &base).await.unwrap();
        let history = FearAndGreed::vec_from_response(raw).unwrap();

        assert_eq!(history.len(), 3);
        assert_eq!(history[0].value, 72);
        assert_eq!(history[1].value, 55);
        assert_eq!(history[2].value, 20);
        assert!(history[0].timestamp > history[1].timestamp);
        assert!(history[1].timestamp > history[2].timestamp);
    }

    #[tokio::test]
    async fn fetch_raw_maps_http_error_status() {
        let mut server = mockito::Server::new_async().await;
        let base = format!("{}/fng", server.url());
        let _m = server
            .mock("GET", "/fng")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body("{}")
            .create_async()
            .await;

        let err = fetch_raw(1, &base).await.unwrap_err();
        assert!(matches!(
            err,
            FinanceError::ExternalApiError { status: 500, .. }
        ));
    }
}
