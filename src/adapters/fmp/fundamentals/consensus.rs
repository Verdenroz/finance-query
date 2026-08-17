//! FMP aggregated analyst-consensus endpoints (price targets, rating rollup).
//!
//! These are the stable consensus endpoints — server-side rollups over the whole
//! analyst panel, as opposed to the raw per-analyst grade actions served by
//! [`estimates`](super::estimates).

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::{build_client, first_or_missing};
use crate::error::Result;
use crate::models::fundamentals::{PriceTargetConsensus, PriceTargetSummary, RatingConsensus};

// ============================================================================
// Response types
// ============================================================================

/// Consensus price target entry (`/stable/price-target-consensus`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceTargetConsensusDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Highest analyst target.
    #[serde(rename = "targetHigh")]
    pub target_high: Option<f64>,
    /// Lowest analyst target.
    #[serde(rename = "targetLow")]
    pub target_low: Option<f64>,
    /// Mean analyst target.
    #[serde(rename = "targetConsensus")]
    pub target_consensus: Option<f64>,
    /// Median analyst target.
    #[serde(rename = "targetMedian")]
    pub target_median: Option<f64>,
}

/// Price-target activity summary entry (`/stable/price-target-summary`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceTargetSummaryDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Targets published in the last month.
    #[serde(rename = "lastMonthCount")]
    pub last_month: Option<i64>,
    /// Average target published in the last month.
    #[serde(rename = "lastMonthAvgPriceTarget")]
    pub last_month_avg_price_target: Option<f64>,
    /// Targets published in the last quarter.
    #[serde(rename = "lastQuarterCount")]
    pub last_quarter: Option<i64>,
    /// Average target published in the last quarter.
    #[serde(rename = "lastQuarterAvgPriceTarget")]
    pub last_quarter_avg_price_target: Option<f64>,
    /// Targets published in the last year.
    #[serde(rename = "lastYearCount")]
    pub last_year: Option<i64>,
    /// Average target published in the last year.
    #[serde(rename = "lastYearAvgPriceTarget")]
    pub last_year_avg_price_target: Option<f64>,
    /// Targets published all time.
    #[serde(rename = "allTimeCount")]
    pub all_time: Option<i64>,
    /// Average target published all time.
    #[serde(rename = "allTimeAvgPriceTarget")]
    pub all_time_avg_price_target: Option<f64>,
}

/// Upgrades/downgrades consensus entry (`/stable/grades-consensus`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RatingConsensusDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Strong-buy ratings.
    #[serde(rename = "strongBuy")]
    pub strong_buy: Option<i64>,
    /// Buy ratings.
    pub buy: Option<i64>,
    /// Hold ratings.
    pub hold: Option<i64>,
    /// Sell ratings.
    pub sell: Option<i64>,
    /// Strong-sell ratings.
    #[serde(rename = "strongSell")]
    pub strong_sell: Option<i64>,
    /// Headline consensus label.
    pub consensus: Option<String>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Fetch the consensus price target for a symbol.
pub async fn price_target_consensus(symbol: &str) -> Result<Vec<PriceTargetConsensusDTO>> {
    build_client()?
        .get("/stable/price-target-consensus", &[("symbol", symbol)])
        .await
}

/// Fetch the price-target activity summary for a symbol.
pub async fn price_target_summary(symbol: &str) -> Result<Vec<PriceTargetSummaryDTO>> {
    build_client()?
        .get("/stable/price-target-summary", &[("symbol", symbol)])
        .await
}

/// Fetch the aggregated upgrades/downgrades consensus for a symbol.
pub async fn upgrades_downgrades_consensus(symbol: &str) -> Result<Vec<RatingConsensusDTO>> {
    build_client()?
        .get("/stable/grades-consensus", &[("symbol", symbol)])
        .await
}

// ============================================================================
// Canonical conversions
// ============================================================================

pub(crate) fn to_price_target_consensus(
    dto: PriceTargetConsensusDTO,
    symbol: &str,
) -> PriceTargetConsensus {
    PriceTargetConsensus {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        target_high: dto.target_high,
        target_low: dto.target_low,
        target_consensus: dto.target_consensus,
        target_median: dto.target_median,
    }
}

pub(crate) fn to_price_target_summary(
    dto: PriceTargetSummaryDTO,
    symbol: &str,
) -> PriceTargetSummary {
    PriceTargetSummary {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        last_month_count: dto.last_month,
        last_month_avg: dto.last_month_avg_price_target,
        last_quarter_count: dto.last_quarter,
        last_quarter_avg: dto.last_quarter_avg_price_target,
        last_year_count: dto.last_year,
        last_year_avg: dto.last_year_avg_price_target,
        all_time_count: dto.all_time,
        all_time_avg: dto.all_time_avg_price_target,
    }
}

pub(crate) fn to_rating_consensus(dto: RatingConsensusDTO, symbol: &str) -> RatingConsensus {
    RatingConsensus {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        strong_buy: dto.strong_buy,
        buy: dto.buy,
        hold: dto.hold,
        sell: dto.sell,
        strong_sell: dto.strong_sell,
        consensus: dto.consensus,
    }
}

/// Fetch the canonical consensus price target for a symbol.
pub async fn fetch_price_target_consensus_response(symbol: &str) -> Result<PriceTargetConsensus> {
    let rows = price_target_consensus(symbol).await?;
    let dto = first_or_missing(rows, symbol, "price target consensus")?;
    Ok(to_price_target_consensus(dto, symbol))
}

/// Fetch the canonical price-target activity summary for a symbol.
pub async fn fetch_price_target_summary_response(symbol: &str) -> Result<PriceTargetSummary> {
    let rows = price_target_summary(symbol).await?;
    let dto = first_or_missing(rows, symbol, "price target summary")?;
    Ok(to_price_target_summary(dto, symbol))
}

/// Fetch the canonical rating consensus for a symbol.
pub async fn fetch_rating_consensus_response(symbol: &str) -> Result<RatingConsensus> {
    let rows = upgrades_downgrades_consensus(symbol).await?;
    let dto = first_or_missing(rows, symbol, "rating consensus")?;
    Ok(to_rating_consensus(dto, symbol))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FinanceError;

    #[test]
    fn price_target_consensus_maps_every_field() {
        let dto: PriceTargetConsensusDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "targetHigh": 300.0,
            "targetLow": 150.0,
            "targetConsensus": 225.5,
            "targetMedian": 230.0
        }))
        .unwrap();

        let out = to_price_target_consensus(dto, "AAPL");
        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.target_high, Some(300.0));
        assert_eq!(out.target_low, Some(150.0));
        assert_eq!(out.target_consensus, Some(225.5));
        assert_eq!(out.target_median, Some(230.0));
    }

    #[test]
    fn price_target_summary_maps_every_window() {
        let dto: PriceTargetSummaryDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "lastMonthCount": 5,
            "lastMonthAvgPriceTarget": 220.5,
            "lastQuarterCount": 12,
            "lastQuarterAvgPriceTarget": 210.0,
            "lastYearCount": 45,
            "lastYearAvgPriceTarget": 200.0,
            "allTimeCount": 300,
            "allTimeAvgPriceTarget": 190.0,
            "publishers": "[\"StreetInsider\",\"TheFly\"]"
        }))
        .unwrap();

        let out = to_price_target_summary(dto, "AAPL");
        assert_eq!(out.last_month_count, Some(5));
        assert_eq!(out.last_month_avg, Some(220.5));
        assert_eq!(out.last_quarter_count, Some(12));
        assert_eq!(out.last_quarter_avg, Some(210.0));
        assert_eq!(out.last_year_count, Some(45));
        assert_eq!(out.last_year_avg, Some(200.0));
        assert_eq!(out.all_time_count, Some(300));
        assert_eq!(out.all_time_avg, Some(190.0));
    }

    #[test]
    fn rating_consensus_maps_grade_distribution() {
        let dto: RatingConsensusDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "strongBuy": 10,
            "buy": 20,
            "hold": 5,
            "sell": 1,
            "strongSell": 0,
            "consensus": "Buy"
        }))
        .unwrap();

        let out = to_rating_consensus(dto, "AAPL");
        assert_eq!(out.strong_buy, Some(10));
        assert_eq!(out.buy, Some(20));
        assert_eq!(out.hold, Some(5));
        assert_eq!(out.sell, Some(1));
        assert_eq!(out.strong_sell, Some(0));
        assert_eq!(out.consensus.as_deref(), Some("Buy"));
    }

    #[test]
    fn missing_symbol_falls_back_to_the_requested_one() {
        let dto: RatingConsensusDTO = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(
            to_rating_consensus(dto, "MSFT").symbol.as_deref(),
            Some("MSFT")
        );
    }

    #[test]
    fn empty_response_is_an_error_not_a_panic() {
        let err =
            first_or_missing::<RatingConsensusDTO>(vec![], "AAPL", "rating consensus").unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "got {err:?}"
        );
    }
}
