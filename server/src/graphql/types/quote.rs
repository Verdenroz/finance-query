//! GraphQL type mirroring the library's `Quote` struct.
//!
//! Uses the dual-derive pattern (`SimpleObject` + `Deserialize`) so resolvers
//! can deserialize directly from the `serde_json::Value` stored in the cache
//! without any manual field-mapping.

use super::batch::GqlBatchError;
use super::formatted::GqlFormattedValue;
use crate::graphql::pagination::{self, Page};
use async_graphql::{ComplexObject, Json, Result, SimpleObject};
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
    pub regular_market_price: Option<Json<GqlFormattedValue>>,
    pub regular_market_change: Option<Json<GqlFormattedValue>>,
    pub regular_market_change_percent: Option<Json<GqlFormattedValue>>,
    pub regular_market_time: Option<i64>,
    pub regular_market_day_high: Option<Json<GqlFormattedValue>>,
    pub regular_market_day_low: Option<Json<GqlFormattedValue>>,
    pub regular_market_open: Option<Json<GqlFormattedValue>>,
    pub regular_market_previous_close: Option<Json<GqlFormattedValue>>,
    pub regular_market_volume: Option<Json<GqlFormattedValue>>,
    pub market_state: Option<String>,

    // ── Convenience aliases (without FormattedValue wrapper) ────────────────
    pub day_high: Option<Json<GqlFormattedValue>>,
    pub day_low: Option<Json<GqlFormattedValue>>,
    pub open: Option<Json<GqlFormattedValue>>,
    pub previous_close: Option<Json<GqlFormattedValue>>,
    pub volume: Option<Json<GqlFormattedValue>>,
    pub all_time_high: Option<Json<GqlFormattedValue>>,
    pub all_time_low: Option<Json<GqlFormattedValue>>,

    // ── Pre/post market ─────────────────────────────────────────────────────
    pub pre_market_price: Option<Json<GqlFormattedValue>>,
    pub pre_market_change: Option<Json<GqlFormattedValue>>,
    pub pre_market_change_percent: Option<Json<GqlFormattedValue>>,
    pub pre_market_time: Option<i64>,
    pub post_market_price: Option<Json<GqlFormattedValue>>,
    pub post_market_change: Option<Json<GqlFormattedValue>>,
    pub post_market_change_percent: Option<Json<GqlFormattedValue>>,
    pub post_market_time: Option<i64>,

    // ── Volume & market cap ─────────────────────────────────────────────────
    pub average_volume: Option<Json<GqlFormattedValue>>,
    pub market_cap: Option<Json<GqlFormattedValue>>,
    pub enterprise_value: Option<Json<GqlFormattedValue>>,
    pub enterprise_to_revenue: Option<Json<GqlFormattedValue>>,
    pub enterprise_to_ebitda: Option<Json<GqlFormattedValue>>,
    pub price_to_book: Option<Json<GqlFormattedValue>>,

    // ── Valuation ratios ────────────────────────────────────────────────────
    #[serde(alias = "forwardPE")]
    pub forward_pe: Option<Json<GqlFormattedValue>>,
    #[serde(alias = "trailingPE")]
    pub trailing_pe: Option<Json<GqlFormattedValue>>,
    pub beta: Option<Json<GqlFormattedValue>>,

    // ── 52-week range & moving averages ────────────────────────────────────
    pub fifty_two_week_high: Option<Json<GqlFormattedValue>>,
    pub fifty_two_week_low: Option<Json<GqlFormattedValue>>,
    #[serde(alias = "52WeekChange")]
    pub week_52_change: Option<Json<GqlFormattedValue>>,
    #[serde(alias = "SandP52WeekChange")]
    pub sand_p_52_week_change: Option<Json<GqlFormattedValue>>,
    pub fifty_day_average: Option<Json<GqlFormattedValue>>,
    pub two_hundred_day_average: Option<Json<GqlFormattedValue>>,

    // ── Dividends ───────────────────────────────────────────────────────────
    pub dividend_rate: Option<Json<GqlFormattedValue>>,
    pub dividend_yield: Option<Json<GqlFormattedValue>>,
    pub trailing_annual_dividend_rate: Option<Json<GqlFormattedValue>>,
    pub trailing_annual_dividend_yield: Option<Json<GqlFormattedValue>>,
    pub five_year_avg_dividend_yield: Option<Json<GqlFormattedValue>>,
    pub ex_dividend_date: Option<Json<GqlFormattedValue>>,
    pub payout_ratio: Option<Json<GqlFormattedValue>>,
    pub last_dividend_value: Option<Json<GqlFormattedValue>>,
    pub last_dividend_date: Option<Json<GqlFormattedValue>>,

    // ── Bid / ask ───────────────────────────────────────────────────────────
    pub bid: Option<Json<GqlFormattedValue>>,
    pub bid_size: Option<Json<GqlFormattedValue>>,
    pub ask: Option<Json<GqlFormattedValue>>,
    pub ask_size: Option<Json<GqlFormattedValue>>,

    // ── Shares & ownership ──────────────────────────────────────────────────
    pub shares_outstanding: Option<Json<GqlFormattedValue>>,
    pub float_shares: Option<Json<GqlFormattedValue>>,
    pub implied_shares_outstanding: Option<Json<GqlFormattedValue>>,
    pub held_percent_insiders: Option<Json<GqlFormattedValue>>,
    pub held_percent_institutions: Option<Json<GqlFormattedValue>>,
    pub shares_short: Option<Json<GqlFormattedValue>>,
    pub shares_short_prior_month: Option<Json<GqlFormattedValue>>,
    pub short_ratio: Option<Json<GqlFormattedValue>>,
    pub short_percent_of_float: Option<Json<GqlFormattedValue>>,
    pub shares_percent_shares_out: Option<Json<GqlFormattedValue>>,
    pub date_short_interest: Option<Json<GqlFormattedValue>>,

    // ── Analyst targets ─────────────────────────────────────────────────────
    pub current_price: Option<Json<GqlFormattedValue>>,
    pub target_high_price: Option<Json<GqlFormattedValue>>,
    pub target_low_price: Option<Json<GqlFormattedValue>>,
    pub target_mean_price: Option<Json<GqlFormattedValue>>,
    pub target_median_price: Option<Json<GqlFormattedValue>>,
    pub recommendation_mean: Option<Json<GqlFormattedValue>>,
    pub number_of_analyst_opinions: Option<Json<GqlFormattedValue>>,
    pub recommendation_key: Option<String>,

    // ── Financials (key metrics) ────────────────────────────────────────────
    pub total_debt: Option<Json<GqlFormattedValue>>,
    pub total_revenue: Option<Json<GqlFormattedValue>>,
    pub net_income_to_common: Option<Json<GqlFormattedValue>>,
    pub debt_to_equity: Option<Json<GqlFormattedValue>>,
    pub revenue_per_share: Option<Json<GqlFormattedValue>>,
    pub return_on_assets: Option<Json<GqlFormattedValue>>,
    pub return_on_equity: Option<Json<GqlFormattedValue>>,
    pub free_cashflow: Option<Json<GqlFormattedValue>>,
    pub operating_cashflow: Option<Json<GqlFormattedValue>>,
    pub profit_margins: Option<Json<GqlFormattedValue>>,
    pub gross_margins: Option<Json<GqlFormattedValue>>,
    pub ebitda_margins: Option<Json<GqlFormattedValue>>,
    pub operating_margins: Option<Json<GqlFormattedValue>>,
    pub gross_profits: Option<Json<GqlFormattedValue>>,
    pub earnings_growth: Option<Json<GqlFormattedValue>>,
    pub revenue_growth: Option<Json<GqlFormattedValue>>,
    pub earnings_quarterly_growth: Option<Json<GqlFormattedValue>>,
    pub current_ratio: Option<Json<GqlFormattedValue>>,
    pub quick_ratio: Option<Json<GqlFormattedValue>>,
    pub trailing_eps: Option<Json<GqlFormattedValue>>,
    pub forward_eps: Option<Json<GqlFormattedValue>>,
    pub book_value: Option<Json<GqlFormattedValue>>,

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
    pub nav_price: Option<Json<GqlFormattedValue>>,
    pub total_assets: Option<Json<GqlFormattedValue>>,
    #[serde(alias = "yield")]
    pub yield_value: Option<Json<GqlFormattedValue>>,

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
    pub price_hint: Option<Json<GqlFormattedValue>>,

    // ── Dates ───────────────────────────────────────────────────────────────
    pub last_split_date: Option<Json<GqlFormattedValue>>,
    pub last_split_factor: Option<String>,
    pub last_fiscal_year_end: Option<Json<GqlFormattedValue>>,
    pub next_fiscal_year_end: Option<Json<GqlFormattedValue>>,
    pub most_recent_quarter: Option<Json<GqlFormattedValue>>,

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

/// Result of the batch `quotes` root field: successfully fetched quotes plus
/// any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct GqlQuotesBatch {
    #[graphql(skip)]
    pub quotes: Vec<GqlQuote>,
    pub errors: Vec<GqlBatchError>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlQuotesBatch {
    /// Successfully fetched quotes.
    async fn quotes(
        &self,
        #[graphql(desc = "Max quotes to return; omitted = every matching quote in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlQuote>> {
        pagination::paginate(&self.quotes, first, after).await
    }
}
