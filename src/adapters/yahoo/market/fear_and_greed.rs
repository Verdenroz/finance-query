//! Alternative.me Fear & Greed Index endpoint.
//!
//! No authentication required. Returns the current market sentiment index
//! (Alternative.me's index specifically reflects crypto/Bitcoin sentiment).

use std::time::Duration;

use serde::Deserialize;

use crate::error::{FinanceError, Result};
use finance_query_models::sentiment::{FearAndGreed, FearAndGreedFieldsInit, FearGreedLabel};

const BASE_URL: &str = "https://api.alternative.me/fng";

/// Fetch the current Fear & Greed Index from Alternative.me.
pub(crate) async fn fetch() -> Result<FearAndGreed> {
    let raw = fetch_raw(1, BASE_URL).await?;
    parse_current(raw)
}

/// Fetch `limit` historical entries (newest first) of the Fear & Greed Index
/// from Alternative.me. Alternative.me's index specifically reflects crypto
/// (Bitcoin) market sentiment.
pub(crate) async fn fetch_history(limit: u32) -> Result<Vec<FearAndGreed>> {
    let raw = fetch_raw(limit, BASE_URL).await?;
    parse_history(raw)
}

#[derive(Debug, Deserialize)]
struct FearAndGreedApiResponse {
    data: Vec<FearAndGreedEntry>,
}

#[derive(Debug, Deserialize)]
struct FearAndGreedEntry {
    value: String,
    value_classification: String,
    timestamp: String,
}

fn parse_current(resp: FearAndGreedApiResponse) -> Result<FearAndGreed> {
    let entry =
        resp.data
            .into_iter()
            .next()
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "data".to_string(),
                context: "Alternative.me API returned empty data array".to_string(),
            })?;

    parse_entry(entry)
}

/// Parse every entry in the response (current + historical), newest
/// first — matching the API's own ordering. Used for `limit > 1` requests.
/// Errors when `data` is empty or the first entry (the current reading) fails
/// to parse; unparseable historical entries are skipped.
fn parse_history(resp: FearAndGreedApiResponse) -> Result<Vec<FearAndGreed>> {
    if resp.data.is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "data".to_string(),
            context: "Alternative.me API returned empty data array".to_string(),
        });
    }

    let mut entries = Vec::with_capacity(resp.data.len());
    for (index, entry) in resp.data.into_iter().enumerate() {
        match parse_entry(entry) {
            Ok(parsed) => entries.push(parsed),
            Err(e) if index == 0 => return Err(e),
            Err(e) => {
                tracing::warn!(index, error = %e, "skipping unparseable fear and greed entry");
            }
        }
    }
    Ok(entries)
}

fn parse_entry(entry: FearAndGreedEntry) -> Result<FearAndGreed> {
    let value = entry
        .value
        .parse::<u8>()
        .map_err(|_| FinanceError::ResponseStructureError {
            field: "value".to_string(),
            context: format!("Cannot parse '{}' as u8", entry.value),
        })?;

    let classification = parse_classification(&entry.value_classification)?;

    let timestamp =
        entry
            .timestamp
            .parse::<i64>()
            .map_err(|_| FinanceError::ResponseStructureError {
                field: "timestamp".to_string(),
                context: format!("Cannot parse '{}' as i64", entry.timestamp),
            })?;

    Ok(FearAndGreed::from(FearAndGreedFieldsInit {
        value,
        classification,
        timestamp,
    }))
}

fn parse_classification(s: &str) -> Result<FearGreedLabel> {
    match s {
        "Extreme Fear" => Ok(FearGreedLabel::ExtremeFear),
        "Fear" => Ok(FearGreedLabel::Fear),
        "Neutral" => Ok(FearGreedLabel::Neutral),
        "Greed" => Ok(FearGreedLabel::Greed),
        "Extreme Greed" => Ok(FearGreedLabel::ExtremeGreed),
        other => Err(FinanceError::ResponseStructureError {
            field: "value_classification".to_string(),
            context: format!("Unknown classification '{other}'"),
        }),
    }
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

    #[test]
    fn test_parse_classification() {
        assert_eq!(
            parse_classification("Extreme Fear").unwrap(),
            FearGreedLabel::ExtremeFear
        );
        assert_eq!(parse_classification("Fear").unwrap(), FearGreedLabel::Fear);
        assert_eq!(
            parse_classification("Neutral").unwrap(),
            FearGreedLabel::Neutral
        );
        assert_eq!(
            parse_classification("Greed").unwrap(),
            FearGreedLabel::Greed
        );
        assert_eq!(
            parse_classification("Extreme Greed").unwrap(),
            FearGreedLabel::ExtremeGreed
        );
        assert!(parse_classification("unknown").is_err());
    }

    #[test]
    fn test_fear_greed_from_response() {
        let resp = FearAndGreedApiResponse {
            data: vec![FearAndGreedEntry {
                value: "25".to_string(),
                value_classification: "Fear".to_string(),
                timestamp: "1700000000".to_string(),
            }],
        };
        let fg = parse_current(resp).unwrap();
        assert_eq!(fg.value, 25);
        assert_eq!(fg.classification, FearGreedLabel::Fear);
        assert_eq!(fg.timestamp, 1700000000);
    }

    #[test]
    fn test_empty_data_returns_error() {
        let resp = FearAndGreedApiResponse { data: vec![] };
        assert!(parse_current(resp).is_err());
    }

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
        let fg = parse_current(raw).unwrap();
        assert_eq!(fg.value, 20);
        assert_eq!(
            fg.classification,
            finance_query_models::sentiment::FearGreedLabel::ExtremeFear
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
        let history = parse_history(raw).unwrap();

        assert_eq!(history.len(), 3);
        assert_eq!(history[0].value, 72);
        assert_eq!(history[1].value, 55);
        assert_eq!(history[2].value, 20);
        assert!(history[0].timestamp > history[1].timestamp);
        assert!(history[1].timestamp > history[2].timestamp);
    }

    fn history_entry(value: &str, timestamp: &str) -> FearAndGreedEntry {
        FearAndGreedEntry {
            value: value.to_string(),
            value_classification: "Fear".to_string(),
            timestamp: timestamp.to_string(),
        }
    }

    #[test]
    fn parse_history_skips_bad_historical_entry() {
        let resp = FearAndGreedApiResponse {
            data: vec![
                history_entry("30", "1700000200"),
                history_entry("20", "1700000100"),
                history_entry("bad", "1700000000"),
            ],
        };

        let history = parse_history(resp).unwrap();
        let values: Vec<u8> = history.iter().map(|fg| fg.value).collect();
        assert_eq!(values, [30, 20]);
    }

    #[test]
    fn parse_history_errors_when_current_entry_is_bad() {
        let resp = FearAndGreedApiResponse {
            data: vec![
                history_entry("bad", "1700000200"),
                history_entry("20", "1700000100"),
            ],
        };
        assert!(parse_history(resp).is_err());
    }

    #[test]
    fn parse_history_errors_on_empty_data() {
        let resp = FearAndGreedApiResponse { data: vec![] };
        assert!(parse_history(resp).is_err());
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
