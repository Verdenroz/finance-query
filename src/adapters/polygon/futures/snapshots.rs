//! Current Massive futures contract snapshots.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::futures::FuturesQuote;

use super::super::{build_client, models::PaginatedResponseDTO};

/// Futures contract details included in a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesSnapshotDetailsDTO {
    /// Settlement timestamp in Unix nanoseconds.
    pub settlement_date: Option<i64>,
}

/// Latest one-minute aggregate included in a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesMinuteDTO {
    /// Close price.
    pub close: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Last update timestamp.
    pub last_updated: Option<i64>,
    /// Low price.
    pub low: Option<f64>,
    /// Open price.
    pub open: Option<f64>,
    /// Contract volume.
    pub volume: Option<u64>,
}

/// Latest futures quote included in a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesLastQuoteDTO {
    /// Ask price.
    pub ask: Option<f64>,
    /// Ask size.
    pub ask_size: Option<u64>,
    /// Ask timestamp.
    pub ask_timestamp: Option<i64>,
    /// Bid price.
    pub bid: Option<f64>,
    /// Bid size.
    pub bid_size: Option<u64>,
    /// Bid timestamp.
    pub bid_timestamp: Option<i64>,
    /// Last update timestamp.
    pub last_updated: Option<i64>,
}

/// Latest futures trade included in a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesLastTradeDTO {
    /// Last update timestamp.
    pub last_updated: Option<i64>,
    /// Trade price.
    pub price: Option<f64>,
    /// Trade size.
    pub size: Option<u64>,
}

/// Trading-session metrics included in a futures snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesSessionDTO {
    /// Change from previous settlement.
    pub change: Option<f64>,
    /// Fractional change from previous settlement.
    pub change_percent: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Open price.
    pub open: Option<f64>,
    /// Previous settlement price.
    pub previous_settlement: Option<f64>,
    /// Current settlement price.
    pub settlement_price: Option<f64>,
    /// Contract volume.
    pub volume: Option<u64>,
}

/// Snapshot for one futures contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesSnapshotDTO {
    /// Contract details.
    pub details: Option<FuturesSnapshotDetailsDTO>,
    /// Latest minute aggregate.
    pub last_minute: Option<FuturesMinuteDTO>,
    /// Latest quote.
    pub last_quote: Option<FuturesLastQuoteDTO>,
    /// Latest trade.
    pub last_trade: Option<FuturesLastTradeDTO>,
    /// Product code.
    pub product_code: Option<String>,
    /// Current session metrics.
    pub session: Option<FuturesSessionDTO>,
    /// Contract ticker.
    pub ticker: Option<String>,
}

/// Response wrapper for futures snapshots.
pub type FuturesSnapshotResponseDTO = PaginatedResponseDTO<FuturesSnapshotDTO>;

/// Fetch a snapshot for a futures contract ticker.
pub async fn futures_snapshot(ticker: &str) -> Result<FuturesSnapshotResponseDTO> {
    build_client()?
        .get("/futures/v1/snapshot", &[("ticker", ticker)])
        .await
}

/// Fetch a futures quote in finance-query's canonical representation.
pub async fn fetch_futures_quote_response(symbol: &str) -> Result<FuturesQuote> {
    Ok(snapshot_to_quote(symbol, futures_snapshot(symbol).await?))
}

fn snapshot_to_quote(symbol: &str, response: FuturesSnapshotResponseDTO) -> FuturesQuote {
    let snapshot = response
        .results
        .and_then(|results| results.into_iter().next());
    let session = snapshot.as_ref().and_then(|item| item.session.as_ref());
    let last_trade = snapshot.as_ref().and_then(|item| item.last_trade.as_ref());
    FuturesQuote {
        symbol: snapshot
            .as_ref()
            .and_then(|item| item.ticker.clone())
            .unwrap_or_else(|| symbol.to_string()),
        name: None,
        underlying: snapshot.as_ref().and_then(|item| item.product_code.clone()),
        exchange: None,
        expiration_date: None,
        price: last_trade
            .and_then(|trade| trade.price)
            .or_else(|| session.and_then(|value| value.close)),
        change: session.and_then(|value| value.change),
        change_percent: session
            .and_then(|value| value.change_percent)
            .map(|fraction| fraction * 100.0),
        open_interest: None,
        volume: session.and_then(|value| value.volume),
        timestamp: last_trade.and_then(|trade| trade.last_updated),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_snapshot_shape_maps_to_canonical_quote() {
        let response: FuturesSnapshotResponseDTO = serde_json::from_value(serde_json::json!({
            "status": "OK",
            "results": [{
                "ticker": "ESZ6",
                "product_code": "ES",
                "last_trade": {"price": 6052.0, "last_updated": 1786000000000000000_i64},
                "session": {"change": 12.0, "change_percent": 0.002, "volume": 1000}
            }]
        }))
        .unwrap();
        let quote = snapshot_to_quote("ESZ6", response);
        assert_eq!(quote.price, Some(6052.0));
        assert_eq!(quote.underlying.as_deref(), Some("ES"));
        assert_eq!(quote.volume, Some(1000));
        assert_eq!(quote.change, Some(12.0));
        assert!((quote.change_percent.unwrap() - 0.2).abs() < 1e-9);
    }
}
