//! Technical indicator endpoints (SMA, EMA, RSI, MACD, WMA, DEMA, TEMA, Williams, ADX).

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// A single technical indicator data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TechnicalIndicatorValue {
    /// Date or datetime string.
    pub date: Option<String>,
    /// Open price.
    pub open: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// Trading volume.
    pub volume: Option<f64>,
    /// Simple moving average.
    pub sma: Option<f64>,
    /// Exponential moving average.
    pub ema: Option<f64>,
    /// Relative strength index.
    pub rsi: Option<f64>,
    /// MACD value.
    pub macd: Option<f64>,
    /// MACD signal line.
    #[serde(rename = "macdSignal")]
    pub macd_signal: Option<f64>,
    /// MACD histogram.
    #[serde(rename = "macdHist")]
    pub macd_hist: Option<f64>,
    /// Weighted moving average.
    pub wma: Option<f64>,
    /// Double exponential moving average.
    pub dema: Option<f64>,
    /// Triple exponential moving average.
    pub tema: Option<f64>,
    /// Williams %R.
    pub williams: Option<f64>,
    /// Average directional index.
    pub adx: Option<f64>,
}

// ============================================================================
// Helpers
// ============================================================================

/// Fetch a daily technical indicator.
async fn fetch_daily_indicator(
    symbol: &str,
    indicator_name: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    let client = build_client()?;
    let path = format!("/api/v3/technical_indicator/daily/{symbol}");
    let period_str = period.to_string();
    client
        .get(
            &path,
            &[
                ("period", &*period_str),
                ("type", type_),
                ("indicator", indicator_name),
            ],
        )
        .await
}

/// Fetch an intraday technical indicator.
async fn fetch_intraday_indicator(
    symbol: &str,
    interval: &str,
    indicator_name: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    let client = build_client()?;
    let path = format!("/api/v3/technical_indicator/{interval}/{symbol}");
    let period_str = period.to_string();
    client
        .get(
            &path,
            &[
                ("period", &*period_str),
                ("type", type_),
                ("indicator", indicator_name),
            ],
        )
        .await
}

// ============================================================================
// Daily indicators
// ============================================================================

/// Fetch daily Simple Moving Average (SMA).
pub async fn daily_sma(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "sma", period, type_).await
}

/// Fetch daily Exponential Moving Average (EMA).
pub async fn daily_ema(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "ema", period, type_).await
}

/// Fetch daily Relative Strength Index (RSI).
pub async fn daily_rsi(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "rsi", period, type_).await
}

/// Fetch daily MACD.
pub async fn daily_macd(
    symbol: &str,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    let client = build_client()?;
    let path = format!("/api/v3/technical_indicator/daily/{symbol}");
    client
        .get(&path, &[("type", type_), ("indicator", "macd")])
        .await
}

/// Fetch daily Weighted Moving Average (WMA).
pub async fn daily_wma(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "wma", period, type_).await
}

/// Fetch daily Double Exponential Moving Average (DEMA).
pub async fn daily_dema(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "dema", period, type_).await
}

/// Fetch daily Triple Exponential Moving Average (TEMA).
pub async fn daily_tema(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "tema", period, type_).await
}

/// Fetch daily Williams %R.
pub async fn daily_williams(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "williams", period, type_).await
}

/// Fetch daily Average Directional Index (ADX).
pub async fn daily_adx(
    symbol: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_daily_indicator(symbol, "adx", period, type_).await
}

// ============================================================================
// Intraday indicators
// ============================================================================

/// Fetch intraday Simple Moving Average (SMA).
///
/// * `interval` - e.g., `"1min"`, `"5min"`, `"15min"`, `"30min"`, `"1hour"`, `"4hour"`
pub async fn intraday_sma(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "sma", period, type_).await
}

/// Fetch intraday Exponential Moving Average (EMA).
pub async fn intraday_ema(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "ema", period, type_).await
}

/// Fetch intraday Relative Strength Index (RSI).
pub async fn intraday_rsi(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "rsi", period, type_).await
}

/// Fetch intraday MACD.
pub async fn intraday_macd(
    symbol: &str,
    interval: &str,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    let client = build_client()?;
    let path = format!("/api/v3/technical_indicator/{interval}/{symbol}");
    client
        .get(&path, &[("type", type_), ("indicator", "macd")])
        .await
}

/// Fetch intraday Weighted Moving Average (WMA).
pub async fn intraday_wma(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "wma", period, type_).await
}

/// Fetch intraday Double Exponential Moving Average (DEMA).
pub async fn intraday_dema(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "dema", period, type_).await
}

/// Fetch intraday Triple Exponential Moving Average (TEMA).
pub async fn intraday_tema(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "tema", period, type_).await
}

/// Fetch intraday Williams %R.
pub async fn intraday_williams(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "williams", period, type_).await
}

/// Fetch intraday Average Directional Index (ADX).
pub async fn intraday_adx(
    symbol: &str,
    interval: &str,
    period: u32,
    type_: &str,
) -> Result<Vec<TechnicalIndicatorValue>> {
    fetch_intraday_indicator(symbol, interval, "adx", period, type_).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daily_sma_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/technical_indicator/daily/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "20".into()),
                mockito::Matcher::UrlEncoded("type".into(), "close".into()),
                mockito::Matcher::UrlEncoded("indicator".into(), "sma".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "date": "2024-01-15",
                        "open": 182.0,
                        "high": 185.0,
                        "low": 181.0,
                        "close": 184.0,
                        "volume": 50000000.0,
                        "sma": 183.5
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let path = "/api/v3/technical_indicator/daily/AAPL";
        let resp: Vec<TechnicalIndicatorValue> = client
            .get(
                path,
                &[
                    ("period", "20"),
                    ("type", "close"),
                    ("indicator", "sma"),
                ],
            )
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert!((resp[0].sma.unwrap() - 183.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_intraday_ema_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/technical_indicator/5min/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "10".into()),
                mockito::Matcher::UrlEncoded("type".into(), "close".into()),
                mockito::Matcher::UrlEncoded("indicator".into(), "ema".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "date": "2024-01-15 10:05:00",
                        "open": 182.5,
                        "high": 183.0,
                        "low": 182.0,
                        "close": 182.8,
                        "volume": 1200000.0,
                        "ema": 182.6
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let path = "/api/v3/technical_indicator/5min/AAPL";
        let resp: Vec<TechnicalIndicatorValue> = client
            .get(
                path,
                &[
                    ("period", "10"),
                    ("type", "close"),
                    ("indicator", "ema"),
                ],
            )
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert!((resp[0].ema.unwrap() - 182.6).abs() < 0.01);
    }
}
