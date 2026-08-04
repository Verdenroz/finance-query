//! Trailing-twelve-month (TTM) fundamentals snapshots.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; FMP is currently the only provider. A TTM snapshot is a single
//! always-current rollup — unlike [`FinancialStatement`](super::FinancialStatement),
//! it is not indexed by fiscal period, so callers do not need to know whether
//! the latest filed period is still current.
//!
//! The `TTM`-suffixed `serde` aliases let the provider's wire shape deserialize
//! straight into these types instead of through a field-identical DTO; aliases
//! are deserialize-only, so serialized output stays snake_case.

use serde::{Deserialize, Serialize};

/// Per-share, valuation, and leverage metrics over the trailing twelve months.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyMetricsTtm {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Revenue per share.
    #[serde(alias = "revenuePerShareTTM")]
    pub revenue_per_share: Option<f64>,
    /// Net income per share.
    #[serde(alias = "netIncomePerShareTTM")]
    pub net_income_per_share: Option<f64>,
    /// Operating cash flow per share.
    #[serde(alias = "operatingCashFlowPerShareTTM")]
    pub operating_cash_flow_per_share: Option<f64>,
    /// Free cash flow per share.
    #[serde(alias = "freeCashFlowPerShareTTM")]
    pub free_cash_flow_per_share: Option<f64>,
    /// Cash per share.
    #[serde(alias = "cashPerShareTTM")]
    pub cash_per_share: Option<f64>,
    /// Book value per share.
    #[serde(alias = "bookValuePerShareTTM")]
    pub book_value_per_share: Option<f64>,
    /// Tangible book value per share.
    #[serde(alias = "tangibleBookValuePerShareTTM")]
    pub tangible_book_value_per_share: Option<f64>,
    /// Shareholders' equity per share.
    #[serde(alias = "shareholdersEquityPerShareTTM")]
    pub shareholders_equity_per_share: Option<f64>,
    /// Interest-bearing debt per share.
    #[serde(alias = "interestDebtPerShareTTM")]
    pub interest_debt_per_share: Option<f64>,
    /// Market capitalization.
    #[serde(alias = "marketCapTTM")]
    pub market_cap: Option<f64>,
    /// Enterprise value.
    #[serde(alias = "enterpriseValueTTM")]
    pub enterprise_value: Option<f64>,
    /// Price-to-earnings ratio.
    #[serde(alias = "peRatioTTM")]
    pub pe_ratio: Option<f64>,
    /// Price-to-book ratio.
    #[serde(alias = "pbRatioTTM")]
    pub pb_ratio: Option<f64>,
    /// Enterprise value to sales.
    #[serde(alias = "evToSalesTTM")]
    pub ev_to_sales: Option<f64>,
    /// Enterprise value to EBITDA.
    #[serde(alias = "enterpriseValueOverEBITDATTM")]
    pub ev_to_ebitda: Option<f64>,
    /// Enterprise value to operating cash flow.
    #[serde(alias = "evToOperatingCashFlowTTM")]
    pub ev_to_operating_cash_flow: Option<f64>,
    /// Enterprise value to free cash flow.
    #[serde(alias = "evToFreeCashFlowTTM")]
    pub ev_to_free_cash_flow: Option<f64>,
    /// Earnings yield.
    #[serde(alias = "earningsYieldTTM")]
    pub earnings_yield: Option<f64>,
    /// Free cash flow yield.
    #[serde(alias = "freeCashFlowYieldTTM")]
    pub free_cash_flow_yield: Option<f64>,
    /// Debt to equity.
    #[serde(alias = "debtToEquityTTM")]
    pub debt_to_equity: Option<f64>,
    /// Debt to assets.
    #[serde(alias = "debtToAssetsTTM")]
    pub debt_to_assets: Option<f64>,
    /// Net debt to EBITDA.
    #[serde(alias = "netDebtToEBITDATTM")]
    pub net_debt_to_ebitda: Option<f64>,
    /// Current ratio.
    #[serde(alias = "currentRatioTTM")]
    pub current_ratio: Option<f64>,
    /// Interest coverage.
    #[serde(alias = "interestCoverageTTM")]
    pub interest_coverage: Option<f64>,
    /// Return on equity.
    #[serde(alias = "roeTTM")]
    pub return_on_equity: Option<f64>,
    /// Return on invested capital.
    #[serde(alias = "roicTTM")]
    pub return_on_invested_capital: Option<f64>,
    /// Dividend yield (fraction, not percent).
    #[serde(alias = "dividendYieldTTM")]
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    #[serde(alias = "payoutRatioTTM")]
    pub payout_ratio: Option<f64>,
    /// Graham number.
    #[serde(alias = "grahamNumberTTM")]
    pub graham_number: Option<f64>,
}

/// Liquidity, margin, return, and valuation ratios over the trailing twelve
/// months.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialRatiosTtm {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Current ratio.
    #[serde(alias = "currentRatioTTM")]
    pub current_ratio: Option<f64>,
    /// Quick ratio.
    #[serde(alias = "quickRatioTTM")]
    pub quick_ratio: Option<f64>,
    /// Cash ratio.
    #[serde(alias = "cashRatioTTM")]
    pub cash_ratio: Option<f64>,
    /// Gross profit margin.
    #[serde(alias = "grossProfitMarginTTM")]
    pub gross_profit_margin: Option<f64>,
    /// Operating profit margin.
    #[serde(alias = "operatingProfitMarginTTM")]
    pub operating_profit_margin: Option<f64>,
    /// Net profit margin.
    #[serde(alias = "netProfitMarginTTM")]
    pub net_profit_margin: Option<f64>,
    /// Return on assets.
    #[serde(alias = "returnOnAssetsTTM")]
    pub return_on_assets: Option<f64>,
    /// Return on equity.
    #[serde(alias = "returnOnEquityTTM")]
    pub return_on_equity: Option<f64>,
    /// Return on capital employed.
    #[serde(alias = "returnOnCapitalEmployedTTM")]
    pub return_on_capital_employed: Option<f64>,
    /// Debt ratio.
    #[serde(alias = "debtRatioTTM")]
    pub debt_ratio: Option<f64>,
    /// Debt-to-equity ratio.
    #[serde(alias = "debtEquityRatioTTM")]
    pub debt_equity_ratio: Option<f64>,
    /// Interest coverage.
    #[serde(alias = "interestCoverageTTM")]
    pub interest_coverage: Option<f64>,
    /// Price-to-earnings ratio.
    #[serde(alias = "priceEarningsRatioTTM")]
    pub price_earnings_ratio: Option<f64>,
    /// Price/earnings-to-growth ratio.
    #[serde(alias = "pegRatioTTM")]
    pub peg_ratio: Option<f64>,
    /// Price-to-book ratio.
    #[serde(alias = "priceToBookRatioTTM")]
    pub price_to_book_ratio: Option<f64>,
    /// Price-to-sales ratio.
    #[serde(alias = "priceToSalesRatioTTM")]
    pub price_to_sales_ratio: Option<f64>,
    /// Price-to-free-cash-flow ratio.
    #[serde(alias = "priceToFreeCashFlowsRatioTTM")]
    pub price_to_free_cash_flows_ratio: Option<f64>,
    /// Enterprise value multiple (EV/EBITDA).
    #[serde(alias = "enterpriseValueMultipleTTM")]
    pub enterprise_value_multiple: Option<f64>,
    /// Dividend yield (fraction, not percent).
    #[serde(alias = "dividendYielTTM", alias = "dividendYieldTTM")]
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    #[serde(alias = "payoutRatioTTM")]
    pub payout_ratio: Option<f64>,
}
