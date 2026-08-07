//! FMP trailing-twelve-month snapshot endpoints (`key-metrics-ttm`, `ratios-ttm`).
//!
//! Distinct from the period-based endpoints in [`analysis`](super::analysis):
//! these return one always-current rollup rather than a fiscal-period series,
//! and FMP handles partial periods and restatements server-side.
//!
//! FMP's `*TTM`-suffixed keys deserialize straight into the public models via
//! their `serde` aliases — see [`crate::models::fundamentals::ttm`].

use crate::adapters::fmp::{build_client, first_or_missing};
use crate::error::Result;
use crate::models::fundamentals::{FinancialRatiosTtm, KeyMetricsTtm};

// ============================================================================
// Query functions
// ============================================================================

/// Fetch the TTM key-metrics snapshot for a symbol.
pub async fn key_metrics_ttm(symbol: &str) -> Result<Vec<KeyMetricsTtm>> {
    build_client()?
        .get("/stable/key-metrics-ttm", &[("symbol", symbol)])
        .await
}

/// Fetch the TTM ratios snapshot for a symbol.
pub async fn ratios_ttm(symbol: &str) -> Result<Vec<FinancialRatiosTtm>> {
    build_client()?
        .get("/stable/ratios-ttm", &[("symbol", symbol)])
        .await
}

// ============================================================================
// Canonical responses
// ============================================================================

/// Fetch the canonical TTM key-metrics snapshot for a symbol.
pub async fn fetch_key_metrics_ttm_response(symbol: &str) -> Result<KeyMetricsTtm> {
    let rows = key_metrics_ttm(symbol).await?;
    let mut metrics = first_or_missing(rows, symbol, "TTM key metrics")?;
    metrics.symbol = metrics.symbol.or_else(|| Some(symbol.to_string()));
    Ok(metrics)
}

/// Fetch the canonical TTM ratios snapshot for a symbol.
pub async fn fetch_ratios_ttm_response(symbol: &str) -> Result<FinancialRatiosTtm> {
    let rows = ratios_ttm(symbol).await?;
    let mut ratios = first_or_missing(rows, symbol, "TTM ratios")?;
    ratios.symbol = ratios.symbol.or_else(|| Some(symbol.to_string()));
    Ok(ratios)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FinanceError;

    #[test]
    fn key_metrics_ttm_maps_suffixed_keys() {
        let out: KeyMetricsTtm = serde_json::from_value(serde_json::json!({
            "revenuePerShareTTM": 25.5,
            "netIncomePerShareTTM": 6.5,
            "marketCapTTM": 3.4e12,
            "peRatioTTM": 34.1,
            "enterpriseValueOverEBITDATTM": 26.2,
            "roeTTM": 1.6,
            "roicTTM": 0.55,
            "dividendYieldTTM": 0.0044,
            "grahamNumberTTM": 42.1
        }))
        .unwrap();

        assert_eq!(out.revenue_per_share, Some(25.5));
        assert_eq!(out.market_cap, Some(3.4e12));
        assert_eq!(out.pe_ratio, Some(34.1));
        // `enterpriseValueOverEBITDATTM` is aliased to the shorter public name.
        assert_eq!(out.ev_to_ebitda, Some(26.2));
        assert_eq!(out.return_on_equity, Some(1.6));
        assert_eq!(out.return_on_invested_capital, Some(0.55));
        assert_eq!(out.graham_number, Some(42.1));
    }

    #[test]
    fn ratios_ttm_accepts_fmps_misspelled_dividend_yield() {
        let misspelled: FinancialRatiosTtm = serde_json::from_value(serde_json::json!({
            "dividendYielTTM": 0.0044
        }))
        .unwrap();
        assert_eq!(misspelled.dividend_yield, Some(0.0044));

        let corrected: FinancialRatiosTtm = serde_json::from_value(serde_json::json!({
            "dividendYieldTTM": 0.0051
        }))
        .unwrap();
        assert_eq!(corrected.dividend_yield, Some(0.0051));
    }

    #[test]
    fn ratios_ttm_maps_margins_and_multiples() {
        let out: FinancialRatiosTtm = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "currentRatioTTM": 0.87,
            "grossProfitMarginTTM": 0.462,
            "netProfitMarginTTM": 0.239,
            "priceEarningsRatioTTM": 34.12,
            "pegRatioTTM": 2.4,
            "enterpriseValueMultipleTTM": 26.2,
            "payoutRatioTTM": 0.15
        }))
        .unwrap();

        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.current_ratio, Some(0.87));
        assert_eq!(out.gross_profit_margin, Some(0.462));
        assert_eq!(out.net_profit_margin, Some(0.239));
        assert_eq!(out.price_earnings_ratio, Some(34.12));
        assert_eq!(out.peg_ratio, Some(2.4));
        assert_eq!(out.enterprise_value_multiple, Some(26.2));
        assert_eq!(out.payout_ratio, Some(0.15));
    }

    /// Aliases are deserialize-only, so the emitted JSON shape is unchanged.
    #[test]
    fn serialized_output_stays_snake_case() {
        let parsed: KeyMetricsTtm =
            serde_json::from_value(serde_json::json!({ "marketCapTTM": 3.4e12 })).unwrap();
        let json = serde_json::to_value(&parsed).unwrap();
        assert!(json.get("market_cap").is_some());
        assert!(json.get("marketCapTTM").is_none());
    }

    #[test]
    fn empty_response_is_an_error_not_a_panic() {
        let err = first_or_missing::<KeyMetricsTtm>(vec![], "AAPL", "TTM key metrics").unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "got {err:?}"
        );
    }
}
