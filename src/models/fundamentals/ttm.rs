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
//!
//! Per-share metrics live on [`FinancialRatiosTtm`], not [`KeyMetricsTtm`] —
//! FMP's stable tier serves them from `ratios-ttm`.

use serde::{Deserialize, Serialize};

/// Valuation, capital-efficiency, and working-capital metrics over the
/// trailing twelve months.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyMetricsTtm {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Market capitalization.
    #[serde(alias = "marketCap")]
    pub market_cap: Option<f64>,
    /// Enterprise value.
    #[serde(alias = "enterpriseValueTTM")]
    pub enterprise_value: Option<f64>,
    /// Enterprise value to sales.
    #[serde(alias = "evToSalesTTM")]
    pub ev_to_sales: Option<f64>,
    /// Enterprise value to operating cash flow.
    #[serde(alias = "evToOperatingCashFlowTTM")]
    pub ev_to_operating_cash_flow: Option<f64>,
    /// Enterprise value to free cash flow.
    #[serde(alias = "evToFreeCashFlowTTM")]
    pub ev_to_free_cash_flow: Option<f64>,
    /// Enterprise value to EBITDA.
    #[serde(alias = "evToEBITDATTM")]
    pub ev_to_ebitda: Option<f64>,
    /// Net debt to EBITDA.
    #[serde(alias = "netDebtToEBITDATTM")]
    pub net_debt_to_ebitda: Option<f64>,
    /// Current ratio.
    #[serde(alias = "currentRatioTTM")]
    pub current_ratio: Option<f64>,
    /// Operating cash flow divided by net income.
    #[serde(alias = "incomeQualityTTM")]
    pub income_quality: Option<f64>,
    /// Graham number.
    #[serde(alias = "grahamNumberTTM")]
    pub graham_number: Option<f64>,
    /// Graham net-net working capital per share.
    #[serde(alias = "grahamNetNetTTM")]
    pub graham_net_net: Option<f64>,
    /// Net income divided by pre-tax income (fraction).
    #[serde(alias = "taxBurdenTTM")]
    pub tax_burden: Option<f64>,
    /// Pre-tax income divided by EBIT (fraction).
    #[serde(alias = "interestBurdenTTM")]
    pub interest_burden: Option<f64>,
    /// Working capital.
    #[serde(alias = "workingCapitalTTM")]
    pub working_capital: Option<f64>,
    /// Invested capital.
    #[serde(alias = "investedCapitalTTM")]
    pub invested_capital: Option<f64>,
    /// Return on assets (fraction).
    #[serde(alias = "returnOnAssetsTTM")]
    pub return_on_assets: Option<f64>,
    /// Operating income divided by total assets (fraction).
    #[serde(alias = "operatingReturnOnAssetsTTM")]
    pub operating_return_on_assets: Option<f64>,
    /// Return on tangible assets (fraction).
    #[serde(alias = "returnOnTangibleAssetsTTM")]
    pub return_on_tangible_assets: Option<f64>,
    /// Return on equity (fraction).
    #[serde(alias = "returnOnEquityTTM")]
    pub return_on_equity: Option<f64>,
    /// Return on invested capital (fraction).
    #[serde(alias = "returnOnInvestedCapitalTTM")]
    pub return_on_invested_capital: Option<f64>,
    /// Return on capital employed (fraction).
    #[serde(alias = "returnOnCapitalEmployedTTM")]
    pub return_on_capital_employed: Option<f64>,
    /// Earnings yield (fraction).
    #[serde(alias = "earningsYieldTTM")]
    pub earnings_yield: Option<f64>,
    /// Free cash flow yield (fraction).
    #[serde(alias = "freeCashFlowYieldTTM")]
    pub free_cash_flow_yield: Option<f64>,
    /// Capital expenditure to operating cash flow (fraction).
    #[serde(alias = "capexToOperatingCashFlowTTM")]
    pub capex_to_operating_cash_flow: Option<f64>,
    /// Capital expenditure to depreciation (fraction).
    #[serde(alias = "capexToDepreciationTTM")]
    pub capex_to_depreciation: Option<f64>,
    /// Capital expenditure to revenue (fraction).
    #[serde(alias = "capexToRevenueTTM")]
    pub capex_to_revenue: Option<f64>,
    /// Selling, general and administrative expense to revenue (fraction).
    #[serde(alias = "salesGeneralAndAdministrativeToRevenueTTM")]
    pub sales_general_and_administrative_to_revenue: Option<f64>,
    /// Research and development expense to revenue (fraction).
    // FMP misspells this key upstream ("Developement"); the alias is verbatim.
    #[serde(alias = "researchAndDevelopementToRevenueTTM")]
    pub research_and_development_to_revenue: Option<f64>,
    /// Stock-based compensation to revenue (fraction).
    #[serde(alias = "stockBasedCompensationToRevenueTTM")]
    pub stock_based_compensation_to_revenue: Option<f64>,
    /// Intangible assets to total assets (fraction).
    #[serde(alias = "intangiblesToTotalAssetsTTM")]
    pub intangibles_to_total_assets: Option<f64>,
    /// Average receivables.
    #[serde(alias = "averageReceivablesTTM")]
    pub average_receivables: Option<f64>,
    /// Average payables.
    #[serde(alias = "averagePayablesTTM")]
    pub average_payables: Option<f64>,
    /// Average inventory.
    #[serde(alias = "averageInventoryTTM")]
    pub average_inventory: Option<f64>,
    /// Days sales outstanding.
    #[serde(alias = "daysOfSalesOutstandingTTM")]
    pub days_of_sales_outstanding: Option<f64>,
    /// Days payables outstanding.
    #[serde(alias = "daysOfPayablesOutstandingTTM")]
    pub days_of_payables_outstanding: Option<f64>,
    /// Days inventory outstanding.
    #[serde(alias = "daysOfInventoryOutstandingTTM")]
    pub days_of_inventory_outstanding: Option<f64>,
    /// Operating cycle in days.
    #[serde(alias = "operatingCycleTTM")]
    pub operating_cycle: Option<f64>,
    /// Cash conversion cycle in days.
    #[serde(alias = "cashConversionCycleTTM")]
    pub cash_conversion_cycle: Option<f64>,
    /// Free cash flow to equity.
    #[serde(alias = "freeCashFlowToEquityTTM")]
    pub free_cash_flow_to_equity: Option<f64>,
    /// Free cash flow to the firm.
    #[serde(alias = "freeCashFlowToFirmTTM")]
    pub free_cash_flow_to_firm: Option<f64>,
    /// Tangible asset value.
    #[serde(alias = "tangibleAssetValueTTM")]
    pub tangible_asset_value: Option<f64>,
    /// Net current asset value.
    #[serde(alias = "netCurrentAssetValueTTM")]
    pub net_current_asset_value: Option<f64>,
}

/// Margin, turnover, liquidity, coverage, valuation, and per-share ratios over
/// the trailing twelve months.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialRatiosTtm {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Gross profit margin (fraction).
    #[serde(alias = "grossProfitMarginTTM")]
    pub gross_profit_margin: Option<f64>,
    /// EBIT margin (fraction).
    #[serde(alias = "ebitMarginTTM")]
    pub ebit_margin: Option<f64>,
    /// EBITDA margin (fraction).
    #[serde(alias = "ebitdaMarginTTM")]
    pub ebitda_margin: Option<f64>,
    /// Operating profit margin (fraction).
    #[serde(alias = "operatingProfitMarginTTM")]
    pub operating_profit_margin: Option<f64>,
    /// Pre-tax profit margin (fraction).
    #[serde(alias = "pretaxProfitMarginTTM")]
    pub pretax_profit_margin: Option<f64>,
    /// Continuing-operations profit margin (fraction).
    #[serde(alias = "continuousOperationsProfitMarginTTM")]
    pub continuous_operations_profit_margin: Option<f64>,
    /// Net profit margin (fraction).
    #[serde(alias = "netProfitMarginTTM")]
    pub net_profit_margin: Option<f64>,
    /// Bottom-line profit margin (fraction).
    #[serde(alias = "bottomLineProfitMarginTTM")]
    pub bottom_line_profit_margin: Option<f64>,
    /// Receivables turnover.
    #[serde(alias = "receivablesTurnoverTTM")]
    pub receivables_turnover: Option<f64>,
    /// Payables turnover.
    #[serde(alias = "payablesTurnoverTTM")]
    pub payables_turnover: Option<f64>,
    /// Inventory turnover.
    #[serde(alias = "inventoryTurnoverTTM")]
    pub inventory_turnover: Option<f64>,
    /// Fixed-asset turnover.
    #[serde(alias = "fixedAssetTurnoverTTM")]
    pub fixed_asset_turnover: Option<f64>,
    /// Asset turnover.
    #[serde(alias = "assetTurnoverTTM")]
    pub asset_turnover: Option<f64>,
    /// Current ratio.
    #[serde(alias = "currentRatioTTM")]
    pub current_ratio: Option<f64>,
    /// Quick ratio.
    #[serde(alias = "quickRatioTTM")]
    pub quick_ratio: Option<f64>,
    /// Solvency ratio.
    #[serde(alias = "solvencyRatioTTM")]
    pub solvency_ratio: Option<f64>,
    /// Cash ratio.
    #[serde(alias = "cashRatioTTM")]
    pub cash_ratio: Option<f64>,
    /// Price-to-earnings ratio.
    #[serde(alias = "priceToEarningsRatioTTM")]
    pub price_earnings_ratio: Option<f64>,
    /// Price/earnings-to-growth ratio.
    #[serde(alias = "priceToEarningsGrowthRatioTTM")]
    pub peg_ratio: Option<f64>,
    /// Forward price/earnings-to-growth ratio.
    #[serde(alias = "forwardPriceToEarningsGrowthRatioTTM")]
    pub forward_peg_ratio: Option<f64>,
    /// Diluted price-to-earnings ratio.
    #[serde(alias = "priceToEarningsDilutedRatioTTM")]
    pub price_to_earnings_diluted_ratio: Option<f64>,
    /// Diluted price/earnings-to-growth ratio.
    #[serde(alias = "priceToEarningsDilutedGrowthRatioTTM")]
    pub price_to_earnings_diluted_growth_ratio: Option<f64>,
    /// Price-to-book ratio.
    #[serde(alias = "priceToBookRatioTTM")]
    pub price_to_book_ratio: Option<f64>,
    /// Price-to-sales ratio.
    #[serde(alias = "priceToSalesRatioTTM")]
    pub price_to_sales_ratio: Option<f64>,
    /// Price-to-free-cash-flow ratio.
    #[serde(alias = "priceToFreeCashFlowRatioTTM")]
    pub price_to_free_cash_flows_ratio: Option<f64>,
    /// Price-to-operating-cash-flow ratio.
    #[serde(alias = "priceToOperatingCashFlowRatioTTM")]
    pub price_to_operating_cash_flow_ratio: Option<f64>,
    /// Debt-to-assets ratio.
    #[serde(alias = "debtToAssetsRatioTTM")]
    pub debt_ratio: Option<f64>,
    /// Debt-to-equity ratio.
    #[serde(alias = "debtToEquityRatioTTM")]
    pub debt_equity_ratio: Option<f64>,
    /// Debt-to-capital ratio.
    #[serde(alias = "debtToCapitalRatioTTM")]
    pub debt_to_capital_ratio: Option<f64>,
    /// Long-term-debt-to-capital ratio.
    #[serde(alias = "longTermDebtToCapitalRatioTTM")]
    pub long_term_debt_to_capital_ratio: Option<f64>,
    /// Financial leverage ratio.
    #[serde(alias = "financialLeverageRatioTTM")]
    pub financial_leverage_ratio: Option<f64>,
    /// Working-capital turnover ratio.
    #[serde(alias = "workingCapitalTurnoverRatioTTM")]
    pub working_capital_turnover_ratio: Option<f64>,
    /// Operating cash flow to current liabilities.
    #[serde(alias = "operatingCashFlowRatioTTM")]
    pub operating_cash_flow_ratio: Option<f64>,
    /// Operating cash flow to sales (fraction).
    #[serde(alias = "operatingCashFlowSalesRatioTTM")]
    pub operating_cash_flow_sales_ratio: Option<f64>,
    /// Free cash flow to operating cash flow (fraction).
    #[serde(alias = "freeCashFlowOperatingCashFlowRatioTTM")]
    pub free_cash_flow_operating_cash_flow_ratio: Option<f64>,
    /// Debt-service coverage ratio.
    #[serde(alias = "debtServiceCoverageRatioTTM")]
    pub debt_service_coverage_ratio: Option<f64>,
    /// Interest coverage.
    #[serde(alias = "interestCoverageRatioTTM")]
    pub interest_coverage: Option<f64>,
    /// Short-term operating cash flow coverage ratio.
    #[serde(alias = "shortTermOperatingCashFlowCoverageRatioTTM")]
    pub short_term_operating_cash_flow_coverage_ratio: Option<f64>,
    /// Operating cash flow coverage ratio.
    #[serde(alias = "operatingCashFlowCoverageRatioTTM")]
    pub operating_cash_flow_coverage_ratio: Option<f64>,
    /// Capital-expenditure coverage ratio.
    #[serde(alias = "capitalExpenditureCoverageRatioTTM")]
    pub capital_expenditure_coverage_ratio: Option<f64>,
    /// Dividends-paid-and-capex coverage ratio.
    #[serde(alias = "dividendPaidAndCapexCoverageRatioTTM")]
    pub dividend_paid_and_capex_coverage_ratio: Option<f64>,
    /// Payout ratio (fraction).
    #[serde(alias = "dividendPayoutRatioTTM")]
    pub payout_ratio: Option<f64>,
    /// Dividend yield (fraction, not percent).
    #[serde(alias = "dividendYielTTM", alias = "dividendYieldTTM")]
    pub dividend_yield: Option<f64>,
    /// Dividend per share.
    #[serde(alias = "dividendPerShareTTM")]
    pub dividend_per_share: Option<f64>,
    /// Enterprise value.
    #[serde(alias = "enterpriseValueTTM")]
    pub enterprise_value: Option<f64>,
    /// Enterprise value multiple (EV/EBITDA).
    #[serde(alias = "enterpriseValueMultipleTTM")]
    pub enterprise_value_multiple: Option<f64>,
    /// Revenue per share.
    #[serde(alias = "revenuePerShareTTM")]
    pub revenue_per_share: Option<f64>,
    /// Net income per share.
    #[serde(alias = "netIncomePerShareTTM")]
    pub net_income_per_share: Option<f64>,
    /// Interest-bearing debt per share.
    #[serde(alias = "interestDebtPerShareTTM")]
    pub interest_debt_per_share: Option<f64>,
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
    /// Operating cash flow per share.
    #[serde(alias = "operatingCashFlowPerShareTTM")]
    pub operating_cash_flow_per_share: Option<f64>,
    /// Capital expenditure per share.
    #[serde(alias = "capexPerShareTTM")]
    pub capex_per_share: Option<f64>,
    /// Free cash flow per share.
    #[serde(alias = "freeCashFlowPerShareTTM")]
    pub free_cash_flow_per_share: Option<f64>,
    /// Net income divided by pre-tax income (fraction).
    #[serde(alias = "netIncomePerEBTTTM")]
    pub net_income_per_ebt: Option<f64>,
    /// Pre-tax income divided by EBIT (fraction).
    #[serde(alias = "ebtPerEbitTTM")]
    pub ebt_per_ebit: Option<f64>,
    /// Price to fair value.
    #[serde(alias = "priceToFairValueTTM")]
    pub price_to_fair_value: Option<f64>,
    /// Total debt to market capitalization (fraction).
    #[serde(alias = "debtToMarketCapTTM")]
    pub debt_to_market_cap: Option<f64>,
    /// Effective tax rate (fraction).
    #[serde(alias = "effectiveTaxRateTTM")]
    pub effective_tax_rate: Option<f64>,
}
