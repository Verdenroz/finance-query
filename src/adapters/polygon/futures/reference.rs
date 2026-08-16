//! Futures contract, product, schedule, exchange, and market-status reference data.
// unrouted: awaiting a capability route; see #264.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::super::{build_client, models::PaginatedResponseDTO};

/// Listed futures contract specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesContractDTO {
    /// Whether the contract is currently tradeable for the requested date.
    pub active: Option<bool>,
    /// Point-in-time reference date.
    pub date: Option<String>,
    /// Calendar days until maturity.
    pub days_to_maturity: Option<i64>,
    /// First trade date.
    pub first_trade_date: Option<String>,
    /// CME product group code.
    pub group_code: Option<String>,
    /// Last trade date.
    pub last_trade_date: Option<String>,
    /// Maximum order quantity.
    pub max_order_quantity: Option<i64>,
    /// Minimum order quantity.
    pub min_order_quantity: Option<i64>,
    /// Contract name.
    pub name: Option<String>,
    /// Product code.
    pub product_code: Option<String>,
    /// Settlement date.
    pub settlement_date: Option<String>,
    /// Settlement tick size.
    pub settlement_tick_size: Option<f64>,
    /// Spread tick size.
    pub spread_tick_size: Option<f64>,
    /// Contract ticker.
    pub ticker: Option<String>,
    /// Trade tick size.
    pub trade_tick_size: Option<f64>,
    /// Trading venue MIC.
    pub trading_venue: Option<String>,
    /// Contract type, such as `single` or `combo`.
    #[serde(rename = "type")]
    pub contract_type: Option<String>,
}

/// Futures product specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesProductDTO {
    /// Asset class.
    pub asset_class: Option<String>,
    /// Asset sub-class.
    pub asset_sub_class: Option<String>,
    /// Point-in-time reference date.
    pub date: Option<String>,
    /// Last update date.
    pub last_updated: Option<String>,
    /// Product name.
    pub name: Option<String>,
    /// Price quotation convention.
    pub price_quotation: Option<String>,
    /// Product code.
    pub product_code: Option<String>,
    /// Sector.
    pub sector: Option<String>,
    /// Settlement currency.
    pub settlement_currency_code: Option<String>,
    /// Settlement method.
    pub settlement_method: Option<String>,
    /// Settlement type.
    pub settlement_type: Option<String>,
    /// Sub-sector.
    pub sub_sector: Option<String>,
    /// Trading currency.
    pub trade_currency_code: Option<String>,
    /// Trading venue MIC.
    pub trading_venue: Option<String>,
    /// Product type.
    #[serde(rename = "type")]
    pub product_type: Option<String>,
    /// Unit of measure.
    pub unit_of_measure: Option<String>,
    /// Quantity per unit of measure.
    pub unit_of_measure_qty: Option<f64>,
}

/// One futures market schedule event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesScheduleDTO {
    /// Event type, such as `open`, `close`, or `pause`.
    pub event: Option<String>,
    /// Product code.
    pub product_code: Option<String>,
    /// Product name.
    pub product_name: Option<String>,
    /// Trading-session end date.
    pub session_end_date: Option<String>,
    /// Event timestamp in RFC 3339 format.
    pub timestamp: Option<String>,
    /// Trading venue MIC.
    pub trading_venue: Option<String>,
}

/// Current futures market status for a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesMarketStatusDTO {
    /// Current event, such as `open`, `pause`, or `close`.
    pub market_event: Option<String>,
    /// Product name.
    pub name: Option<String>,
    /// Product code.
    pub product_code: Option<String>,
    /// Trading-session end date.
    pub session_end_date: Option<String>,
    /// Evaluation timestamp in RFC 3339 format.
    pub timestamp: Option<String>,
    /// Trading venue MIC.
    pub trading_venue: Option<String>,
}

/// Supported futures exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesExchangeDTO {
    /// Exchange acronym.
    pub acronym: Option<String>,
    /// Massive exchange identifier.
    pub id: Option<String>,
    /// Locale.
    pub locale: Option<String>,
    /// Market identifier code.
    pub mic: Option<String>,
    /// Exchange name.
    pub name: Option<String>,
    /// Operating market identifier code.
    pub operating_mic: Option<String>,
    /// Exchange type.
    #[serde(rename = "type")]
    pub exchange_type: Option<String>,
    /// Exchange website.
    pub url: Option<String>,
}

/// List futures contracts, optionally filtering by ticker, product, date, or activity.
pub async fn futures_contracts(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesContractDTO>> {
    build_client()?.get("/futures/v1/contracts", params).await
}

/// List futures products and their specifications.
pub async fn futures_products(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesProductDTO>> {
    build_client()?.get("/futures/v1/products", params).await
}

/// List futures trading schedule events.
pub async fn futures_schedules(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesScheduleDTO>> {
    build_client()?.get("/futures/v1/schedules", params).await
}

/// Fetch current futures market status by product or venue.
pub async fn futures_market_status(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesMarketStatusDTO>> {
    build_client()?
        .get("/futures/v1/market-status", params)
        .await
}

/// List supported futures exchanges.
pub async fn futures_exchanges(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesExchangeDTO>> {
    build_client()?.get("/futures/v1/exchanges", params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_contract_shape_deserializes() {
        let contract: FuturesContractDTO = serde_json::from_value(serde_json::json!({
            "active": true,
            "ticker": "ESZ6",
            "product_code": "ES",
            "trade_tick_size": 0.25,
            "trading_venue": "XCME",
            "type": "single"
        }))
        .unwrap();
        assert_eq!(contract.ticker.as_deref(), Some("ESZ6"));
        assert_eq!(contract.contract_type.as_deref(), Some("single"));
    }
}
