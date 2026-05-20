//! US options data endpoints: realtime and historical options chains.

#![allow(dead_code)]
use crate::error::{FinanceError, Result};

use super::build_client;
use super::models::*;

/// Helper to parse options contracts from the response.
fn parse_contracts(json: &serde_json::Value) -> Result<Vec<OptionContractDTO>> {
    let data = json.get("data").and_then(|v| v.as_array()).ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "data".to_string(),
            context: "Missing data array in options response".to_string(),
        }
    })?;

    Ok(data
        .iter()
        .filter_map(|c| {
            Some(OptionContractDTO {
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
pub async fn realtime_options(symbol: &str, contract: Option<&str>) -> Result<OptionsChainDTO> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol)];
    if let Some(c) = contract {
        params.push(("contract", c));
    }
    let json = client.get("REALTIME_OPTIONS", &params).await?;
    let contracts = parse_contracts(&json)?;

    Ok(OptionsChainDTO {
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
pub async fn historical_options(symbol: &str, date: Option<&str>) -> Result<OptionsChainDTO> {
    let client = build_client()?;
    let mut params = vec![("symbol", symbol)];
    if let Some(d) = date {
        params.push(("date", d));
    }
    let json = client.get("HISTORICAL_OPTIONS", &params).await?;
    let contracts = parse_contracts(&json)?;

    Ok(OptionsChainDTO {
        symbol: symbol.to_string(),
        contracts,
    })
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical Options chain for a symbol.
pub async fn fetch_options_response(
    symbol: &str,
    date: Option<i64>,
) -> Result<crate::models::options::Options> {
    let date_str = date.map(timestamp_to_date_string);
    let chain = if let Some(ref d) = date_str {
        historical_options(symbol, Some(d)).await?
    } else {
        realtime_options(symbol, None).await?
    };

    let mut calls = Vec::new();
    let mut puts = Vec::new();
    let mut expiration_dates_set = std::collections::HashSet::new();

    for contract in chain.contracts {
        let exp_ts = parse_av_date(&contract.expiration).unwrap_or(0);
        expiration_dates_set.insert(exp_ts);

        let contract_data = crate::models::options::OptionContract {
            contract_symbol: contract.contractid,
            strike: contract.strike,
            currency: None,
            last_price: contract.last,
            change: None,
            percent_change: None,
            volume: contract.volume.map(|v| v as i64),
            open_interest: contract.open_interest.map(|v| v as i64),
            bid: contract.bid,
            ask: contract.ask,
            contract_size: None,
            expiration: Some(exp_ts),
            last_trade_date: None,
            implied_volatility: contract.implied_volatility,
            in_the_money: None,
        };

        if contract.option_type == "call" {
            calls.push(contract_data);
        } else {
            puts.push(contract_data);
        }
    }

    let mut expiration_dates: Vec<i64> = expiration_dates_set.into_iter().collect();
    expiration_dates.sort();

    Ok(crate::providers::build_options(
        chain.symbol,
        crate::Provider::AlphaVantage,
        expiration_dates,
        calls,
        puts,
    ))
}

/// Parse an Alpha Vantage date string (YYYY-MM-DD) to a Unix timestamp.
fn parse_av_date(date_str: &str) -> Option<i64> {
    if date_str.is_empty() {
        return None;
    }
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
}

/// Convert a Unix timestamp to a YYYY-MM-DD date string.
fn timestamp_to_date_string(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}
