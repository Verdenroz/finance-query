//! Cryptocurrency endpoints: exchange rates and crypto time series.

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Helper to parse crypto time series responses.
fn parse_crypto_series(
    json: &serde_json::Value,
    symbol: &str,
    market: &str,
) -> Result<CryptoTimeSeries> {
    let meta = json.get("Meta Data");
    let last_refreshed = meta
        .and_then(|m| {
            m.get("6. Last Refreshed")
                .or_else(|| m.get("5. Last Refreshed"))
        })
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let series = json
        .as_object()
        .and_then(|obj| {
            obj.iter()
                .find(|(k, _)| k.starts_with("Time Series"))
                .map(|(_, v)| v)
        })
        .and_then(|v| v.as_object())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Time Series Crypto".to_string(),
            context: "Missing crypto time series data".to_string(),
        })?;

    let market_lower = market.to_lowercase();
    let open_key = format!("1a. open ({market_lower})");
    let high_key = format!("2a. high ({market_lower})");
    let low_key = format!("3a. low ({market_lower})");
    let close_key = format!("4a. close ({market_lower})");

    let mut entries: Vec<CryptoEntry> = series
        .iter()
        .filter_map(|(timestamp, values)| {
            // Try market-specific keys first, fall back to generic numbered keys
            let open = values
                .get(&open_key)
                .or_else(|| values.get("1. open"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let high = values
                .get(&high_key)
                .or_else(|| values.get("2. high"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let low = values
                .get(&low_key)
                .or_else(|| values.get("3. low"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let close = values
                .get(&close_key)
                .or_else(|| values.get("4. close"))
                .and_then(|v| v.as_str()?.parse().ok())?;
            let volume = values
                .get("5. volume")
                .and_then(|v| v.as_str()?.parse().ok())
                .unwrap_or(0.0);

            Some(CryptoEntry {
                timestamp: timestamp.clone(),
                open,
                high,
                low,
                close,
                volume,
            })
        })
        .collect();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(CryptoTimeSeries {
        symbol: symbol.to_string(),
        market: market.to_string(),
        last_refreshed,
        entries,
    })
}

/// Fetch intraday cryptocurrency time series.
pub async fn crypto_intraday(
    symbol: &str,
    market: &str,
    interval: AvInterval,
    output_size: Option<OutputSize>,
) -> Result<CryptoTimeSeries> {
    let client = build_client()?;
    let mut params = vec![
        ("symbol", symbol),
        ("market", market),
        ("interval", interval.as_str()),
    ];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("CRYPTO_INTRADAY", &params).await?;
    parse_crypto_series(&json, symbol, market)
}

/// Fetch daily cryptocurrency time series.
pub async fn crypto_daily(symbol: &str, market: &str) -> Result<CryptoTimeSeries> {
    let client = build_client()?;
    let json = client
        .get(
            "DIGITAL_CURRENCY_DAILY",
            &[("symbol", symbol), ("market", market)],
        )
        .await?;
    parse_crypto_series(&json, symbol, market)
}

/// Fetch weekly cryptocurrency time series.
pub async fn crypto_weekly(symbol: &str, market: &str) -> Result<CryptoTimeSeries> {
    let client = build_client()?;
    let json = client
        .get(
            "DIGITAL_CURRENCY_WEEKLY",
            &[("symbol", symbol), ("market", market)],
        )
        .await?;
    parse_crypto_series(&json, symbol, market)
}

/// Fetch monthly cryptocurrency time series.
pub async fn crypto_monthly(symbol: &str, market: &str) -> Result<CryptoTimeSeries> {
    let client = build_client()?;
    let json = client
        .get(
            "DIGITAL_CURRENCY_MONTHLY",
            &[("symbol", symbol), ("market", market)],
        )
        .await?;
    parse_crypto_series(&json, symbol, market)
}
