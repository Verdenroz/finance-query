//! Forex (foreign exchange) endpoints: exchange rates and FX time series.

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Fetch the real-time exchange rate between two currencies.
pub async fn exchange_rate(from_currency: &str, to_currency: &str) -> Result<ExchangeRate> {
    let client = build_client()?;
    let json = client
        .get(
            "CURRENCY_EXCHANGE_RATE",
            &[
                ("from_currency", from_currency),
                ("to_currency", to_currency),
            ],
        )
        .await?;

    let rate = json.get("Realtime Currency Exchange Rate").ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "Realtime Currency Exchange Rate".to_string(),
            context: "Missing exchange rate data in response".to_string(),
        }
    })?;

    Ok(ExchangeRate {
        from_currency_code: rate
            .get("1. From_Currency Code")
            .and_then(|v| v.as_str())
            .unwrap_or(from_currency)
            .to_string(),
        from_currency_name: rate
            .get("2. From_Currency Name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        to_currency_code: rate
            .get("3. To_Currency Code")
            .and_then(|v| v.as_str())
            .unwrap_or(to_currency)
            .to_string(),
        to_currency_name: rate
            .get("4. To_Currency Name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        exchange_rate: rate
            .get("5. Exchange Rate")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        last_refreshed: rate
            .get("6. Last Refreshed")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        bid_price: rate
            .get("8. Bid Price")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
        ask_price: rate
            .get("9. Ask Price")
            .and_then(|v| v.as_str()?.parse().ok())
            .unwrap_or(0.0),
    })
}

/// Helper to parse FX time series responses.
fn parse_fx_series(
    json: &serde_json::Value,
    from_symbol: &str,
    to_symbol: &str,
) -> Result<ForexTimeSeries> {
    let meta = json.get("Meta Data");
    let last_refreshed = meta
        .and_then(|m| {
            m.get("5. Last Refreshed")
                .or_else(|| m.get("4. Last Refreshed"))
        })
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Find the time series key (varies by endpoint)
    let series = json
        .as_object()
        .and_then(|obj| {
            obj.iter()
                .find(|(k, _)| k.starts_with("Time Series"))
                .map(|(_, v)| v)
        })
        .and_then(|v| v.as_object())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "Time Series FX".to_string(),
            context: "Missing FX time series data".to_string(),
        })?;

    let mut entries: Vec<ForexEntry> = series
        .iter()
        .filter_map(|(timestamp, values)| {
            Some(ForexEntry {
                timestamp: timestamp.clone(),
                open: values.get("1. open")?.as_str()?.parse().ok()?,
                high: values.get("2. high")?.as_str()?.parse().ok()?,
                low: values.get("3. low")?.as_str()?.parse().ok()?,
                close: values.get("4. close")?.as_str()?.parse().ok()?,
            })
        })
        .collect();

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(ForexTimeSeries {
        from_symbol: from_symbol.to_string(),
        to_symbol: to_symbol.to_string(),
        last_refreshed,
        entries,
    })
}

/// Fetch intraday FX time series.
pub async fn fx_intraday(
    from_symbol: &str,
    to_symbol: &str,
    interval: AvInterval,
    output_size: Option<OutputSize>,
) -> Result<ForexTimeSeries> {
    let client = build_client()?;
    let mut params = vec![
        ("from_symbol", from_symbol),
        ("to_symbol", to_symbol),
        ("interval", interval.as_str()),
    ];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("FX_INTRADAY", &params).await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}

/// Fetch daily FX time series.
pub async fn fx_daily(
    from_symbol: &str,
    to_symbol: &str,
    output_size: Option<OutputSize>,
) -> Result<ForexTimeSeries> {
    let client = build_client()?;
    let mut params = vec![("from_symbol", from_symbol), ("to_symbol", to_symbol)];
    let os_str;
    if let Some(os) = output_size {
        os_str = os.as_str().to_string();
        params.push(("outputsize", &os_str));
    }
    let json = client.get("FX_DAILY", &params).await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}

/// Fetch weekly FX time series.
pub async fn fx_weekly(from_symbol: &str, to_symbol: &str) -> Result<ForexTimeSeries> {
    let client = build_client()?;
    let json = client
        .get(
            "FX_WEEKLY",
            &[("from_symbol", from_symbol), ("to_symbol", to_symbol)],
        )
        .await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}

/// Fetch monthly FX time series.
pub async fn fx_monthly(from_symbol: &str, to_symbol: &str) -> Result<ForexTimeSeries> {
    let client = build_client()?;
    let json = client
        .get(
            "FX_MONTHLY",
            &[("from_symbol", from_symbol), ("to_symbol", to_symbol)],
        )
        .await?;
    parse_fx_series(&json, from_symbol, to_symbol)
}
