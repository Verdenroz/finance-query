//! Polygon's unified cross-market snapshot (`GET /v3/snapshot`).
//!
//! One request covers stocks, options, FX, crypto, and indices — the
//! per-market snapshot endpoints each cost their own request and rate-limit
//! unit, which matters on the 5 req/sec free tier.

use serde::{Deserialize, Serialize};

use crate::adapters::polygon::build_client;
use crate::error::{FinanceError, Result};
use crate::models::quote::snapshot::{AssetClass, MarketSnapshot};

/// Polygon caps `ticker.any_of` at 250 symbols per request.
const MAX_SYMBOLS: usize = 250;

// ============================================================================
// Response types
// ============================================================================

/// `GET /v3/snapshot` envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UnifiedSnapshotResponseDTO {
    /// Snapshot rows, one per resolved (or failed) ticker.
    pub results: Option<Vec<UnifiedSnapshotDTO>>,
    /// Cursor URL for the next page.
    pub next_url: Option<String>,
    /// Request identifier.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
}

/// One row of a unified snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UnifiedSnapshotDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Instrument name.
    pub name: Option<String>,
    /// Asset class (`stocks`, `options`, `fx`, `crypto`, `indices`).
    #[serde(rename = "type")]
    pub asset_type: Option<String>,
    /// Market trading status.
    pub market_status: Option<String>,
    /// Most recent trade.
    pub last_trade: Option<UnifiedTradeDTO>,
    /// Most recent quote.
    pub last_quote: Option<UnifiedQuoteDTO>,
    /// Current trading session aggregates.
    pub session: Option<UnifiedSessionDTO>,
    /// Per-ticker error code (e.g. `"NOT_FOUND"`).
    pub error: Option<String>,
    /// Per-ticker error message.
    pub message: Option<String>,
}

/// Last-trade block of a unified snapshot row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UnifiedTradeDTO {
    /// Trade price.
    pub price: Option<f64>,
    /// Trade size.
    pub size: Option<f64>,
}

/// Last-quote block of a unified snapshot row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UnifiedQuoteDTO {
    /// Best bid.
    pub bid: Option<f64>,
    /// Best ask.
    pub ask: Option<f64>,
}

/// Session block of a unified snapshot row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UnifiedSessionDTO {
    /// Session open.
    pub open: Option<f64>,
    /// Session high.
    pub high: Option<f64>,
    /// Session low.
    pub low: Option<f64>,
    /// Session close.
    pub close: Option<f64>,
    /// Previous session close.
    pub previous_close: Option<f64>,
    /// Session volume.
    pub volume: Option<f64>,
    /// Absolute session change.
    pub change: Option<f64>,
    /// Percentage session change.
    pub change_percent: Option<f64>,
}

// ============================================================================
// Query function
// ============================================================================

/// Fetch a unified snapshot for a mixed list of tickers.
pub async fn unified_snapshot(tickers: &[&str]) -> Result<UnifiedSnapshotResponseDTO> {
    let joined = tickers.join(",");
    let limit = tickers.len().to_string();
    build_client()?
        .get_as(
            "/v3/snapshot",
            &[("ticker.any_of", &joined), ("limit", &limit)],
            "unified_snapshot",
            "unified snapshot response",
        )
        .await
}

// ============================================================================
// Canonical conversion
// ============================================================================

fn parse_asset_class(raw: Option<&str>) -> Option<AssetClass> {
    match raw? {
        "stocks" => Some(AssetClass::Stocks),
        "options" => Some(AssetClass::Options),
        "fx" => Some(AssetClass::Fx),
        "crypto" => Some(AssetClass::Crypto),
        "indices" => Some(AssetClass::Indices),
        _ => None,
    }
}

pub(crate) fn to_market_snapshot(dto: UnifiedSnapshotDTO) -> MarketSnapshot {
    let session = dto.session;
    MarketSnapshot {
        symbol: dto.ticker,
        name: dto.name,
        asset_class: parse_asset_class(dto.asset_type.as_deref()),
        market_status: dto.market_status,
        last_price: dto.last_trade.and_then(|t| t.price),
        bid: dto.last_quote.as_ref().and_then(|q| q.bid),
        ask: dto.last_quote.and_then(|q| q.ask),
        open: session.as_ref().and_then(|s| s.open),
        high: session.as_ref().and_then(|s| s.high),
        low: session.as_ref().and_then(|s| s.low),
        close: session.as_ref().and_then(|s| s.close),
        previous_close: session.as_ref().and_then(|s| s.previous_close),
        volume: session.as_ref().and_then(|s| s.volume),
        change: session.as_ref().and_then(|s| s.change),
        change_percent: session.and_then(|s| s.change_percent),
        error: dto.error,
        message: dto.message,
    }
}

/// Fetch canonical cross-market snapshots for a mixed list of symbols.
///
/// Rows for symbols Polygon could not resolve are returned with `error` set
/// rather than dropped, so the caller can tell "no data" from "not requested".
pub async fn fetch_unified_snapshot_response(symbols: &[&str]) -> Result<Vec<MarketSnapshot>> {
    if symbols.is_empty() {
        return Ok(Vec::new());
    }
    if symbols.len() > MAX_SYMBOLS {
        return Err(FinanceError::InvalidParameter {
            param: "symbols".to_string(),
            reason: format!(
                "Polygon's unified snapshot accepts at most {MAX_SYMBOLS} symbols per request, got {}",
                symbols.len()
            ),
        });
    }

    let resp = unified_snapshot(symbols).await?;
    Ok(resp
        .results
        .unwrap_or_default()
        .into_iter()
        .map(to_market_snapshot)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_a_stock_row_across_trade_quote_and_session() {
        let dto: UnifiedSnapshotDTO = serde_json::from_value(serde_json::json!({
            "ticker": "AAPL",
            "name": "Apple Inc.",
            "type": "stocks",
            "market_status": "closed",
            "last_trade": { "price": 227.5, "size": 2.0 },
            "last_quote": { "bid": 227.4, "ask": 227.6 },
            "session": {
                "open": 225.0,
                "high": 228.0,
                "low": 224.5,
                "close": 227.5,
                "previous_close": 226.0,
                "volume": 51_000_000.0,
                "change": 1.5,
                "change_percent": 0.66
            }
        }))
        .unwrap();

        let out = to_market_snapshot(dto);
        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.asset_class, Some(AssetClass::Stocks));
        assert_eq!(out.last_price, Some(227.5));
        assert_eq!(out.bid, Some(227.4));
        assert_eq!(out.ask, Some(227.6));
        assert_eq!(out.previous_close, Some(226.0));
        assert_eq!(out.change_percent, Some(0.66));
        assert!(out.error.is_none());
    }

    /// A failed lookup must survive as a row so callers can align results with
    /// their request list rather than guessing which symbol went missing.
    #[test]
    fn preserves_per_ticker_errors() {
        let dto: UnifiedSnapshotDTO = serde_json::from_value(serde_json::json!({
            "ticker": "TSLAAPL",
            "error": "NOT_FOUND",
            "message": "Ticker not found."
        }))
        .unwrap();

        let out = to_market_snapshot(dto);
        assert_eq!(out.symbol.as_deref(), Some("TSLAAPL"));
        assert_eq!(out.error.as_deref(), Some("NOT_FOUND"));
        assert_eq!(out.message.as_deref(), Some("Ticker not found."));
        assert!(out.last_price.is_none());
    }

    #[test]
    fn parses_every_asset_class_and_rejects_unknown() {
        assert_eq!(parse_asset_class(Some("stocks")), Some(AssetClass::Stocks));
        assert_eq!(
            parse_asset_class(Some("options")),
            Some(AssetClass::Options)
        );
        assert_eq!(parse_asset_class(Some("fx")), Some(AssetClass::Fx));
        assert_eq!(parse_asset_class(Some("crypto")), Some(AssetClass::Crypto));
        assert_eq!(
            parse_asset_class(Some("indices")),
            Some(AssetClass::Indices)
        );
        assert_eq!(parse_asset_class(Some("bonds")), None);
        assert_eq!(parse_asset_class(None), None);
    }

    #[tokio::test]
    async fn rejects_batches_over_polygons_cap_before_the_request() {
        let symbols: Vec<String> = (0..MAX_SYMBOLS + 1).map(|i| format!("T{i}")).collect();
        let refs: Vec<&str> = symbols.iter().map(String::as_str).collect();
        let err = fetch_unified_snapshot_response(&refs).await.unwrap_err();
        assert!(
            matches!(err, FinanceError::InvalidParameter { ref param, .. } if param == "symbols"),
            "got {err:?}"
        );
    }

    #[tokio::test]
    async fn empty_request_short_circuits_without_a_call() {
        assert!(
            fetch_unified_snapshot_response(&[])
                .await
                .unwrap()
                .is_empty()
        );
    }
}
