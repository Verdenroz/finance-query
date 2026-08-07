//! Current Massive futures trade and quote endpoints.

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use super::super::{build_client, models::PaginatedResponseDTO};

/// One futures trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesTradeDTO {
    /// CME multicast channel.
    pub channel: Option<i64>,
    /// Trade price.
    pub price: Option<f64>,
    /// Exchange reporting sequence.
    pub report_sequence: Option<i64>,
    /// Event sequence number.
    pub sequence_number: Option<i64>,
    /// Trading-session end date.
    pub session_end_date: Option<String>,
    /// Number of contracts traded.
    pub size: Option<f64>,
    /// Contract ticker.
    pub ticker: Option<String>,
    /// Exchange timestamp in Unix nanoseconds.
    pub timestamp: Option<i64>,
}

/// One futures best-bid-and-offer quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesQuoteDTO {
    /// Ask price.
    pub ask_price: Option<f64>,
    /// Ask size in contracts.
    pub ask_size: Option<f64>,
    /// Ask timestamp in Unix nanoseconds.
    pub ask_timestamp: Option<i64>,
    /// Bid price.
    pub bid_price: Option<f64>,
    /// Bid size in contracts.
    pub bid_size: Option<f64>,
    /// Bid timestamp in Unix nanoseconds.
    pub bid_timestamp: Option<i64>,
    /// CME multicast channel.
    pub channel: Option<i64>,
    /// Exchange reporting sequence.
    pub report_sequence: Option<i64>,
    /// Event sequence number.
    pub sequence_number: Option<i64>,
    /// Trading-session end date.
    pub session_end_date: Option<String>,
    /// Contract ticker.
    pub ticker: Option<String>,
    /// Quote timestamp in Unix nanoseconds.
    pub timestamp: Option<i64>,
}

/// Fetch tick-level futures trades.
pub async fn futures_trades(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesTradeDTO>> {
    let path = format!("/futures/v1/trades/{}", encode_path_segment(ticker));
    build_client()?.get(&path, params).await
}

/// Fetch tick-level futures quotes.
pub async fn futures_quotes(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FuturesQuoteDTO>> {
    let path = format!("/futures/v1/quotes/{}", encode_path_segment(ticker));
    build_client()?.get(&path, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_trade_and_quote_shapes_deserialize() {
        let trade: FuturesTradeDTO = serde_json::from_value(serde_json::json!({
            "channel": 1,
            "price": 6052.0,
            "report_sequence": 2,
            "sequence_number": 3,
            "session_end_date": "2026-08-06",
            "size": 4,
            "ticker": "ESZ6",
            "timestamp": 1786000000000000000_i64
        }))
        .unwrap();
        assert_eq!(trade.ticker.as_deref(), Some("ESZ6"));

        let quote: FuturesQuoteDTO = serde_json::from_value(serde_json::json!({
            "ask_price": 6052.25,
            "bid_price": 6052.0,
            "ticker": "ESZ6",
            "timestamp": 1786000000000000000_i64
        }))
        .unwrap();
        assert_eq!(quote.ask_price, Some(6052.25));
    }
}
