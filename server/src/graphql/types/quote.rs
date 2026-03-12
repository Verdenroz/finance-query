//! GraphQL type mirroring the library's `Quote` struct.
//!
//! Uses the dual-derive pattern (`SimpleObject` + `Deserialize`) so resolvers
//! can deserialize directly from the `serde_json::Value` stored in the cache
//! without any manual field-mapping.

use async_graphql::{Json, SimpleObject};
use serde::Deserialize;

/// Full quote data for a stock / ETF / fund, mirroring `finance_query::Quote`.
///
/// Scalar and formatted-value fields are typed directly; complex nested objects
/// (e.g. `earnings`, `calendarEvents`) are exposed as opaque `Json<Value>` and
/// can be promoted to typed wrappers later without breaking the schema.
#[derive(SimpleObject, Deserialize, Default, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlQuote {
    // ── Identity & metadata ─────────────────────────────────────────────────
    pub symbol: Option<String>,
    pub logo_url: Option<String>,
    pub company_logo_url: Option<String>,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub quote_type: Option<String>,
    pub currency: Option<String>,
    pub currency_symbol: Option<String>,
    pub underlying_symbol: Option<String>,
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,

    // ── Real-time price data ────────────────────────────────────────────────
    pub regular_market_price: Option<Json<serde_json::Value>>,
    pub regular_market_change: Option<Json<serde_json::Value>>,
    pub regular_market_change_percent: Option<Json<serde_json::Value>>,
    pub regular_market_time: Option<i64>,
    pub regular_market_day_high: Option<Json<serde_json::Value>>,
    pub regular_market_day_low: Option<Json<serde_json::Value>>,
    pub regular_market_open: Option<Json<serde_json::Value>>,
    pub regular_market_previous_close: Option<Json<serde_json::Value>>,
    pub regular_market_volume: Option<Json<serde_json::Value>>,
    pub market_state: Option<String>,

    // ── Convenience aliases (without FormattedValue wrapper) ────────────────
    pub day_high: Option<Json<serde_json::Value>>,
    pub day_low: Option<Json<serde_json::Value>>,
    pub open: Option<Json<serde_json::Value>>,
    pub previous_close: Option<Json<serde_json::Value>>,
    pub volume: Option<Json<serde_json::Value>>,
    pub all_time_high: Option<Json<serde_json::Value>>,
    pub all_time_low: Option<Json<serde_json::Value>>,

    // ── Pre/post market ─────────────────────────────────────────────────────
    pub pre_market_price: Option<Json<serde_json::Value>>,
    pub pre_market_change: Option<Json<serde_json::Value>>,
    pub pre_market_change_percent: Option<Json<serde_json::Value>>,
    pub pre_market_time: Option<i64>,
    pub post_market_price: Option<Json<serde_json::Value>>,
    pub post_market_change: Option<Json<serde_json::Value>>,
    pub post_market_change_percent: Option<Json<serde_json::Value>>,
    pub post_market_time: Option<i64>,

    // ── Volume & market cap ─────────────────────────────────────────────────
    pub average_volume: Option<Json<serde_json::Value>>,
    pub market_cap: Option<Json<serde_json::Value>>,
    pub enterprise_value: Option<Json<serde_json::Value>>,
    pub enterprise_to_revenue: Option<Json<serde_json::Value>>,
    pub enterprise_to_ebitda: Option<Json<serde_json::Value>>,
    pub price_to_book: Option<Json<serde_json::Value>>,

    // ── Valuation ratios ────────────────────────────────────────────────────
    pub forward_pe: Option<Json<serde_json::Value>>,
    pub trailing_pe: Option<Json<serde_json::Value>>,
    pub beta: Option<Json<serde_json::Value>>,

    // ── 52-week range & moving averages ────────────────────────────────────
    pub fifty_two_week_high: Option<Json<serde_json::Value>>,
    pub fifty_two_week_low: Option<Json<serde_json::Value>>,
    pub fifty_day_average: Option<Json<serde_json::Value>>,
    pub two_hundred_day_average: Option<Json<serde_json::Value>>,

    // ── Dividends ───────────────────────────────────────────────────────────
    pub dividend_rate: Option<Json<serde_json::Value>>,
    pub dividend_yield: Option<Json<serde_json::Value>>,
    pub trailing_annual_dividend_rate: Option<Json<serde_json::Value>>,
    pub trailing_annual_dividend_yield: Option<Json<serde_json::Value>>,
    pub five_year_avg_dividend_yield: Option<Json<serde_json::Value>>,
    pub ex_dividend_date: Option<Json<serde_json::Value>>,
    pub payout_ratio: Option<Json<serde_json::Value>>,
    pub last_dividend_value: Option<Json<serde_json::Value>>,
    pub last_dividend_date: Option<Json<serde_json::Value>>,

    // ── Bid / ask ───────────────────────────────────────────────────────────
    pub bid: Option<Json<serde_json::Value>>,
    pub bid_size: Option<Json<serde_json::Value>>,
    pub ask: Option<Json<serde_json::Value>>,
    pub ask_size: Option<Json<serde_json::Value>>,

    // ── Shares & ownership ──────────────────────────────────────────────────
    pub shares_outstanding: Option<Json<serde_json::Value>>,
    pub float_shares: Option<Json<serde_json::Value>>,
    pub implied_shares_outstanding: Option<Json<serde_json::Value>>,
    pub held_percent_insiders: Option<Json<serde_json::Value>>,
    pub held_percent_institutions: Option<Json<serde_json::Value>>,
    pub shares_short: Option<Json<serde_json::Value>>,
    pub shares_short_prior_month: Option<Json<serde_json::Value>>,
    pub short_ratio: Option<Json<serde_json::Value>>,
    pub short_percent_of_float: Option<Json<serde_json::Value>>,
    pub shares_percent_shares_out: Option<Json<serde_json::Value>>,
    pub date_short_interest: Option<Json<serde_json::Value>>,

    // ── Analyst targets ─────────────────────────────────────────────────────
    pub current_price: Option<Json<serde_json::Value>>,
    pub target_high_price: Option<Json<serde_json::Value>>,
    pub target_low_price: Option<Json<serde_json::Value>>,
    pub target_mean_price: Option<Json<serde_json::Value>>,
    pub target_median_price: Option<Json<serde_json::Value>>,
    pub recommendation_mean: Option<Json<serde_json::Value>>,
    pub number_of_analyst_opinions: Option<Json<serde_json::Value>>,
    pub recommendation_key: Option<String>,

    // ── Financials (key metrics) ────────────────────────────────────────────
    pub total_debt: Option<Json<serde_json::Value>>,
    pub total_revenue: Option<Json<serde_json::Value>>,
    pub net_income_to_common: Option<Json<serde_json::Value>>,
    pub debt_to_equity: Option<Json<serde_json::Value>>,
    pub revenue_per_share: Option<Json<serde_json::Value>>,
    pub return_on_assets: Option<Json<serde_json::Value>>,
    pub return_on_equity: Option<Json<serde_json::Value>>,
    pub free_cashflow: Option<Json<serde_json::Value>>,
    pub operating_cashflow: Option<Json<serde_json::Value>>,
    pub profit_margins: Option<Json<serde_json::Value>>,
    pub gross_margins: Option<Json<serde_json::Value>>,
    pub ebitda_margins: Option<Json<serde_json::Value>>,
    pub operating_margins: Option<Json<serde_json::Value>>,
    pub gross_profits: Option<Json<serde_json::Value>>,
    pub earnings_growth: Option<Json<serde_json::Value>>,
    pub revenue_growth: Option<Json<serde_json::Value>>,
    pub earnings_quarterly_growth: Option<Json<serde_json::Value>>,
    pub current_ratio: Option<Json<serde_json::Value>>,
    pub quick_ratio: Option<Json<serde_json::Value>>,
    pub trailing_eps: Option<Json<serde_json::Value>>,
    pub forward_eps: Option<Json<serde_json::Value>>,
    pub book_value: Option<Json<serde_json::Value>>,

    // ── Company profile ─────────────────────────────────────────────────────
    pub sector: Option<String>,
    pub sector_key: Option<String>,
    pub sector_disp: Option<String>,
    pub industry: Option<String>,
    pub industry_key: Option<String>,
    pub industry_disp: Option<String>,
    pub long_business_summary: Option<String>,
    pub website: Option<String>,
    pub ir_website: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub full_time_employees: Option<i64>,

    // ── Fund-specific ───────────────────────────────────────────────────────
    pub category: Option<String>,
    pub fund_family: Option<String>,
    pub nav_price: Option<Json<serde_json::Value>>,
    pub total_assets: Option<Json<serde_json::Value>>,
    pub yield_value: Option<Json<serde_json::Value>>,

    // ── Governance ──────────────────────────────────────────────────────────
    pub audit_risk: Option<i32>,
    pub board_risk: Option<i32>,
    pub compensation_risk: Option<i32>,
    pub shareholder_rights_risk: Option<i32>,
    pub overall_risk: Option<i32>,

    // ── Exchange metadata ───────────────────────────────────────────────────
    pub time_zone_full_name: Option<String>,
    pub time_zone_short_name: Option<String>,
    pub gmt_off_set_milliseconds: Option<i64>,
    pub first_trade_date_epoch_utc: Option<i64>,
    pub exchange_data_delayed_by: Option<i32>,
    pub financial_currency: Option<String>,
    pub tradeable: Option<bool>,
    pub price_hint: Option<Json<serde_json::Value>>,

    // ── Dates ───────────────────────────────────────────────────────────────
    pub last_split_date: Option<Json<serde_json::Value>>,
    pub last_split_factor: Option<String>,
    pub last_fiscal_year_end: Option<Json<serde_json::Value>>,
    pub next_fiscal_year_end: Option<Json<serde_json::Value>>,
    pub most_recent_quarter: Option<Json<serde_json::Value>>,

    // ── Complex nested objects exposed as opaque JSON ───────────────────────
    // These can be promoted to typed wrappers in a future PR.
    pub earnings: Option<Json<serde_json::Value>>,
    pub calendar_events: Option<Json<serde_json::Value>>,
    pub recommendation_trend: Option<Json<serde_json::Value>>,
    pub upgrade_downgrade_history: Option<Json<serde_json::Value>>,
    pub earnings_history: Option<Json<serde_json::Value>>,
    pub earnings_trend: Option<Json<serde_json::Value>>,
    pub insider_holders: Option<Json<serde_json::Value>>,
    pub insider_transactions: Option<Json<serde_json::Value>>,
    pub institution_ownership: Option<Json<serde_json::Value>>,
    pub fund_ownership: Option<Json<serde_json::Value>>,
    pub major_holders_breakdown: Option<Json<serde_json::Value>>,
    pub net_share_purchase_activity: Option<Json<serde_json::Value>>,
    pub sec_filings: Option<Json<serde_json::Value>>,
    pub balance_sheet_history: Option<Json<serde_json::Value>>,
    pub balance_sheet_history_quarterly: Option<Json<serde_json::Value>>,
    pub cashflow_statement_history: Option<Json<serde_json::Value>>,
    pub cashflow_statement_history_quarterly: Option<Json<serde_json::Value>>,
    pub income_statement_history: Option<Json<serde_json::Value>>,
    pub income_statement_history_quarterly: Option<Json<serde_json::Value>>,
    pub equity_performance: Option<Json<serde_json::Value>>,
    pub index_trend: Option<Json<serde_json::Value>>,
    pub industry_trend: Option<Json<serde_json::Value>>,
    pub sector_trend: Option<Json<serde_json::Value>>,
    pub fund_profile: Option<Json<serde_json::Value>>,
    pub fund_performance: Option<Json<serde_json::Value>>,
    pub top_holdings: Option<Json<serde_json::Value>>,
    pub company_officers: Option<Json<serde_json::Value>>,
}
