//! FMP share-float endpoint (`/stable/shares-float`).
//!
//! Lives under `fundamentals/` rather than `corporate/` because float routes
//! through `Capability::FUNDAMENTALS`, alongside the short-interest data it is
//! normally read with.

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::error::{FinanceError, Result};
use crate::models::fundamentals::ShareFloat;

/// Share float entry (`/stable/shares-float`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SharesFloatDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Free float as a percentage of shares outstanding.
    #[serde(rename = "freeFloat")]
    pub free_float: Option<f64>,
    /// Freely tradable shares.
    #[serde(rename = "floatShares")]
    pub float_shares: Option<f64>,
    /// Total shares outstanding.
    #[serde(rename = "outstandingShares")]
    pub outstanding_shares: Option<f64>,
    /// As-of timestamp, sometimes with a time component.
    pub date: Option<String>,
}

/// Fetch the share float for a symbol.
pub async fn shares_float(symbol: &str) -> Result<Vec<SharesFloatDTO>> {
    build_client()?
        .get("/stable/shares-float", &[("symbol", symbol)])
        .await
}

/// FMP dates on this endpoint arrive as `YYYY-MM-DD HH:MM:SS`; the model's
/// contract is a plain date, matching the other `ShareFloat` routes.
fn date_only(raw: Option<String>) -> Option<String> {
    raw.map(|d| match d.split_once([' ', 'T']) {
        Some((date, _)) => date.to_string(),
        None => d,
    })
}

pub(crate) fn to_share_float(dto: SharesFloatDTO, symbol: &str) -> ShareFloat {
    ShareFloat {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        float_shares: dto.float_shares,
        outstanding_shares: dto.outstanding_shares,
        float_percent: dto.free_float,
        date: date_only(dto.date),
    }
}

/// Fetch the canonical share float for a symbol.
pub async fn fetch_share_float_response(symbol: &str) -> Result<ShareFloat> {
    let dto = shares_float(symbol)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| FinanceError::SymbolNotFound {
            symbol: Some(symbol.to_string()),
            context: format!("FMP returned no share float for {symbol}"),
        })?;
    Ok(to_share_float(dto, symbol))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_float_maps_and_trims_the_timestamp() {
        let dto: SharesFloatDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "freeFloat": 99.89,
            "floatShares": 15_200_000_000.0,
            "outstandingShares": 15_300_000_000.0,
            "date": "2024-05-01 00:00:00"
        }))
        .unwrap();

        let out = to_share_float(dto, "AAPL");
        assert_eq!(out.float_shares, Some(15_200_000_000.0));
        assert_eq!(out.outstanding_shares, Some(15_300_000_000.0));
        assert_eq!(out.float_percent, Some(99.89));
        assert_eq!(out.date.as_deref(), Some("2024-05-01"));
    }

    #[test]
    fn date_only_passes_plain_dates_and_none_through() {
        assert_eq!(
            date_only(Some("2024-05-01".into())).as_deref(),
            Some("2024-05-01")
        );
        assert_eq!(
            date_only(Some("2024-05-01T12:00:00Z".into())).as_deref(),
            Some("2024-05-01")
        );
        assert_eq!(date_only(None), None);
    }
}
