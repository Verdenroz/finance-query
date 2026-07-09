//! Stock technical indicator endpoints: SMA, EMA, MACD, RSI.
#![allow(dead_code)]

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use super::build_client;
use super::models::*;

/// Fetch SMA (Simple Moving Average) for a stock ticker.
pub async fn stock_sma(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponseDTO> {
    fetch_indicator(ticker, "sma", params).await
}

/// Fetch EMA (Exponential Moving Average) for a stock ticker.
pub async fn stock_ema(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponseDTO> {
    fetch_indicator(ticker, "ema", params).await
}

/// Fetch MACD for a stock ticker.
pub async fn stock_macd(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponseDTO> {
    fetch_indicator(ticker, "macd", params).await
}

/// Fetch RSI (Relative Strength Index) for a stock ticker.
pub async fn stock_rsi(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponseDTO> {
    fetch_indicator(ticker, "rsi", params).await
}

async fn fetch_indicator(
    ticker: &str,
    indicator: &str,
    params: &[(&str, &str)],
) -> Result<IndicatorResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v1/indicators/{}/{}",
        indicator,
        encode_path_segment(ticker)
    );
    client
        .get_as(&path, params, indicator, &format!("{indicator} response"))
        .await
}
