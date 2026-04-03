//! Stock technical indicator endpoints: SMA, EMA, MACD, RSI.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch SMA (Simple Moving Average) for a stock ticker.
pub async fn stock_sma(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "sma", params).await
}

/// Fetch EMA (Exponential Moving Average) for a stock ticker.
pub async fn stock_ema(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "ema", params).await
}

/// Fetch MACD for a stock ticker.
pub async fn stock_macd(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "macd", params).await
}

/// Fetch RSI (Relative Strength Index) for a stock ticker.
pub async fn stock_rsi(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "rsi", params).await
}

async fn fetch_indicator(
    ticker: &str,
    indicator: &str,
    params: &[(&str, &str)],
) -> Result<IndicatorResponse> {
    let client = build_client()?;
    let path = format!("/v1/indicators/{}/{}", indicator, ticker);
    let json = client.get_raw(&path, params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: indicator.to_string(),
        context: format!("Failed to parse {indicator} response: {e}"),
    })
}
