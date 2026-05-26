use crate::models::format::{Both, Format};
use finance_query_derive::FormatConvert;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Default key statistics for a symbol
///
/// Contains extensive statistical data including valuation metrics, share data, and financial ratios.
///
/// The type parameter `F` controls how numeric fields are represented:
/// - `DefaultKeyStatistics` / `DefaultKeyStatistics<Both>` — **default**; fields hold `FormattedValue<T>`
/// - `DefaultKeyStatistics<Raw>` — fields hold `T` directly (e.g. `Option<f64>`)
/// - `DefaultKeyStatistics<Pretty>` — fields hold `Option<String>` (human-readable)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FormatConvert)]
#[serde(rename_all = "camelCase", bound = "")]
pub struct DefaultKeyStatistics<F: Format = Both> {
    /// 52-week price change percentage
    #[serde(rename = "52WeekChange", skip_serializing_if = "Option::is_none")]
    pub week_52_change: Option<F::Value<f64>>,

    /// S&P 500 52-week change percentage
    #[serde(rename = "SandP52WeekChange", skip_serializing_if = "Option::is_none")]
    pub sand_p_52_week_change: Option<F::Value<f64>>,

    /// Annual holdings turnover (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annual_holdings_turnover: Option<Value>,

    /// Annual report expense ratio (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annual_report_expense_ratio: Option<Value>,

    /// Beta coefficient (volatility vs market)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beta: Option<F::Value<f64>>,

    /// 3-year beta (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beta3_year: Option<Value>,

    /// Book value per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_value: Option<F::Value<f64>>,

    /// Fund category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Date of short interest data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_short_interest: Option<F::Value<i64>>,

    /// Quarterly earnings growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_quarterly_growth: Option<F::Value<f64>>,

    /// Enterprise value to EBITDA ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_to_ebitda: Option<F::Value<f64>>,

    /// Enterprise value to revenue ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_to_revenue: Option<F::Value<f64>>,

    /// Total enterprise value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_value: Option<F::Value<i64>>,

    /// 5-year average return (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_year_average_return: Option<Value>,

    /// Number of floating shares
    #[serde(skip_serializing_if = "Option::is_none")]
    pub float_shares: Option<F::Value<i64>>,

    /// Forward earnings per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_eps: Option<F::Value<f64>>,

    /// Forward price-to-earnings ratio
    #[serde(rename = "forwardPE", skip_serializing_if = "Option::is_none")]
    pub forward_pe: Option<F::Value<f64>>,

    /// Fund family name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_family: Option<String>,

    /// Fund inception date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_inception_date: Option<Value>,

    /// Funding to date (for private companies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding_to_date: Option<Value>,

    /// Percentage of shares held by insiders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub held_percent_insiders: Option<F::Value<f64>>,

    /// Percentage of shares held by institutions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub held_percent_institutions: Option<F::Value<f64>>,

    /// Implied shares outstanding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implied_shares_outstanding: Option<F::Value<i64>>,

    /// Last capital gain (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_cap_gain: Option<Value>,

    /// Last dividend date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_dividend_date: Option<F::Value<i64>>,

    /// Last dividend value per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_dividend_value: Option<F::Value<f64>>,

    /// Last fiscal year end date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fiscal_year_end: Option<F::Value<i64>>,

    /// Last stock split date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_split_date: Option<F::Value<i64>>,

    /// Last stock split factor (e.g., "4:1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_split_factor: Option<String>,

    /// Latest amount raised (for private companies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_amount_raised: Option<Value>,

    /// Latest funding date (for private companies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_funding_date: Option<Value>,

    /// Latest implied valuation (for private companies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_implied_valuation: Option<Value>,

    /// Latest share class
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_share_class: Option<String>,

    /// Lead investor (for private companies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lead_investor: Option<String>,

    /// Legal type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_type: Option<String>,

    /// Maximum age of data in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<i64>,

    /// Morningstar overall rating (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morning_star_overall_rating: Option<Value>,

    /// Morningstar risk rating (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morning_star_risk_rating: Option<Value>,

    /// Most recent quarter end date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub most_recent_quarter: Option<F::Value<i64>>,

    /// Net income to common shareholders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_income_to_common: Option<F::Value<i64>>,

    /// Next fiscal year end date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_fiscal_year_end: Option<F::Value<i64>>,

    /// PEG ratio (Price/Earnings to Growth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peg_ratio: Option<Value>,

    /// Price hint (decimal places)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_hint: Option<F::Value<i64>>,

    /// Price to book ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_to_book: Option<F::Value<f64>>,

    /// Price to sales ratio (trailing 12 months)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_to_sales_trailing12_months: Option<Value>,

    /// Profit margins percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_margins: Option<F::Value<f64>>,

    /// Quarter-to-date return (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qtd_return: Option<Value>,

    /// Quarterly revenue growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_quarterly_growth: Option<Value>,

    /// Total shares outstanding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_outstanding: Option<F::Value<i64>>,

    /// Short interest as percentage of shares outstanding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_percent_shares_out: Option<F::Value<f64>>,

    /// Number of shares short
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_short: Option<F::Value<i64>>,

    /// Previous month date for short interest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_short_previous_month_date: Option<F::Value<i64>>,

    /// Shares short in prior month
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_short_prior_month: Option<F::Value<i64>>,

    /// Short interest as percentage of float
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_percent_of_float: Option<F::Value<f64>>,

    /// Short ratio (days to cover)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_ratio: Option<F::Value<f64>>,

    /// 3-year average return (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub three_year_average_return: Option<Value>,

    /// Total assets (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_assets: Option<Value>,

    /// Total funding rounds (for private companies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_funding_rounds: Option<Value>,

    /// Trailing earnings per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_eps: Option<F::Value<f64>>,

    /// Yield percentage (for bonds/funds)
    #[serde(rename = "yield", skip_serializing_if = "Option::is_none")]
    pub yield_value: Option<Value>,

    /// Year-to-date return (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_default_key_statistics() {
        let json = r#"{
            "52WeekChange": {"fmt": "17.38%", "raw": 0.173828},
            "SandP52WeekChange": {"fmt": "11.35%", "raw": 0.11350584},
            "beta": {"fmt": "1.11", "raw": 1.109},
            "bookValue": {"fmt": "4.99", "raw": 4.991},
            "enterpriseValue": {"fmt": "4.13T", "longFmt": "4,134,771,359,744", "raw": 4134771359744},
            "forwardPE": {"fmt": "30.39", "raw": 30.387243},
            "lastSplitFactor": "4:1",
            "maxAge": 1,
            "sharesOutstanding": {"fmt": "14.78B", "longFmt": "14,776,353,000", "raw": 14776353000},
            "trailingEps": {"fmt": "7.45", "raw": 7.45}
        }"#;

        let stats: DefaultKeyStatistics = serde_json::from_str(json).unwrap();
        assert_eq!(
            stats.week_52_change.as_ref().map(|v| v.raw),
            Some(Some(0.173828))
        );
        assert_eq!(stats.beta.as_ref().map(|v| v.raw), Some(Some(1.109)));
        assert_eq!(stats.last_split_factor.as_deref(), Some("4:1"));
        assert_eq!(stats.max_age, Some(1));
        assert_eq!(
            stats.shares_outstanding.as_ref().map(|v| v.raw),
            Some(Some(14776353000))
        );
    }

    #[test]
    fn test_into_raw() {
        let json = r#"{
            "beta": {"fmt": "1.11", "raw": 1.109},
            "trailingEps": {"fmt": "7.45", "raw": 7.45},
            "lastSplitFactor": "4:1"
        }"#;

        let stats: DefaultKeyStatistics = serde_json::from_str(json).unwrap();
        let raw = stats.into_raw();
        assert_eq!(raw.beta, Some(1.109));
        assert_eq!(raw.trailing_eps, Some(7.45));
        assert_eq!(raw.last_split_factor.as_deref(), Some("4:1"));
    }

    #[test]
    fn test_into_pretty() {
        let json = r#"{
            "beta": {"fmt": "1.11", "raw": 1.109},
            "enterpriseValue": {"fmt": "4.13T", "longFmt": "4,134,771,359,744", "raw": 4134771359744}
        }"#;

        let stats: DefaultKeyStatistics = serde_json::from_str(json).unwrap();
        let pretty = stats.into_pretty();
        assert_eq!(pretty.beta.as_deref(), Some("1.11"));
        assert_eq!(pretty.enterprise_value.as_deref(), Some("4.13T"));
    }
}
