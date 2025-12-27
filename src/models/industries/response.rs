use crate::models::quote::FormattedValue;
use serde::{Deserialize, Serialize};

// ============================================================================
// Raw response structs (private) - for parsing Yahoo's nested structure
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct RawIndustryResponse {
    data: RawIndustryData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIndustryData {
    name: String,
    symbol: Option<String>,
    key: String,
    sector_name: Option<String>,
    sector_key: Option<String>,
    overview: Option<RawOverview>,
    performance: Option<RawPerformance>,
    #[serde(default)]
    performance_overview_benchmark: Option<RawBenchmarkPerformance>,
    #[serde(default)]
    top_companies: Vec<RawCompany>,
    #[serde(default)]
    top_performing_companies: Vec<RawPerformingCompany>,
    #[serde(default)]
    top_growth_companies: Vec<RawGrowthCompany>,
    #[serde(default)]
    research_reports: Vec<RawResearchReport>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawOverview {
    companies_count: Option<i64>,
    market_cap: Option<FormattedValue<f64>>,
    description: Option<String>,
    market_weight: Option<FormattedValue<f64>>,
    employee_count: Option<FormattedValue<i64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPerformance {
    ytd_change_percent: Option<FormattedValue<f64>>,
    reg_market_change_percent: Option<FormattedValue<f64>>,
    three_year_change_percent: Option<FormattedValue<f64>>,
    one_year_change_percent: Option<FormattedValue<f64>>,
    five_year_change_percent: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawBenchmarkPerformance {
    name: Option<String>,
    ytd_change_percent: Option<FormattedValue<f64>>,
    reg_market_change_percent: Option<FormattedValue<f64>>,
    three_year_change_percent: Option<FormattedValue<f64>>,
    one_year_change_percent: Option<FormattedValue<f64>>,
    five_year_change_percent: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCompany {
    symbol: String,
    name: Option<String>,
    last_price: Option<FormattedValue<f64>>,
    market_cap: Option<FormattedValue<f64>>,
    market_weight: Option<FormattedValue<f64>>,
    #[serde(rename = "regMarketChangePercent")]
    day_change_percent: Option<FormattedValue<f64>>,
    ytd_return: Option<FormattedValue<f64>>,
    rating: Option<String>,
    target_price: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPerformingCompany {
    symbol: String,
    name: Option<String>,
    last_price: Option<FormattedValue<f64>>,
    ytd_return: Option<FormattedValue<f64>>,
    target_price: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGrowthCompany {
    symbol: String,
    name: Option<String>,
    last_price: Option<FormattedValue<f64>>,
    ytd_return: Option<FormattedValue<f64>>,
    growth_estimate: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawResearchReport {
    id: Option<String>,
    #[serde(rename = "reportTitle")]
    title: Option<String>,
    provider: Option<String>,
    report_date: Option<String>,
    report_type: Option<String>,
    investment_rating: Option<String>,
    target_price: Option<f64>,
    target_price_status: Option<String>,
}

// ============================================================================
// Public response structs - clean API surface
// ============================================================================

/// Industry data from Yahoo Finance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Industry {
    /// Industry name
    pub name: String,
    /// Industry key (URL slug)
    pub key: String,
    /// Industry index symbol (e.g., "^YH31130020")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Parent sector name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_name: Option<String>,
    /// Parent sector key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_key: Option<String>,
    /// Industry overview statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<IndustryOverview>,
    /// Industry performance metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<IndustryPerformance>,
    /// Benchmark performance (e.g., S&P 500)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark: Option<BenchmarkPerformance>,
    /// Top companies by market cap
    #[serde(default)]
    pub top_companies: Vec<IndustryCompany>,
    /// Top performing companies by YTD return
    #[serde(default)]
    pub top_performing_companies: Vec<PerformingCompany>,
    /// Top growth companies by growth estimate
    #[serde(default)]
    pub top_growth_companies: Vec<GrowthCompany>,
    /// Research reports
    #[serde(default)]
    pub research_reports: Vec<ResearchReport>,
}

/// Industry overview statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IndustryOverview {
    /// Description of the industry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Number of companies in the industry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companies_count: Option<i64>,
    /// Total market cap
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<f64>,
    /// Industry weight within the sector (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_weight: Option<f64>,
    /// Total employee count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_count: Option<i64>,
}

/// Industry performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IndustryPerformance {
    /// Daily change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_change_percent: Option<f64>,
    /// Year-to-date change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_change_percent: Option<f64>,
    /// 1-year change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_year_change_percent: Option<f64>,
    /// 3-year change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub three_year_change_percent: Option<f64>,
    /// 5-year change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_year_change_percent: Option<f64>,
}

/// Benchmark performance for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BenchmarkPerformance {
    /// Benchmark name (e.g., "S&P 500")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Daily change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_change_percent: Option<f64>,
    /// Year-to-date change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_change_percent: Option<f64>,
    /// 1-year change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_year_change_percent: Option<f64>,
    /// 3-year change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub three_year_change_percent: Option<f64>,
    /// 5-year change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_year_change_percent: Option<f64>,
}

/// Company within an industry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct IndustryCompany {
    /// Stock ticker symbol
    pub symbol: String,
    /// Company name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Last traded price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<f64>,
    /// Market capitalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<f64>,
    /// Weight within the industry (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_weight: Option<f64>,
    /// Daily change percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_change_percent: Option<f64>,
    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<f64>,
    /// Analyst rating
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    /// Analyst target price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price: Option<f64>,
}

/// Top performing company by YTD return
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PerformingCompany {
    /// Stock ticker symbol
    pub symbol: String,
    /// Company name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Last traded price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<f64>,
    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<f64>,
    /// Analyst target price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price: Option<f64>,
}

/// Top growth company by growth estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GrowthCompany {
    /// Stock ticker symbol
    pub symbol: String,
    /// Company name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Last traded price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<f64>,
    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<f64>,
    /// Growth estimate (as decimal, e.g., 3.0 = 300%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub growth_estimate: Option<f64>,
}

/// Research report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResearchReport {
    /// Report ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Report title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Report provider (e.g., "Argus Research")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Report date (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_date: Option<String>,
    /// Report type (e.g., "Quantitative Report")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_type: Option<String>,
    /// Investment rating (e.g., "Bullish", "Bearish", "Neutral")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub investment_rating: Option<String>,
    /// Target price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price: Option<f64>,
    /// Target price status (e.g., "Increased", "Maintained")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price_status: Option<String>,
}

// ============================================================================
// Conversion implementations
// ============================================================================

impl Industry {
    /// Parse from Yahoo Finance JSON response
    pub(crate) fn from_response(json: &serde_json::Value) -> Result<Self, String> {
        let raw: RawIndustryResponse =
            serde_json::from_value(json.clone()).map_err(|e| e.to_string())?;

        let data = raw.data;

        Ok(Industry {
            name: data.name,
            key: data.key,
            symbol: data.symbol,
            sector_name: data.sector_name,
            sector_key: data.sector_key,
            overview: data.overview.map(|o| IndustryOverview {
                description: o.description,
                companies_count: o.companies_count,
                market_cap: o.market_cap.and_then(|v| v.raw),
                market_weight: o.market_weight.and_then(|v| v.raw),
                employee_count: o.employee_count.and_then(|v| v.raw),
            }),
            performance: data.performance.map(|p| IndustryPerformance {
                day_change_percent: p.reg_market_change_percent.and_then(|v| v.raw),
                ytd_change_percent: p.ytd_change_percent.and_then(|v| v.raw),
                one_year_change_percent: p.one_year_change_percent.and_then(|v| v.raw),
                three_year_change_percent: p.three_year_change_percent.and_then(|v| v.raw),
                five_year_change_percent: p.five_year_change_percent.and_then(|v| v.raw),
            }),
            benchmark: data
                .performance_overview_benchmark
                .map(|b| BenchmarkPerformance {
                    name: b.name,
                    day_change_percent: b.reg_market_change_percent.and_then(|v| v.raw),
                    ytd_change_percent: b.ytd_change_percent.and_then(|v| v.raw),
                    one_year_change_percent: b.one_year_change_percent.and_then(|v| v.raw),
                    three_year_change_percent: b.three_year_change_percent.and_then(|v| v.raw),
                    five_year_change_percent: b.five_year_change_percent.and_then(|v| v.raw),
                }),
            top_companies: data
                .top_companies
                .into_iter()
                .map(|c| IndustryCompany {
                    symbol: c.symbol,
                    name: c.name,
                    last_price: c.last_price.and_then(|v| v.raw),
                    market_cap: c.market_cap.and_then(|v| v.raw),
                    market_weight: c.market_weight.and_then(|v| v.raw),
                    day_change_percent: c.day_change_percent.and_then(|v| v.raw),
                    ytd_return: c.ytd_return.and_then(|v| v.raw),
                    rating: c.rating,
                    target_price: c.target_price.and_then(|v| v.raw),
                })
                .collect(),
            top_performing_companies: data
                .top_performing_companies
                .into_iter()
                .map(|c| PerformingCompany {
                    symbol: c.symbol,
                    name: c.name,
                    last_price: c.last_price.and_then(|v| v.raw),
                    ytd_return: c.ytd_return.and_then(|v| v.raw),
                    target_price: c.target_price.and_then(|v| v.raw),
                })
                .collect(),
            top_growth_companies: data
                .top_growth_companies
                .into_iter()
                .map(|c| GrowthCompany {
                    symbol: c.symbol,
                    name: c.name,
                    last_price: c.last_price.and_then(|v| v.raw),
                    ytd_return: c.ytd_return.and_then(|v| v.raw),
                    growth_estimate: c.growth_estimate.and_then(|v| v.raw),
                })
                .collect(),
            research_reports: data
                .research_reports
                .into_iter()
                .map(|r| ResearchReport {
                    id: r.id,
                    title: r.title,
                    provider: r.provider,
                    report_date: r.report_date,
                    report_type: r.report_type,
                    investment_rating: r.investment_rating,
                    target_price: r.target_price,
                    target_price_status: r.target_price_status,
                })
                .collect(),
        })
    }
}
