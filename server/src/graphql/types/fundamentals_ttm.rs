//! GraphQL types for the fundamentals surfaces that hang off a ticker.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// One day of FINRA short-sale volume.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlShortVolume {
    pub date: Option<String>,
    pub short_volume: Option<f64>,
    pub short_exempt_volume: Option<f64>,
    pub total_volume: Option<f64>,
}

/// One analyst upgrade or downgrade, provider-routed.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlGradingAction {
    pub symbol: Option<String>,
    pub date: Option<String>,
    pub grading_company: Option<String>,
    pub previous_grade: Option<String>,
    pub new_grade: Option<String>,
}

/// Analyst price-target publication counts and averages by window.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlPriceTargetSummary {
    pub symbol: Option<String>,
    pub last_month_count: Option<i64>,
    pub last_month_avg: Option<f64>,
    pub last_quarter_count: Option<i64>,
    pub last_quarter_avg: Option<f64>,
    pub last_year_count: Option<i64>,
    pub last_year_avg: Option<f64>,
    pub all_time_count: Option<i64>,
    pub all_time_avg: Option<f64>,
}

/// One executive's disclosed compensation for a year.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlExecutiveCompensation {
    pub symbol: Option<String>,
    pub cik: Option<String>,
    pub company_name: Option<String>,
    pub name_and_position: Option<String>,
    pub year: Option<i32>,
    pub salary: Option<f64>,
    pub bonus: Option<f64>,
    pub stock_award: Option<f64>,
    pub option_award: Option<f64>,
    pub incentive_plan_compensation: Option<f64>,
    pub other_compensation: Option<f64>,
    pub total: Option<f64>,
    pub filing_date: Option<String>,
    pub url: Option<String>,
}

/// Trailing-twelve-month key metrics.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlKeyMetricsTtm {
    pub symbol: Option<String>,
    pub market_cap: Option<f64>,
    pub enterprise_value: Option<f64>,
    pub ev_to_sales: Option<f64>,
    pub ev_to_operating_cash_flow: Option<f64>,
    pub ev_to_free_cash_flow: Option<f64>,
    pub ev_to_ebitda: Option<f64>,
    pub net_debt_to_ebitda: Option<f64>,
    pub current_ratio: Option<f64>,
    pub income_quality: Option<f64>,
    pub graham_number: Option<f64>,
    pub graham_net_net: Option<f64>,
    pub tax_burden: Option<f64>,
    pub interest_burden: Option<f64>,
    pub working_capital: Option<f64>,
    pub invested_capital: Option<f64>,
    pub return_on_assets: Option<f64>,
    pub operating_return_on_assets: Option<f64>,
    pub return_on_tangible_assets: Option<f64>,
    pub return_on_equity: Option<f64>,
    pub return_on_invested_capital: Option<f64>,
    pub return_on_capital_employed: Option<f64>,
    pub earnings_yield: Option<f64>,
    pub free_cash_flow_yield: Option<f64>,
    pub capex_to_operating_cash_flow: Option<f64>,
    pub capex_to_depreciation: Option<f64>,
    pub capex_to_revenue: Option<f64>,
    pub sales_general_and_administrative_to_revenue: Option<f64>,
    pub research_and_development_to_revenue: Option<f64>,
    pub stock_based_compensation_to_revenue: Option<f64>,
    pub intangibles_to_total_assets: Option<f64>,
    pub average_receivables: Option<f64>,
    pub average_payables: Option<f64>,
    pub average_inventory: Option<f64>,
    pub days_of_sales_outstanding: Option<f64>,
    pub days_of_payables_outstanding: Option<f64>,
    pub days_of_inventory_outstanding: Option<f64>,
    pub operating_cycle: Option<f64>,
    pub cash_conversion_cycle: Option<f64>,
    pub free_cash_flow_to_equity: Option<f64>,
    pub free_cash_flow_to_firm: Option<f64>,
    pub tangible_asset_value: Option<f64>,
    pub net_current_asset_value: Option<f64>,
}

/// Trailing-twelve-month financial ratios.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFinancialRatiosTtm {
    pub symbol: Option<String>,
    pub gross_profit_margin: Option<f64>,
    pub ebit_margin: Option<f64>,
    pub ebitda_margin: Option<f64>,
    pub operating_profit_margin: Option<f64>,
    pub pretax_profit_margin: Option<f64>,
    pub continuous_operations_profit_margin: Option<f64>,
    pub net_profit_margin: Option<f64>,
    pub bottom_line_profit_margin: Option<f64>,
    pub receivables_turnover: Option<f64>,
    pub payables_turnover: Option<f64>,
    pub inventory_turnover: Option<f64>,
    pub fixed_asset_turnover: Option<f64>,
    pub asset_turnover: Option<f64>,
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub solvency_ratio: Option<f64>,
    pub cash_ratio: Option<f64>,
    pub price_earnings_ratio: Option<f64>,
    pub peg_ratio: Option<f64>,
    pub forward_peg_ratio: Option<f64>,
    pub price_to_earnings_diluted_ratio: Option<f64>,
    pub price_to_earnings_diluted_growth_ratio: Option<f64>,
    pub price_to_book_ratio: Option<f64>,
    pub price_to_sales_ratio: Option<f64>,
    pub price_to_free_cash_flows_ratio: Option<f64>,
    pub price_to_operating_cash_flow_ratio: Option<f64>,
    pub debt_ratio: Option<f64>,
    pub debt_equity_ratio: Option<f64>,
    pub debt_to_capital_ratio: Option<f64>,
    pub long_term_debt_to_capital_ratio: Option<f64>,
    pub financial_leverage_ratio: Option<f64>,
    pub working_capital_turnover_ratio: Option<f64>,
    pub operating_cash_flow_ratio: Option<f64>,
    pub operating_cash_flow_sales_ratio: Option<f64>,
    pub free_cash_flow_operating_cash_flow_ratio: Option<f64>,
    pub debt_service_coverage_ratio: Option<f64>,
    pub interest_coverage: Option<f64>,
    pub short_term_operating_cash_flow_coverage_ratio: Option<f64>,
    pub operating_cash_flow_coverage_ratio: Option<f64>,
    pub capital_expenditure_coverage_ratio: Option<f64>,
    pub dividend_paid_and_capex_coverage_ratio: Option<f64>,
    pub payout_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub dividend_per_share: Option<f64>,
    pub enterprise_value: Option<f64>,
    pub enterprise_value_multiple: Option<f64>,
    pub revenue_per_share: Option<f64>,
    pub net_income_per_share: Option<f64>,
    pub interest_debt_per_share: Option<f64>,
    pub cash_per_share: Option<f64>,
    pub book_value_per_share: Option<f64>,
    pub tangible_book_value_per_share: Option<f64>,
    pub shareholders_equity_per_share: Option<f64>,
    pub operating_cash_flow_per_share: Option<f64>,
    pub capex_per_share: Option<f64>,
    pub free_cash_flow_per_share: Option<f64>,
    pub net_income_per_ebt: Option<f64>,
    pub ebt_per_ebit: Option<f64>,
    pub price_to_fair_value: Option<f64>,
    pub debt_to_market_cap: Option<f64>,
    pub effective_tax_rate: Option<f64>,
}
