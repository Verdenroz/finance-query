//! US options data endpoints: realtime and historical options chains.

use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Helper to parse options contracts from the response.
fn parse_contracts(json: &serde_json::Value) -> Result<Vec<OptionContract>> {
    let data = json
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "data".to_string(),
            context: "Missing data array in options response".to_string(),
        })?;

    Ok(data
        .iter()
        .filter_map(|c| {
            Some(OptionContract {
                contractid: c.get("contractID")?.as_str()?.to_string(),
                symbol: c.get("symbol")?.as_str()?.to_string(),
                expiration: c.get("expiration")?.as_str()?.to_string(),
                strike: c.get("strike")?.as_str()?.parse().ok()?,
                option_type: c.get("type")?.as_str()?.to_string(),
                last: c.get("last").and_then(|v| v.as_str()?.parse().ok()),
                mark: c.get("mark").and_then(|v| v.as_str()?.parse().ok()),
                bid: c.get("bid").and_then(|v| v.as_str()?.parse().ok()),
                bid_size: c.get("bid_size").and_then(|v| v.as_str()?.parse().ok()),
                ask: c.get("ask").and_then(|v| v.as_str()?.parse().ok()),
                ask_size: c.get("ask_size").and_then(|v| v.as_str()?.parse().ok()),
                volume: c.get("volume").and_then(|v| v.as_str()?.parse().ok()),
                open_interest: c
                    .get("open_interest")
                    .and_then(|v| v.as_str()?.parse().ok()),
                implied_volatility: c
                    .get("implied_volatility")
                    .and_then(|v| v.as_str()?.parse().ok()),
                delta: c.get("delta").and_then(|v| v.as_str()?.parse().ok()),
                gamma: c.get("gamma").and_then(|v| v.as_str()?.parse().ok()),
                theta: c.get("theta").and_then(|v| v.as_str()?.parse().ok()),
                vega: c.get("vega").and_then(|v| v.as_str()?.parse().ok()),
                rho: c.get("rho").and_then(|v| v.as_str()?.parse().ok()),
            })
        })
        .collect())
}

/// Fetch realtime US options chain for a symbol.
///
/// # Arguments
///
/// * `symbol` - Underlying ticker symbol (e.g., `"AAPL"`)
/// * `contract` - Optional specific contract ID to filter
pub async fn realtime_options(symbol: &str, contract: Option<&str>) -> Result<OptionsChain> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol)];
    if let Some(c) = contract {
        params.push(("contract", c));
    }
    let json = client.get("REALTIME_OPTIONS", &params).await?;
    let contracts = parse_contracts(&json)?;

    Ok(OptionsChain {
        symbol: symbol.to_string(),
        contracts,
    })
}

/// Fetch historical options chain for a symbol on a specific date.
///
/// # Arguments
///
/// * `symbol` - Underlying ticker symbol (e.g., `"AAPL"`)
/// * `date` - Date in `YYYY-MM-DD` format (optional, defaults to latest)
pub async fn historical_options(symbol: &str, date: Option<&str>) -> Result<OptionsChain> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol)];
    if let Some(d) = date {
        params.push(("date", d));
    }
    let json = client.get("HISTORICAL_OPTIONS", &params).await?;
    let contracts = parse_contracts(&json)?;

    Ok(OptionsChain {
        symbol: symbol.to_string(),
        contracts,
    })
}
