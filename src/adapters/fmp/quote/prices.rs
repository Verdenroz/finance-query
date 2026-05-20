//! FMP price and historical data endpoints.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use crate::adapters::fmp::models::{
    FmpQuoteDTO, HistoricalPriceDTO, HistoricalPriceResponseDTO, IntradayPriceDTO,
};

// ============================================================================
// Additional response types
// ============================================================================

/// Stock price change from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockPriceChangeDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// 1-day price change.
    #[serde(rename = "1D")]
    pub one_day: Option<f64>,
    /// 5-day price change.
    #[serde(rename = "5D")]
    pub five_day: Option<f64>,
    /// 1-month price change.
    #[serde(rename = "1M")]
    pub one_month: Option<f64>,
    /// 3-month price change.
    #[serde(rename = "3M")]
    pub three_month: Option<f64>,
    /// 6-month price change.
    #[serde(rename = "6M")]
    pub six_month: Option<f64>,
    /// Year-to-date price change.
    pub ytd: Option<f64>,
    /// 1-year price change.
    #[serde(rename = "1Y")]
    pub one_year: Option<f64>,
    /// 3-year price change.
    #[serde(rename = "3Y")]
    pub three_year: Option<f64>,
    /// 5-year price change.
    #[serde(rename = "5Y")]
    pub five_year: Option<f64>,
    /// 10-year price change.
    #[serde(rename = "10Y")]
    pub ten_year: Option<f64>,
    /// Max price change.
    pub max: Option<f64>,
}

/// Optional parameters for historical price queries.
#[derive(Debug, Clone, Default)]
pub struct HistoricalPriceParams {
    /// Start date (YYYY-MM-DD).
    pub from: Option<String>,
    /// End date (YYYY-MM-DD).
    pub to: Option<String>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Convert FMP quote DTOs into a canonical QuoteSummaryResponse.
fn quote_to_canonical(
    symbol: &str,
    quotes: &[FmpQuoteDTO],
) -> crate::models::quote::QuoteSummaryResponse {
    use crate::models::quote::{FormattedValue, Price, QuoteSummaryResponse};
    let q = quotes.first();
    let price = Price {
        regular_market_price: q.and_then(|q| q.price).map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_change_percent: q.and_then(|q| q.changes_percentage).map(|v| {
            FormattedValue {
                raw: Some(v),
                fmt: None,
                long_fmt: None,
            }
        }),
        regular_market_volume: q
            .and_then(|q| q.volume.map(|v| v as i64))
            .map(|v| FormattedValue {
                raw: Some(v),
                fmt: None,
                long_fmt: None,
            }),
        regular_market_day_high: q.and_then(|q| q.day_high).map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_day_low: q.and_then(|q| q.day_low).map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        }),
        market_cap: q
            .and_then(|q| q.market_cap.map(|v| v as i64))
            .map(|v| FormattedValue {
                raw: Some(v),
                fmt: None,
                long_fmt: None,
            }),
        exchange: q.and_then(|q| q.exchange.clone()),
        ..Default::default()
    };
    QuoteSummaryResponse {
        symbol: symbol.to_string(),
        price: Some(price),
        ..Default::default()
    }
}

/// Fetch a canonical quote summary for a symbol.
pub async fn fetch_canonical_quote(
    symbol: &str,
) -> Result<crate::models::quote::QuoteSummaryResponse> {
    let quotes = quote(symbol).await?;
    Ok(quote_to_canonical(symbol, &quotes))
}

/// Convert historical daily price DTOs into canonical Chart candles.
fn historical_to_candles(historical: Vec<HistoricalPriceDTO>) -> Vec<crate::models::chart::Candle> {
    historical
        .into_iter()
        .filter_map(|r| {
            let ts = chrono::NaiveDate::parse_from_str(r.date.as_deref()?, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)?
                .and_utc()
                .timestamp();
            Some(crate::models::chart::Candle {
                timestamp: ts,
                open: r.open?,
                high: r.high?,
                low: r.low?,
                close: r.close?,
                volume: r.volume.map(|v| v as i64).unwrap_or(0),
                adj_close: None,
                provider_id: Some(crate::providers::Provider::Fmp),
            })
        })
        .collect()
}

/// Convert intraday price DTOs into canonical Chart candles.
fn intraday_to_candles(intraday: Vec<IntradayPriceDTO>) -> Vec<crate::models::chart::Candle> {
    intraday
        .into_iter()
        .filter_map(|r| {
            let ts = chrono::NaiveDateTime::parse_from_str(r.date.as_deref()?, "%Y-%m-%d %H:%M:%S")
                .ok()?
                .and_utc()
                .timestamp();
            Some(crate::models::chart::Candle {
                timestamp: ts,
                open: r.open?,
                high: r.high?,
                low: r.low?,
                close: r.close?,
                volume: r.volume.map(|v| v as i64).unwrap_or(0),
                adj_close: None,
                provider_id: Some(crate::providers::Provider::Fmp),
            })
        })
        .collect()
}

/// Fetch canonical daily chart data with date range.
pub async fn fetch_daily_chart_canonical(
    symbol: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<crate::models::chart::Chart> {
    let params = from.and_then(|f| {
        to.map(|t| HistoricalPriceParams {
            from: Some(f.to_string()),
            to: Some(t.to_string()),
        })
    });
    let resp = historical_price_daily(symbol, params).await?;
    let candles = historical_to_candles(resp.historical);
    Ok(crate::models::chart::Chart {
        symbol: symbol.to_string(),
        meta: Default::default(),
        candles,
        interval: None,
        range: None,
        provider_id: Some(crate::providers::Provider::Fmp),
    })
}

/// Fetch canonical chart candles from daily historical price data.
pub async fn fetch_daily_chart_candles(
    symbol: &str,
    params: Option<HistoricalPriceParams>,
) -> Result<Vec<crate::models::chart::Candle>> {
    let resp = historical_price_daily(symbol, params).await?;
    Ok(historical_to_candles(resp.historical))
}

/// Fetch canonical chart candles from intraday price data.
pub async fn fetch_intraday_chart_candles(
    symbol: &str,
    interval: &str,
    params: Option<HistoricalPriceParams>,
) -> Result<Vec<crate::models::chart::Candle>> {
    let points = historical_price_intraday(symbol, interval, params).await?;
    Ok(intraday_to_candles(points))
}

/// Fetch real-time quote for a symbol.
pub async fn quote(symbol: &str) -> Result<Vec<FmpQuoteDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    client
        .get(
            &format!("/api/v3/quote/{}", encode_path_segment(symbol)),
            &[],
        )
        .await
}

/// Fetch real-time quotes for multiple symbols (comma-separated).
pub async fn batch_quote(symbols: &[&str]) -> Result<Vec<FmpQuoteDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    let joined = symbols.join(",");
    client
        .get(
            &format!("/api/v3/quote/{}", encode_path_segment(&joined)),
            &[],
        )
        .await
}

/// Fetch stock price change percentages for a symbol.
pub async fn stock_price(symbol: &str) -> Result<Vec<StockPriceChangeDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    client
        .get(
            &format!("/api/v3/stock-price-change/{}", encode_path_segment(symbol)),
            &[],
        )
        .await
}

/// Fetch historical daily prices for a symbol.
pub async fn historical_price_daily(
    symbol: &str,
    params: Option<HistoricalPriceParams>,
) -> Result<HistoricalPriceResponseDTO> {
    let client = crate::adapters::fmp::build_client()?;
    let p = params.unwrap_or_default();
    let mut query_params: Vec<(&str, &str)> = Vec::new();
    if let Some(ref from) = p.from {
        query_params.push(("from", from));
    }
    if let Some(ref to) = p.to {
        query_params.push(("to", to));
    }
    client
        .get(
            &format!(
                "/api/v3/historical-price-full/{}",
                encode_path_segment(symbol)
            ),
            &query_params,
        )
        .await
}

/// Fetch intraday historical prices for a symbol.
///
/// Valid intervals: "1min", "5min", "15min", "30min", "1hour", "4hour".
pub async fn historical_price_intraday(
    symbol: &str,
    interval: &str,
    params: Option<HistoricalPriceParams>,
) -> Result<Vec<IntradayPriceDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    let p = params.unwrap_or_default();
    let mut query_params: Vec<(&str, &str)> = Vec::new();
    if let Some(ref from) = p.from {
        query_params.push(("from", from));
    }
    if let Some(ref to) = p.to {
        query_params.push(("to", to));
    }
    client
        .get(
            &format!(
                "/api/v3/historical-chart/{}/{}",
                encode_path_segment(interval),
                encode_path_segment(symbol)
            ),
            &query_params,
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quote_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/quote/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "symbol": "AAPL",
                    "name": "Apple Inc.",
                    "price": 178.72,
                    "change": 2.15,
                    "changesPercentage": 1.22,
                    "dayLow": 176.21,
                    "dayHigh": 179.63,
                    "yearLow": 124.17,
                    "yearHigh": 199.62,
                    "marketCap": 2794000000000_f64,
                    "volume": 58405568,
                    "avgVolume": 54638267,
                    "open": 177.09,
                    "previousClose": 176.57,
                    "eps": 6.42,
                    "pe": 27.84,
                    "timestamp": 1701460800,
                    "exchange": "NASDAQ"
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<FmpQuoteDTO> = client.get("/api/v3/quote/AAPL", &[]).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol, "AAPL");
        assert_eq!(result[0].name.as_deref(), Some("Apple Inc."));
        assert_eq!(result[0].price, Some(178.72));
        assert_eq!(result[0].pe, Some(27.84));
    }

    #[tokio::test]
    async fn test_historical_price_daily_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/historical-price-full/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!({
                    "symbol": "AAPL",
                    "historical": [
                        {
                            "date": "2024-01-02",
                            "open": 187.15,
                            "high": 188.44,
                            "low": 183.89,
                            "close": 185.64,
                            "adjClose": 184.96,
                            "volume": 82488700,
                            "unadjustedVolume": 82488700,
                            "change": -1.51,
                            "changePercent": -0.8068,
                            "vwap": 185.99,
                            "label": "January 02, 2024",
                            "changeOverTime": -0.008068
                        },
                        {
                            "date": "2024-01-03",
                            "open": 184.22,
                            "high": 185.88,
                            "low": 183.43,
                            "close": 184.25,
                            "adjClose": 183.57,
                            "volume": 58414500,
                            "unadjustedVolume": 58414500,
                            "change": 0.03,
                            "changePercent": 0.0163,
                            "vwap": 184.52,
                            "label": "January 03, 2024",
                            "changeOverTime": 0.000163
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: HistoricalPriceResponseDTO = client
            .get("/api/v3/historical-price-full/AAPL", &[])
            .await
            .unwrap();

        assert_eq!(result.symbol.as_deref(), Some("AAPL"));
        assert_eq!(result.historical.len(), 2);
        assert_eq!(result.historical[0].date.as_deref(), Some("2024-01-02"));
        assert_eq!(result.historical[0].close, Some(185.64));
    }

    #[tokio::test]
    async fn test_intraday_price_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/historical-chart/5min/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "date": "2024-01-02 09:30:00",
                        "open": 187.15,
                        "high": 187.44,
                        "low": 186.89,
                        "close": 187.20,
                        "volume": 1234567
                    },
                    {
                        "date": "2024-01-02 09:35:00",
                        "open": 187.20,
                        "high": 187.50,
                        "low": 187.10,
                        "close": 187.35,
                        "volume": 987654
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<IntradayPriceDTO> = client
            .get("/api/v3/historical-chart/5min/AAPL", &[])
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].date.as_deref(), Some("2024-01-02 09:30:00"));
        assert_eq!(result[0].close, Some(187.20));
    }
}
