use serde::{Deserialize, Serialize};

use super::FormattedValue;

/// Fund performance data including returns, risk metrics, and historical performance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundPerformance {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Fund category name (e.g., "Large Blend")
    #[serde(default)]
    pub fund_category_name: Option<String>,

    /// Performance overview for this fund
    #[serde(default)]
    pub performance_overview: Option<PerformanceOverview>,

    /// Performance overview for the fund's category average
    #[serde(default)]
    pub performance_overview_cat: Option<PerformanceOverviewCat>,

    /// Trailing returns (market price)
    #[serde(default)]
    pub trailing_returns: Option<TrailingReturns>,

    /// Trailing returns (NAV - Net Asset Value)
    #[serde(default)]
    pub trailing_returns_nav: Option<TrailingReturnsNav>,

    /// Trailing returns for the fund's category average
    #[serde(default)]
    pub trailing_returns_cat: Option<TrailingReturnsCat>,

    /// Annual total returns by year
    #[serde(default)]
    pub annual_total_returns: Option<AnnualTotalReturns>,

    /// Quarterly returns
    #[serde(default)]
    pub past_quarterly_returns: Option<PastQuarterlyReturns>,

    /// Risk statistics (alpha, beta, sharpe ratio, etc.)
    #[serde(default)]
    pub risk_overview_statistics: Option<RiskOverviewStatistics>,

    /// Risk statistics for the fund's category average
    #[serde(default)]
    pub risk_overview_statistics_cat: Option<RiskOverviewStatisticsCat>,
}

/// Performance overview with key return metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceOverview {
    /// As of date (Unix timestamp)
    #[serde(default)]
    pub as_of_date: Option<FormattedValue<i64>>,

    /// Year-to-date return percentage
    #[serde(default)]
    pub ytd_return_pct: Option<FormattedValue<f64>>,

    /// 5-year average return percentage
    #[serde(default)]
    pub five_yr_avg_return_pct: Option<FormattedValue<f64>>,

    /// 1-year total return
    #[serde(default)]
    pub one_year_total_return: Option<FormattedValue<f64>>,

    /// 3-year total return
    #[serde(default)]
    pub three_year_total_return: Option<FormattedValue<f64>>,
}

/// Category average performance overview
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceOverviewCat {
    /// Year-to-date return percentage (category average)
    #[serde(default)]
    pub ytd_return_pct: Option<FormattedValue<f64>>,

    /// 5-year average return percentage (category average)
    #[serde(default)]
    pub five_yr_avg_return_pct: Option<FormattedValue<f64>>,

    /// 1-year total return (category average)
    #[serde(default)]
    pub one_year_total_return: Option<FormattedValue<f64>>,

    /// 3-year total return (category average)
    #[serde(default)]
    pub three_year_total_return: Option<FormattedValue<f64>>,
}

/// Trailing returns at market price
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingReturns {
    /// As of date (Unix timestamp)
    #[serde(default)]
    pub as_of_date: Option<FormattedValue<i64>>,

    /// Year-to-date return
    #[serde(default)]
    pub ytd: Option<FormattedValue<f64>>,

    /// 1-month return
    #[serde(default)]
    pub one_month: Option<FormattedValue<f64>>,

    /// 3-month return
    #[serde(default)]
    pub three_month: Option<FormattedValue<f64>>,

    /// 1-year return
    #[serde(default)]
    pub one_year: Option<FormattedValue<f64>>,

    /// 3-year return
    #[serde(default)]
    pub three_year: Option<FormattedValue<f64>>,

    /// 5-year return
    #[serde(default)]
    pub five_year: Option<FormattedValue<f64>>,

    /// 10-year return
    #[serde(default)]
    pub ten_year: Option<FormattedValue<f64>>,

    /// Return during last bull market
    #[serde(default)]
    pub last_bull_mkt: Option<FormattedValue<f64>>,

    /// Return during last bear market
    #[serde(default)]
    pub last_bear_mkt: Option<FormattedValue<f64>>,
}

/// Trailing returns at NAV (Net Asset Value)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingReturnsNav {
    /// Year-to-date return
    #[serde(default)]
    pub ytd: Option<FormattedValue<f64>>,

    /// 1-month return
    #[serde(default)]
    pub one_month: Option<FormattedValue<f64>>,

    /// 3-month return
    #[serde(default)]
    pub three_month: Option<FormattedValue<f64>>,

    /// 1-year return
    #[serde(default)]
    pub one_year: Option<FormattedValue<f64>>,

    /// 3-year return
    #[serde(default)]
    pub three_year: Option<FormattedValue<f64>>,

    /// 5-year return
    #[serde(default)]
    pub five_year: Option<FormattedValue<f64>>,

    /// 10-year return
    #[serde(default)]
    pub ten_year: Option<FormattedValue<f64>>,
}

/// Category average trailing returns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailingReturnsCat {
    /// Year-to-date return (category average)
    #[serde(default)]
    pub ytd: Option<FormattedValue<f64>>,

    /// 1-month return (category average)
    #[serde(default)]
    pub one_month: Option<FormattedValue<f64>>,

    /// 3-month return (category average)
    #[serde(default)]
    pub three_month: Option<FormattedValue<f64>>,

    /// 1-year return (category average)
    #[serde(default)]
    pub one_year: Option<FormattedValue<f64>>,

    /// 3-year return (category average)
    #[serde(default)]
    pub three_year: Option<FormattedValue<f64>>,

    /// 5-year return (category average)
    #[serde(default)]
    pub five_year: Option<FormattedValue<f64>>,

    /// 10-year return (category average)
    #[serde(default)]
    pub ten_year: Option<FormattedValue<f64>>,

    /// Return during last bull market (category average)
    #[serde(default)]
    pub last_bull_mkt: Option<FormattedValue<f64>>,

    /// Return during last bear market (category average)
    #[serde(default)]
    pub last_bear_mkt: Option<FormattedValue<f64>>,
}

/// Annual total returns by year
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnualTotalReturns {
    /// Annual returns for this fund
    #[serde(default)]
    pub returns: Option<Vec<AnnualReturn>>,

    /// Annual returns for the category average
    #[serde(default)]
    pub returns_cat: Option<Vec<AnnualReturn>>,
}

/// Single year's return data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnualReturn {
    /// Year (e.g., "2024")
    #[serde(default)]
    pub year: Option<String>,

    /// Annual return value
    #[serde(default)]
    pub annual_value: Option<FormattedValue<f64>>,
}

/// Past quarterly returns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PastQuarterlyReturns {
    /// Quarterly returns
    #[serde(default)]
    pub returns: Option<Vec<serde_json::Value>>,
}

/// Risk overview statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskOverviewStatistics {
    /// Risk statistics for various time periods
    #[serde(default)]
    pub risk_statistics: Option<Vec<RiskStatistic>>,
}

/// Category average risk overview statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskOverviewStatisticsCat {
    /// Category average risk statistics
    #[serde(default)]
    pub risk_statistics_cat: Option<Vec<RiskStatistic>>,
}

/// Risk statistics for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskStatistic {
    /// Time period (e.g., "3y", "5y", "10y")
    #[serde(default)]
    pub year: Option<String>,

    /// Alpha - excess return relative to benchmark
    #[serde(default)]
    pub alpha: Option<FormattedValue<f64>>,

    /// Beta - volatility relative to benchmark
    #[serde(default)]
    pub beta: Option<FormattedValue<f64>>,

    /// Mean annual return
    #[serde(default)]
    pub mean_annual_return: Option<FormattedValue<f64>>,

    /// R-squared - correlation with benchmark (0-100)
    #[serde(default)]
    pub r_squared: Option<FormattedValue<f64>>,

    /// Standard deviation (volatility)
    #[serde(default)]
    pub std_dev: Option<FormattedValue<f64>>,

    /// Sharpe ratio - risk-adjusted return
    #[serde(default)]
    pub sharpe_ratio: Option<FormattedValue<f64>>,

    /// Treynor ratio - return per unit of systematic risk
    #[serde(default)]
    pub treynor_ratio: Option<FormattedValue<f64>>,
}
