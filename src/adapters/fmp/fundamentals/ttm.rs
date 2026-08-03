//! FMP trailing-twelve-month snapshot endpoints (`key-metrics-ttm`, `ratios-ttm`).
//!
//! Distinct from the period-based endpoints in [`analysis`](super::analysis):
//! these return one always-current rollup rather than a fiscal-period series,
//! and FMP handles partial periods and restatements server-side.

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::adapters::fmp::build_client;
use crate::error::{FinanceError, Result};
use crate::models::fundamentals::{FinancialRatiosTtm, KeyMetricsTtm};

// ============================================================================
// Response types
// ============================================================================

/// TTM key-metrics entry (`/api/v3/key-metrics-ttm/{symbol}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyMetricsTtmDTO {
    /// Ticker symbol (absent on some FMP plans).
    pub symbol: Option<String>,
    /// Revenue per share.
    #[serde(rename = "revenuePerShareTTM")]
    pub revenue_per_share: Option<f64>,
    /// Net income per share.
    #[serde(rename = "netIncomePerShareTTM")]
    pub net_income_per_share: Option<f64>,
    /// Operating cash flow per share.
    #[serde(rename = "operatingCashFlowPerShareTTM")]
    pub operating_cash_flow_per_share: Option<f64>,
    /// Free cash flow per share.
    #[serde(rename = "freeCashFlowPerShareTTM")]
    pub free_cash_flow_per_share: Option<f64>,
    /// Cash per share.
    #[serde(rename = "cashPerShareTTM")]
    pub cash_per_share: Option<f64>,
    /// Book value per share.
    #[serde(rename = "bookValuePerShareTTM")]
    pub book_value_per_share: Option<f64>,
    /// Tangible book value per share.
    #[serde(rename = "tangibleBookValuePerShareTTM")]
    pub tangible_book_value_per_share: Option<f64>,
    /// Shareholders' equity per share.
    #[serde(rename = "shareholdersEquityPerShareTTM")]
    pub shareholders_equity_per_share: Option<f64>,
    /// Interest-bearing debt per share.
    #[serde(rename = "interestDebtPerShareTTM")]
    pub interest_debt_per_share: Option<f64>,
    /// Market capitalization.
    #[serde(rename = "marketCapTTM")]
    pub market_cap: Option<f64>,
    /// Enterprise value.
    #[serde(rename = "enterpriseValueTTM")]
    pub enterprise_value: Option<f64>,
    /// PE ratio.
    #[serde(rename = "peRatioTTM")]
    pub pe_ratio: Option<f64>,
    /// PB ratio.
    #[serde(rename = "pbRatioTTM")]
    pub pb_ratio: Option<f64>,
    /// EV to sales.
    #[serde(rename = "evToSalesTTM")]
    pub ev_to_sales: Option<f64>,
    /// EV to EBITDA.
    #[serde(rename = "enterpriseValueOverEBITDATTM")]
    pub enterprise_value_over_ebitda: Option<f64>,
    /// EV to operating cash flow.
    #[serde(rename = "evToOperatingCashFlowTTM")]
    pub ev_to_operating_cash_flow: Option<f64>,
    /// EV to free cash flow.
    #[serde(rename = "evToFreeCashFlowTTM")]
    pub ev_to_free_cash_flow: Option<f64>,
    /// Earnings yield.
    #[serde(rename = "earningsYieldTTM")]
    pub earnings_yield: Option<f64>,
    /// Free cash flow yield.
    #[serde(rename = "freeCashFlowYieldTTM")]
    pub free_cash_flow_yield: Option<f64>,
    /// Debt to equity.
    #[serde(rename = "debtToEquityTTM")]
    pub debt_to_equity: Option<f64>,
    /// Debt to assets.
    #[serde(rename = "debtToAssetsTTM")]
    pub debt_to_assets: Option<f64>,
    /// Net debt to EBITDA.
    #[serde(rename = "netDebtToEBITDATTM")]
    pub net_debt_to_ebitda: Option<f64>,
    /// Current ratio.
    #[serde(rename = "currentRatioTTM")]
    pub current_ratio: Option<f64>,
    /// Interest coverage.
    #[serde(rename = "interestCoverageTTM")]
    pub interest_coverage: Option<f64>,
    /// Return on equity.
    #[serde(rename = "roeTTM")]
    pub roe: Option<f64>,
    /// Return on invested capital.
    #[serde(rename = "roicTTM")]
    pub roic: Option<f64>,
    /// Dividend yield.
    #[serde(rename = "dividendYieldTTM")]
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    #[serde(rename = "payoutRatioTTM")]
    pub payout_ratio: Option<f64>,
    /// Graham number.
    #[serde(rename = "grahamNumberTTM")]
    pub graham_number: Option<f64>,
}

/// TTM ratios entry (`/api/v3/ratios-ttm/{symbol}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialRatiosTtmDTO {
    /// Ticker symbol (absent on some FMP plans).
    pub symbol: Option<String>,
    /// Current ratio.
    #[serde(rename = "currentRatioTTM")]
    pub current_ratio: Option<f64>,
    /// Quick ratio.
    #[serde(rename = "quickRatioTTM")]
    pub quick_ratio: Option<f64>,
    /// Cash ratio.
    #[serde(rename = "cashRatioTTM")]
    pub cash_ratio: Option<f64>,
    /// Gross profit margin.
    #[serde(rename = "grossProfitMarginTTM")]
    pub gross_profit_margin: Option<f64>,
    /// Operating profit margin.
    #[serde(rename = "operatingProfitMarginTTM")]
    pub operating_profit_margin: Option<f64>,
    /// Net profit margin.
    #[serde(rename = "netProfitMarginTTM")]
    pub net_profit_margin: Option<f64>,
    /// Return on assets.
    #[serde(rename = "returnOnAssetsTTM")]
    pub return_on_assets: Option<f64>,
    /// Return on equity.
    #[serde(rename = "returnOnEquityTTM")]
    pub return_on_equity: Option<f64>,
    /// Return on capital employed.
    #[serde(rename = "returnOnCapitalEmployedTTM")]
    pub return_on_capital_employed: Option<f64>,
    /// Debt ratio.
    #[serde(rename = "debtRatioTTM")]
    pub debt_ratio: Option<f64>,
    /// Debt-to-equity ratio.
    #[serde(rename = "debtEquityRatioTTM")]
    pub debt_equity_ratio: Option<f64>,
    /// Interest coverage.
    #[serde(rename = "interestCoverageTTM")]
    pub interest_coverage: Option<f64>,
    /// Price-to-earnings ratio.
    #[serde(rename = "priceEarningsRatioTTM")]
    pub price_earnings_ratio: Option<f64>,
    /// Price/earnings-to-growth ratio.
    #[serde(rename = "pegRatioTTM")]
    pub peg_ratio: Option<f64>,
    /// Price-to-book ratio.
    #[serde(rename = "priceToBookRatioTTM")]
    pub price_to_book_ratio: Option<f64>,
    /// Price-to-sales ratio.
    #[serde(rename = "priceToSalesRatioTTM")]
    pub price_to_sales_ratio: Option<f64>,
    /// Price-to-free-cash-flow ratio.
    #[serde(rename = "priceToFreeCashFlowsRatioTTM")]
    pub price_to_free_cash_flows_ratio: Option<f64>,
    /// Enterprise value multiple.
    #[serde(rename = "enterpriseValueMultipleTTM")]
    pub enterprise_value_multiple: Option<f64>,
    /// Dividend yield. FMP spells the canonical key `dividendYielTTM` (sic) on
    /// this endpoint; the alias covers plans that return the corrected spelling.
    #[serde(rename = "dividendYielTTM", alias = "dividendYieldTTM")]
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    #[serde(rename = "payoutRatioTTM")]
    pub payout_ratio: Option<f64>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Fetch the TTM key-metrics snapshot for a symbol.
pub async fn key_metrics_ttm(symbol: &str) -> Result<Vec<KeyMetricsTtmDTO>> {
    build_client()?
        .get(
            &format!("/api/v3/key-metrics-ttm/{}", encode_path_segment(symbol)),
            &[],
        )
        .await
}

/// Fetch the TTM ratios snapshot for a symbol.
pub async fn ratios_ttm(symbol: &str) -> Result<Vec<FinancialRatiosTtmDTO>> {
    build_client()?
        .get(
            &format!("/api/v3/ratios-ttm/{}", encode_path_segment(symbol)),
            &[],
        )
        .await
}

// ============================================================================
// Canonical conversions
// ============================================================================

fn first_or_missing<T>(rows: Vec<T>, symbol: &str, field: &str) -> Result<T> {
    rows.into_iter()
        .next()
        .ok_or_else(|| FinanceError::SymbolNotFound {
            symbol: Some(symbol.to_string()),
            context: format!("FMP returned no {field} for {symbol}"),
        })
}

pub(crate) fn to_key_metrics_ttm(dto: KeyMetricsTtmDTO, symbol: &str) -> KeyMetricsTtm {
    KeyMetricsTtm {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        revenue_per_share: dto.revenue_per_share,
        net_income_per_share: dto.net_income_per_share,
        operating_cash_flow_per_share: dto.operating_cash_flow_per_share,
        free_cash_flow_per_share: dto.free_cash_flow_per_share,
        cash_per_share: dto.cash_per_share,
        book_value_per_share: dto.book_value_per_share,
        tangible_book_value_per_share: dto.tangible_book_value_per_share,
        shareholders_equity_per_share: dto.shareholders_equity_per_share,
        interest_debt_per_share: dto.interest_debt_per_share,
        market_cap: dto.market_cap,
        enterprise_value: dto.enterprise_value,
        pe_ratio: dto.pe_ratio,
        pb_ratio: dto.pb_ratio,
        ev_to_sales: dto.ev_to_sales,
        ev_to_ebitda: dto.enterprise_value_over_ebitda,
        ev_to_operating_cash_flow: dto.ev_to_operating_cash_flow,
        ev_to_free_cash_flow: dto.ev_to_free_cash_flow,
        earnings_yield: dto.earnings_yield,
        free_cash_flow_yield: dto.free_cash_flow_yield,
        debt_to_equity: dto.debt_to_equity,
        debt_to_assets: dto.debt_to_assets,
        net_debt_to_ebitda: dto.net_debt_to_ebitda,
        current_ratio: dto.current_ratio,
        interest_coverage: dto.interest_coverage,
        return_on_equity: dto.roe,
        return_on_invested_capital: dto.roic,
        dividend_yield: dto.dividend_yield,
        payout_ratio: dto.payout_ratio,
        graham_number: dto.graham_number,
    }
}

pub(crate) fn to_financial_ratios_ttm(
    dto: FinancialRatiosTtmDTO,
    symbol: &str,
) -> FinancialRatiosTtm {
    FinancialRatiosTtm {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        current_ratio: dto.current_ratio,
        quick_ratio: dto.quick_ratio,
        cash_ratio: dto.cash_ratio,
        gross_profit_margin: dto.gross_profit_margin,
        operating_profit_margin: dto.operating_profit_margin,
        net_profit_margin: dto.net_profit_margin,
        return_on_assets: dto.return_on_assets,
        return_on_equity: dto.return_on_equity,
        return_on_capital_employed: dto.return_on_capital_employed,
        debt_ratio: dto.debt_ratio,
        debt_equity_ratio: dto.debt_equity_ratio,
        interest_coverage: dto.interest_coverage,
        price_earnings_ratio: dto.price_earnings_ratio,
        peg_ratio: dto.peg_ratio,
        price_to_book_ratio: dto.price_to_book_ratio,
        price_to_sales_ratio: dto.price_to_sales_ratio,
        price_to_free_cash_flows_ratio: dto.price_to_free_cash_flows_ratio,
        enterprise_value_multiple: dto.enterprise_value_multiple,
        dividend_yield: dto.dividend_yield,
        payout_ratio: dto.payout_ratio,
    }
}

/// Fetch the canonical TTM key-metrics snapshot for a symbol.
pub async fn fetch_key_metrics_ttm_response(symbol: &str) -> Result<KeyMetricsTtm> {
    let rows = key_metrics_ttm(symbol).await?;
    let dto = first_or_missing(rows, symbol, "TTM key metrics")?;
    Ok(to_key_metrics_ttm(dto, symbol))
}

/// Fetch the canonical TTM ratios snapshot for a symbol.
pub async fn fetch_ratios_ttm_response(symbol: &str) -> Result<FinancialRatiosTtm> {
    let rows = ratios_ttm(symbol).await?;
    let dto = first_or_missing(rows, symbol, "TTM ratios")?;
    Ok(to_financial_ratios_ttm(dto, symbol))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_metrics_ttm_maps_suffixed_keys() {
        let dto: KeyMetricsTtmDTO = serde_json::from_value(serde_json::json!({
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

        let out = to_key_metrics_ttm(dto, "AAPL");
        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.revenue_per_share, Some(25.5));
        assert_eq!(out.market_cap, Some(3.4e12));
        assert_eq!(out.pe_ratio, Some(34.1));
        // `enterpriseValueOverEBITDATTM` is renamed to the shorter public name.
        assert_eq!(out.ev_to_ebitda, Some(26.2));
        assert_eq!(out.return_on_equity, Some(1.6));
        assert_eq!(out.return_on_invested_capital, Some(0.55));
        assert_eq!(out.graham_number, Some(42.1));
    }

    #[test]
    fn ratios_ttm_accepts_fmps_misspelled_dividend_yield() {
        let misspelled: FinancialRatiosTtmDTO = serde_json::from_value(serde_json::json!({
            "dividendYielTTM": 0.0044
        }))
        .unwrap();
        assert_eq!(misspelled.dividend_yield, Some(0.0044));

        let corrected: FinancialRatiosTtmDTO = serde_json::from_value(serde_json::json!({
            "dividendYieldTTM": 0.0051
        }))
        .unwrap();
        assert_eq!(corrected.dividend_yield, Some(0.0051));
    }

    #[test]
    fn ratios_ttm_maps_margins_and_multiples() {
        let dto: FinancialRatiosTtmDTO = serde_json::from_value(serde_json::json!({
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

        let out = to_financial_ratios_ttm(dto, "MSFT");
        // Provider-supplied symbol wins over the requested one.
        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.current_ratio, Some(0.87));
        assert_eq!(out.gross_profit_margin, Some(0.462));
        assert_eq!(out.net_profit_margin, Some(0.239));
        assert_eq!(out.price_earnings_ratio, Some(34.12));
        assert_eq!(out.peg_ratio, Some(2.4));
        assert_eq!(out.enterprise_value_multiple, Some(26.2));
        assert_eq!(out.payout_ratio, Some(0.15));
    }

    #[test]
    fn empty_response_is_an_error_not_a_panic() {
        let err =
            first_or_missing::<KeyMetricsTtmDTO>(vec![], "AAPL", "TTM key metrics").unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "got {err:?}"
        );
    }
}
