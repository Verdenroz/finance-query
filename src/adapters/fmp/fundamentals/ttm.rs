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

    fn key_metrics_payload() -> serde_json::Value {
        serde_json::from_str(
            r#"{
            "symbol": "AAPL",
            "marketCap": 3.4e12,
            "enterpriseValueTTM": 3.5e12,
            "evToSalesTTM": 8.8,
            "evToOperatingCashFlowTTM": 26.6,
            "evToFreeCashFlowTTM": 30.1,
            "evToEBITDATTM": 26.2,
            "netDebtToEBITDATTM": 0.36,
            "currentRatioTTM": 0.87,
            "incomeQualityTTM": 1.24,
            "grahamNumberTTM": 42.1,
            "grahamNetNetTTM": -12.4,
            "taxBurdenTTM": 0.76,
            "interestBurdenTTM": 0.99,
            "workingCapitalTTM": -1.1e10,
            "investedCapitalTTM": 1.6e11,
            "returnOnAssetsTTM": 0.29,
            "operatingReturnOnAssetsTTM": 0.37,
            "returnOnTangibleAssetsTTM": 0.3,
            "returnOnEquityTTM": 1.6,
            "returnOnInvestedCapitalTTM": 0.55,
            "returnOnCapitalEmployedTTM": 0.63,
            "earningsYieldTTM": 0.029,
            "freeCashFlowYieldTTM": 0.032,
            "capexToOperatingCashFlowTTM": -0.09,
            "capexToDepreciationTTM": -0.95,
            "capexToRevenueTTM": -0.028,
            "salesGeneralAndAdministrativeToRevenueTTM": 0.066,
            "researchAndDevelopementToRevenueTTM": 0.081,
            "stockBasedCompensationToRevenueTTM": 0.031,
            "intangiblesToTotalAssetsTTM": 0.0,
            "averageReceivablesTTM": 5.8e10,
            "averagePayablesTTM": 6.3e10,
            "averageInventoryTTM": 6.6e9,
            "daysOfSalesOutstandingTTM": 55.7,
            "daysOfPayablesOutstandingTTM": 106.7,
            "daysOfInventoryOutstandingTTM": 11.2,
            "operatingCycleTTM": 66.9,
            "cashConversionCycleTTM": -39.8,
            "freeCashFlowToEquityTTM": 8.9e10,
            "freeCashFlowToFirmTTM": 1.0e11,
            "tangibleAssetValueTTM": 5.7e10,
            "netCurrentAssetValueTTM": -1.4e11
        }"#,
        )
        .unwrap()
    }

    fn ratios_payload() -> serde_json::Value {
        serde_json::from_str(
            r#"{
            "symbol": "AAPL",
            "grossProfitMarginTTM": 0.462,
            "ebitMarginTTM": 0.317,
            "ebitdaMarginTTM": 0.345,
            "operatingProfitMarginTTM": 0.317,
            "pretaxProfitMarginTTM": 0.315,
            "continuousOperationsProfitMarginTTM": 0.239,
            "netProfitMarginTTM": 0.239,
            "bottomLineProfitMarginTTM": 0.239,
            "receivablesTurnoverTTM": 6.55,
            "payablesTurnoverTTM": 3.42,
            "inventoryTurnoverTTM": 32.6,
            "fixedAssetTurnoverTTM": 8.9,
            "assetTurnoverTTM": 1.19,
            "currentRatioTTM": 0.87,
            "quickRatioTTM": 0.83,
            "solvencyRatioTTM": 0.32,
            "cashRatioTTM": 0.21,
            "priceToEarningsRatioTTM": 34.12,
            "priceToEarningsGrowthRatioTTM": 2.4,
            "forwardPriceToEarningsGrowthRatioTTM": 2.1,
            "priceToEarningsDilutedRatioTTM": 34.3,
            "priceToEarningsDilutedGrowthRatioTTM": 2.42,
            "priceToBookRatioTTM": 54.8,
            "priceToSalesRatioTTM": 8.2,
            "priceToFreeCashFlowRatioTTM": 31.4,
            "priceToOperatingCashFlowRatioTTM": 28.6,
            "debtToAssetsRatioTTM": 0.29,
            "debtToEquityRatioTTM": 1.87,
            "debtToCapitalRatioTTM": 0.65,
            "longTermDebtToCapitalRatioTTM": 0.5,
            "financialLeverageRatioTTM": 6.4,
            "workingCapitalTurnoverRatioTTM": -35.2,
            "operatingCashFlowRatioTTM": 1.1,
            "operatingCashFlowSalesRatioTTM": 0.29,
            "freeCashFlowOperatingCashFlowRatioTTM": 0.91,
            "debtServiceCoverageRatioTTM": 4.6,
            "interestCoverageRatioTTM": 28.7,
            "shortTermOperatingCashFlowCoverageRatioTTM": 9.2,
            "operatingCashFlowCoverageRatioTTM": 1.02,
            "capitalExpenditureCoverageRatioTTM": 11.3,
            "dividendPaidAndCapexCoverageRatioTTM": 4.1,
            "dividendPayoutRatioTTM": 0.15,
            "dividendYieldTTM": 0.0044,
            "dividendPerShareTTM": 1.0,
            "enterpriseValueTTM": 3.5e12,
            "enterpriseValueMultipleTTM": 26.2,
            "revenuePerShareTTM": 25.5,
            "netIncomePerShareTTM": 6.5,
            "interestDebtPerShareTTM": 7.1,
            "cashPerShareTTM": 3.9,
            "bookValuePerShareTTM": 4.2,
            "tangibleBookValuePerShareTTM": 4.2,
            "shareholdersEquityPerShareTTM": 4.2,
            "operatingCashFlowPerShareTTM": 7.8,
            "capexPerShareTTM": -0.72,
            "freeCashFlowPerShareTTM": 7.1,
            "netIncomePerEBTTTM": 0.76,
            "ebtPerEbitTTM": 0.99,
            "priceToFairValueTTM": 54.8,
            "debtToMarketCapTTM": 0.03,
            "effectiveTaxRateTTM": 0.24
        }"#,
        )
        .unwrap()
    }

    /// Every key the stable endpoint documents must land on a field; a rename
    /// upstream would otherwise deserialize to `None` without erroring.
    fn assert_every_key_mapped(payload: &serde_json::Value, round_tripped: &serde_json::Value) {
        let mapped = round_tripped
            .as_object()
            .unwrap()
            .values()
            .filter(|v| !v.is_null())
            .count();
        assert_eq!(
            mapped,
            payload.as_object().unwrap().len(),
            "some stable keys did not map onto a field"
        );
    }

    #[test]
    fn key_metrics_ttm_maps_every_stable_key() {
        let payload = key_metrics_payload();
        let out: KeyMetricsTtm = serde_json::from_value(payload.clone()).unwrap();

        assert_eq!(out.market_cap, Some(3.4e12));
        assert_eq!(out.ev_to_ebitda, Some(26.2));
        assert_eq!(out.return_on_equity, Some(1.6));
        assert_eq!(out.return_on_invested_capital, Some(0.55));
        assert_eq!(out.return_on_assets, Some(0.29));
        assert_eq!(out.graham_number, Some(42.1));
        assert_eq!(out.research_and_development_to_revenue, Some(0.081));
        assert_eq!(out.cash_conversion_cycle, Some(-39.8));

        assert_every_key_mapped(&payload, &serde_json::to_value(&out).unwrap());
    }

    #[test]
    fn ratios_ttm_maps_every_stable_key() {
        let payload = ratios_payload();
        let out: FinancialRatiosTtm = serde_json::from_value(payload.clone()).unwrap();

        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.current_ratio, Some(0.87));
        assert_eq!(out.gross_profit_margin, Some(0.462));
        assert_eq!(out.price_earnings_ratio, Some(34.12));
        assert_eq!(out.peg_ratio, Some(2.4));
        assert_eq!(out.debt_ratio, Some(0.29));
        assert_eq!(out.payout_ratio, Some(0.15));
        assert_eq!(out.revenue_per_share, Some(25.5));
        assert_eq!(out.free_cash_flow_per_share, Some(7.1));

        assert_every_key_mapped(&payload, &serde_json::to_value(&out).unwrap());
    }

    #[test]
    fn ratios_ttm_accepts_fmps_misspelled_dividend_yield() {
        let misspelled: FinancialRatiosTtm = serde_json::from_value(serde_json::json!({
            "dividendYielTTM": 0.0044
        }))
        .unwrap();
        assert_eq!(misspelled.dividend_yield, Some(0.0044));
    }

    /// Aliases are deserialize-only, so the emitted JSON shape is unchanged.
    #[test]
    fn serialized_output_stays_snake_case() {
        let parsed: KeyMetricsTtm =
            serde_json::from_value(serde_json::json!({ "marketCap": 3.4e12 })).unwrap();
        let json = serde_json::to_value(&parsed).unwrap();
        assert!(json.get("market_cap").is_some());
        assert!(json.get("marketCap").is_none());
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
