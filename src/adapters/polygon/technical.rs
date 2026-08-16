//! Server-calculated stock technical indicators.
// unrouted: awaiting a capability route; see #264.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use super::build_client;

/// Technical indicator calculated by Massive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechnicalIndicator {
    /// Simple moving average.
    Sma,
    /// Exponential moving average.
    Ema,
    /// Moving average convergence/divergence.
    Macd,
    /// Relative strength index.
    Rsi,
}

impl TechnicalIndicator {
    fn as_path_segment(self) -> &'static str {
        match self {
            Self::Sma => "sma",
            Self::Ema => "ema",
            Self::Macd => "macd",
            Self::Rsi => "rsi",
        }
    }
}

/// One calculated indicator value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TechnicalIndicatorValueDTO {
    /// Aggregate-window timestamp in Unix milliseconds.
    pub timestamp: i64,
    /// Indicator value. For MACD this is the MACD line.
    pub value: f64,
    /// MACD signal line.
    pub signal: Option<f64>,
    /// MACD histogram.
    pub histogram: Option<f64>,
}

/// Underlying aggregates referenced by an indicator response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TechnicalIndicatorUnderlyingDTO {
    /// URL of the aggregate data used in the calculation.
    pub url: Option<String>,
}

/// Technical-indicator result payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TechnicalIndicatorResultsDTO {
    /// Aggregate data used by the calculation.
    pub underlying: Option<TechnicalIndicatorUnderlyingDTO>,
    /// Calculated values.
    #[serde(default)]
    pub values: Vec<TechnicalIndicatorValueDTO>,
}

/// Response envelope for a technical-indicator request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TechnicalIndicatorResponseDTO {
    /// Request identifier.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// Cursor URL for the next page.
    pub next_url: Option<String>,
    /// Indicator values and underlying aggregates.
    pub results: Option<TechnicalIndicatorResultsDTO>,
}

/// Fetch an SMA, EMA, MACD, or RSI series for a stock.
///
/// Massive accepts filters such as `timestamp`, `timespan`, `adjusted`,
/// `window`, `series_type`, `order`, `limit`, and indicator-specific windows.
pub async fn technical_indicator(
    indicator: TechnicalIndicator,
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<TechnicalIndicatorResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v1/indicators/{}/{}",
        indicator.as_path_segment(),
        encode_path_segment(ticker)
    );
    client
        .get_as(
            &path,
            params,
            "technical_indicator",
            "technical indicator response",
        )
        .await
}

/// Fetch a simple moving average series.
pub async fn stock_sma(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<TechnicalIndicatorResponseDTO> {
    technical_indicator(TechnicalIndicator::Sma, ticker, params).await
}

/// Fetch an exponential moving average series.
pub async fn stock_ema(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<TechnicalIndicatorResponseDTO> {
    technical_indicator(TechnicalIndicator::Ema, ticker, params).await
}

/// Fetch a moving average convergence/divergence series.
pub async fn stock_macd(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<TechnicalIndicatorResponseDTO> {
    technical_indicator(TechnicalIndicator::Macd, ticker, params).await
}

/// Fetch a relative strength index series.
pub async fn stock_rsi(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<TechnicalIndicatorResponseDTO> {
    technical_indicator(TechnicalIndicator::Rsi, ticker, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn uses_current_indicator_route_and_parses_values() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/sma/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "2".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"status":"OK","results":{"values":[{"timestamp":1,"value":42.5}]}}"#)
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let response: TechnicalIndicatorResponseDTO = client
            .get_as(
                "/v1/indicators/sma/AAPL",
                &[("limit", "2")],
                "technical_indicator",
                "technical indicator response",
            )
            .await
            .unwrap();
        assert_eq!(response.results.unwrap().values[0].value, 42.5);
    }
}
