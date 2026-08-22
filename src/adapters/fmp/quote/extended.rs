// unrouted: awaiting a capability route; see #264.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AftermarketTradeDTO {
    pub symbol: Option<String>,
    pub price: Option<f64>,
    #[serde(rename = "tradeSize")]
    pub trade_size: Option<u64>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AftermarketQuoteDTO {
    pub symbol: Option<String>,
    #[serde(rename = "bidSize")]
    pub bid_size: Option<u64>,
    #[serde(rename = "bidPrice")]
    pub bid_price: Option<f64>,
    #[serde(rename = "askSize")]
    pub ask_size: Option<u64>,
    #[serde(rename = "askPrice")]
    pub ask_price: Option<f64>,
    pub volume: Option<f64>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockPriceChangeDTO {
    pub symbol: Option<String>,
    #[serde(rename = "1D")]
    pub day_1: Option<f64>,
    #[serde(rename = "5D")]
    pub day_5: Option<f64>,
    #[serde(rename = "1M")]
    pub month_1: Option<f64>,
    #[serde(rename = "3M")]
    pub month_3: Option<f64>,
    #[serde(rename = "6M")]
    pub month_6: Option<f64>,
    pub ytd: Option<f64>,
    #[serde(rename = "1Y")]
    pub year_1: Option<f64>,
    #[serde(rename = "3Y")]
    pub year_3: Option<f64>,
    #[serde(rename = "5Y")]
    pub year_5: Option<f64>,
    #[serde(rename = "10Y")]
    pub year_10: Option<f64>,
    #[serde(rename = "max")]
    pub maximum: Option<f64>,
}

pub async fn aftermarket_trade(symbol: &str) -> Result<Vec<AftermarketTradeDTO>> {
    build_client()?
        .get("/stable/aftermarket-trade", &[("symbol", symbol)])
        .await
}

pub async fn batch_aftermarket_trade(symbols: &[&str]) -> Result<Vec<AftermarketTradeDTO>> {
    let symbols = symbols.join(",");
    build_client()?
        .get("/stable/batch-aftermarket-trade", &[("symbols", &symbols)])
        .await
}

pub async fn aftermarket_quote(symbol: &str) -> Result<Vec<AftermarketQuoteDTO>> {
    build_client()?
        .get("/stable/aftermarket-quote", &[("symbol", symbol)])
        .await
}

pub async fn batch_aftermarket_quote(symbols: &[&str]) -> Result<Vec<AftermarketQuoteDTO>> {
    let symbols = symbols.join(",");
    build_client()?
        .get("/stable/batch-aftermarket-quote", &[("symbols", &symbols)])
        .await
}

pub async fn stock_price_change(symbol: &str) -> Result<Vec<StockPriceChangeDTO>> {
    build_client()?
        .get("/stable/stock-price-change", &[("symbol", symbol)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_quote_payloads_deserialize() {
        let quote: AftermarketQuoteDTO = serde_json::from_str(
            r#"{"symbol":"AAPL","bidSize":1,"bidPrice":200.1,"askSize":2,"askPrice":200.2,"volume":10,"timestamp":1}"#,
        )
        .unwrap();
        assert_eq!(quote.ask_price, Some(200.2));

        let changes: StockPriceChangeDTO =
            serde_json::from_str(r#"{"symbol":"AAPL","1D":1.2,"ytd":8.5,"max":100.0}"#).unwrap();
        assert_eq!(changes.day_1, Some(1.2));
        assert_eq!(changes.maximum, Some(100.0));
    }
}
