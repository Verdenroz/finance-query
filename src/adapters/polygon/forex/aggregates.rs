//! Forex aggregate bar endpoints: grouped daily.

use crate::Provider;
use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::chart::Candle;

use super::super::build_client;
use super::super::models::*;

/// Fetch grouped daily bars for the entire forex market on a given date.
///
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn forex_grouped_daily(
    date: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/aggs/grouped/locale/global/market/fx/{}",
        encode_path_segment(date)
    );

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    client
        .get_as(
            &path,
            &params,
            "forex_grouped_daily",
            "forex grouped daily response",
        )
        .await
}

/// Convert a grouped-daily (all-tickers-for-one-date) response into
/// per-ticker candles. Bars without a `"T"` ticker field are skipped.
fn grouped_daily_to_candles(aggs: AggregateResponseDTO) -> Vec<(String, Candle)> {
    aggs.results
        .into_iter()
        .flatten()
        .filter_map(|r| {
            let ticker = r.ticker.clone()?;
            Some((
                ticker,
                Candle {
                    timestamp: r.timestamp,
                    open: r.open,
                    high: r.high,
                    low: r.low,
                    close: r.close,
                    volume: r.volume as i64,
                    adj_close: None,
                    provider_id: Some(Provider::Polygon),
                },
            ))
        })
        .collect()
}

/// Fetch grouped daily OHLCV bars for every forex ticker on `date`
/// (`YYYY-MM-DD`), as `(symbol, candle)` pairs.
pub async fn fetch_forex_grouped_daily_response(date: &str) -> Result<Vec<(String, Candle)>> {
    let aggs = forex_grouped_daily(date, None).await?;
    Ok(grouped_daily_to_candles(aggs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_forex_grouped_daily_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/grouped/locale/global/market/fx/2024-01-15")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "adjusted": true,
                    "resultsCount": 1,
                    "queryCount": 1,
                    "results": [
                        { "T": "C:EURUSD", "o": 1.1050, "h": 1.1100, "l": 1.1020, "c": 1.1080, "v": 50000.0, "t": 1705276800000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/aggs/grouped/locale/global/market/fx/2024-01-15", &[])
            .await
            .unwrap();

        let resp: AggregateResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let results = resp.results.unwrap();
        assert!(!results.is_empty());
        assert!((results[0].open - 1.1050).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_forex_grouped_daily_mock_maps_per_ticker_candles() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/grouped/locale/global/market/fx/2024-01-16")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "adjusted": true,
                    "resultsCount": 1,
                    "results": [
                        { "T": "C:GBPUSD", "o": 1.2700, "h": 1.2750, "l": 1.2680, "c": 1.2730, "v": 30000.0, "t": 1704067200000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/aggs/grouped/locale/global/market/fx/2024-01-16", &[])
            .await
            .unwrap();

        let resp: AggregateResponseDTO = serde_json::from_value(json).unwrap();
        let candles = grouped_daily_to_candles(resp);
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].0, "C:GBPUSD");
        assert!((candles[0].1.close - 1.2730).abs() < 0.0001);
    }
}
