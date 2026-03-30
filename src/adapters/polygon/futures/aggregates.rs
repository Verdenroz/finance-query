//! Futures aggregate bar endpoints: OHLCV bars.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch aggregate bars (OHLCV) for a futures ticker over a date range.
///
/// # Arguments
///
/// * `ticker` - Futures ticker symbol (e.g., `"ESZ4"`, `"CLF5"`)
/// * `multiplier` - Size of the timespan multiplier (e.g., `1`, `5`, `15`)
/// * `timespan` - Timespan unit (e.g., `Timespan::Day`)
/// * `from` - Start date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `to` - End date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `params` - Optional parameters (adjusted, sort, limit)
pub async fn futures_aggregates(
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
        field: "futures_aggregates".to_string(),
        context: format!("Failed to parse futures aggregate response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_futures_aggregates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/aggs/ticker/ESZ4/range/1/day/2024-01-01/2024-01-31",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "ESZ4",
                    "status": "OK",
                    "adjusted": true,
                    "queryCount": 1,
                    "resultsCount": 2,
                    "request_id": "abc123",
                    "results": [
                        { "o": 4750.0, "h": 4780.0, "l": 4740.0, "c": 4770.0, "v": 1500000.0, "vw": 4760.0, "t": 1704067200000_i64, "n": 800000 },
                        { "o": 4770.0, "h": 4800.0, "l": 4760.0, "c": 4790.0, "v": 1400000.0, "vw": 4780.0, "t": 1704153600000_i64, "n": 750000 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/aggs/ticker/ESZ4/range/1/day/2024-01-01/2024-01-31",
                &[],
            )
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("ESZ4"));
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 2);
        assert!((results[0].open - 4750.0).abs() < 0.01);
        assert!((results[1].close - 4790.0).abs() < 0.01);
    }
}
