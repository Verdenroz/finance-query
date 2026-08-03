//! Trailing-twelve-month (TTM) fundamentals snapshots.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; FMP is currently the only provider. A TTM snapshot is a single
//! always-current rollup — unlike [`FinancialStatement`](super::FinancialStatement),
//! it is not indexed by fiscal period, so callers do not need to know whether
//! the latest filed period is still current.

use serde::{Deserialize, Serialize};

/// Per-share, valuation, and leverage metrics over the trailing twelve months.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyMetricsTtm {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Revenue per share.
    pub revenue_per_share: Option<f64>,
    /// Net income per share.
    pub net_income_per_share: Option<f64>,
    /// Operating cash flow per share.
    pub operating_cash_flow_per_share: Option<f64>,
    /// Free cash flow per share.
    pub free_cash_flow_per_share: Option<f64>,
    /// Cash per share.
    pub cash_per_share: Option<f64>,
    /// Book value per share.
    pub book_value_per_share: Option<f64>,
    /// Tangible book value per share.
    pub tangible_book_value_per_share: Option<f64>,
    /// Shareholders' equity per share.
    pub shareholders_equity_per_share: Option<f64>,
    /// Interest-bearing debt per share.
    pub interest_debt_per_share: Option<f64>,
    /// Market capitalization.
    pub market_cap: Option<f64>,
    /// Enterprise value.
    pub enterprise_value: Option<f64>,
    /// Price-to-earnings ratio.
    pub pe_ratio: Option<f64>,
    /// Price-to-book ratio.
    pub pb_ratio: Option<f64>,
    /// Enterprise value to sales.
    pub ev_to_sales: Option<f64>,
    /// Enterprise value to EBITDA.
    pub ev_to_ebitda: Option<f64>,
    /// Enterprise value to operating cash flow.
    pub ev_to_operating_cash_flow: Option<f64>,
    /// Enterprise value to free cash flow.
    pub ev_to_free_cash_flow: Option<f64>,
    /// Earnings yield.
    pub earnings_yield: Option<f64>,
    /// Free cash flow yield.
    pub free_cash_flow_yield: Option<f64>,
    /// Debt to equity.
    pub debt_to_equity: Option<f64>,
    /// Debt to assets.
    pub debt_to_assets: Option<f64>,
    /// Net debt to EBITDA.
    pub net_debt_to_ebitda: Option<f64>,
    /// Current ratio.
    pub current_ratio: Option<f64>,
    /// Interest coverage.
    pub interest_coverage: Option<f64>,
    /// Return on equity.
    pub return_on_equity: Option<f64>,
    /// Return on invested capital.
    pub return_on_invested_capital: Option<f64>,
    /// Dividend yield (fraction, not percent).
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    pub payout_ratio: Option<f64>,
    /// Graham number.
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
    pub current_ratio: Option<f64>,
    /// Quick ratio.
    pub quick_ratio: Option<f64>,
    /// Cash ratio.
    pub cash_ratio: Option<f64>,
    /// Gross profit margin.
    pub gross_profit_margin: Option<f64>,
    /// Operating profit margin.
    pub operating_profit_margin: Option<f64>,
    /// Net profit margin.
    pub net_profit_margin: Option<f64>,
    /// Return on assets.
    pub return_on_assets: Option<f64>,
    /// Return on equity.
    pub return_on_equity: Option<f64>,
    /// Return on capital employed.
    pub return_on_capital_employed: Option<f64>,
    /// Debt ratio.
    pub debt_ratio: Option<f64>,
    /// Debt-to-equity ratio.
    pub debt_equity_ratio: Option<f64>,
    /// Interest coverage.
    pub interest_coverage: Option<f64>,
    /// Price-to-earnings ratio.
    pub price_earnings_ratio: Option<f64>,
    /// Price/earnings-to-growth ratio.
    pub peg_ratio: Option<f64>,
    /// Price-to-book ratio.
    pub price_to_book_ratio: Option<f64>,
    /// Price-to-sales ratio.
    pub price_to_sales_ratio: Option<f64>,
    /// Price-to-free-cash-flow ratio.
    pub price_to_free_cash_flows_ratio: Option<f64>,
    /// Enterprise value multiple (EV/EBITDA).
    pub enterprise_value_multiple: Option<f64>,
    /// Dividend yield (fraction, not percent).
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    pub payout_ratio: Option<f64>,
}
