use crate::models::quote::FormattedValue;
use serde::{Deserialize, Serialize};

// ============================================================================
// Raw response structs (private) - for parsing Yahoo's nested structure
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct RawSectorResponse {
    data: RawSectorData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSectorData {
    name: String,
    symbol: Option<String>,
    key: String,
    overview: Option<RawOverview>,
    performance: Option<RawPerformance>,
    #[serde(default)]
    performance_overview_benchmark: Option<RawBenchmarkPerformance>,
    #[serde(default)]
    top_companies: Vec<RawCompany>,
    #[serde(default, rename = "topETFs")]
    top_etfs: Vec<RawETF>,
    #[serde(default)]
    top_mutual_funds: Vec<RawMutualFund>,
    #[serde(default)]
    industries: Vec<RawIndustry>,
    #[serde(default)]
    research_reports: Vec<RawResearchReport>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawOverview {
    companies_count: Option<i64>,
    market_cap: Option<FormattedValue<f64>>,
    description: Option<String>,
    industries_count: Option<i64>,
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
    market_cap: Option<FormattedValue<f64>>,
    market_weight: Option<FormattedValue<f64>>,
    last_price: Option<FormattedValue<f64>>,
    target_price: Option<FormattedValue<f64>>,
    reg_market_change_percent: Option<FormattedValue<f64>>,
    ytd_return: Option<FormattedValue<f64>>,
    rating: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawETF {
    symbol: String,
    name: Option<String>,
    net_assets: Option<FormattedValue<f64>>,
    expense_ratio: Option<FormattedValue<f64>>,
    last_price: Option<FormattedValue<f64>>,
    ytd_return: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMutualFund {
    symbol: String,
    name: Option<String>,
    net_assets: Option<FormattedValue<f64>>,
    expense_ratio: Option<FormattedValue<f64>>,
    last_price: Option<FormattedValue<f64>>,
    ytd_return: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIndustry {
    symbol: Option<String>,
    key: Option<String>,
    name: String,
    market_weight: Option<FormattedValue<f64>>,
    reg_market_change_percent: Option<FormattedValue<f64>>,
    ytd_return: Option<FormattedValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawResearchReport {
    id: String,
    head_html: Option<String>,
    provider: Option<String>,
    report_date: Option<String>,
    report_title: Option<String>,
    report_type: Option<String>,
    target_price: Option<f64>,
    target_price_status: Option<String>,
    investment_rating: Option<String>,
}

// ============================================================================
// Public response structs - clean, user-friendly types
// ============================================================================

/// Complete sector data with all available information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Sector {
    /// Sector name (e.g., "Technology")
    pub name: String,

    /// Yahoo Finance sector symbol (e.g., "^YH311")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// Sector key for API calls (e.g., "technology")
    pub key: String,

    /// Sector overview with market statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<SectorOverview>,

    /// Sector performance metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<SectorPerformance>,

    /// Benchmark (S&P 500) comparison performance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark: Option<SectorPerformance>,

    /// Benchmark name (usually "S&P 500")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark_name: Option<String>,

    /// Top companies in the sector
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top_companies: Vec<SectorCompany>,

    /// Top ETFs tracking this sector
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top_etfs: Vec<SectorETF>,

    /// Top mutual funds in this sector
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top_mutual_funds: Vec<SectorMutualFund>,

    /// Industries within this sector
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub industries: Vec<SectorIndustry>,

    /// Recent research reports
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub research_reports: Vec<ResearchReport>,
}

/// Sector overview statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorOverview {
    /// Number of companies in the sector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub companies_count: Option<i64>,

    /// Total market capitalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<FormattedValue<f64>>,

    /// Sector description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Number of industries in the sector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industries_count: Option<i64>,

    /// Market weight (percentage of total market)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_weight: Option<FormattedValue<f64>>,

    /// Total employee count across sector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_count: Option<FormattedValue<i64>>,
}

/// Sector performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorPerformance {
    /// Year-to-date change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_change_percent: Option<FormattedValue<f64>>,

    /// Regular market change percentage (today)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_change_percent: Option<FormattedValue<f64>>,

    /// One year change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_year_change_percent: Option<FormattedValue<f64>>,

    /// Three year change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub three_year_change_percent: Option<FormattedValue<f64>>,

    /// Five year change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_year_change_percent: Option<FormattedValue<f64>>,
}

/// A company in the sector's top companies list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorCompany {
    /// Stock symbol
    pub symbol: String,

    /// Company name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Market capitalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<FormattedValue<f64>>,

    /// Weight in sector (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_weight: Option<FormattedValue<f64>>,

    /// Last traded price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<FormattedValue<f64>>,

    /// Analyst target price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price: Option<FormattedValue<f64>>,

    /// Day change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_change_percent: Option<FormattedValue<f64>>,

    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<FormattedValue<f64>>,

    /// Analyst rating (e.g., "Strong Buy", "Buy", "Hold")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
}

/// An ETF tracking the sector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorETF {
    /// ETF symbol
    pub symbol: String,

    /// ETF name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Net assets under management
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_assets: Option<FormattedValue<f64>>,

    /// Expense ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expense_ratio: Option<FormattedValue<f64>>,

    /// Last traded price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<FormattedValue<f64>>,

    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<FormattedValue<f64>>,
}

/// A mutual fund in the sector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorMutualFund {
    /// Fund symbol
    pub symbol: String,

    /// Fund name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Net assets under management
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_assets: Option<FormattedValue<f64>>,

    /// Expense ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expense_ratio: Option<FormattedValue<f64>>,

    /// Last traded price (NAV)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_price: Option<FormattedValue<f64>>,

    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<FormattedValue<f64>>,
}

/// An industry within the sector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SectorIndustry {
    /// Industry symbol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// Industry key for API calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Industry name
    pub name: String,

    /// Weight in sector (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_weight: Option<FormattedValue<f64>>,

    /// Day change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_change_percent: Option<FormattedValue<f64>>,

    /// Year-to-date return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ytd_return: Option<FormattedValue<f64>>,
}

/// A research report about the sector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResearchReport {
    /// Report ID
    pub id: String,

    /// Report headline/summary (may contain HTML)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,

    /// Research provider (e.g., "Argus Research", "Morningstar")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Report publication date (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_date: Option<String>,

    /// Full report title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_title: Option<String>,

    /// Report type (e.g., "Technical Analysis", "Analyst Report")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_type: Option<String>,

    /// Target price (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price: Option<f64>,

    /// Target price status (e.g., "Maintained", "Raised")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_price_status: Option<String>,

    /// Investment rating (e.g., "Bullish", "Bearish")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub investment_rating: Option<String>,
}

// ============================================================================
// Conversion implementations
// ============================================================================

impl Sector {
    /// Parse Yahoo Finance sector response JSON
    pub(crate) fn from_response(json: &serde_json::Value) -> Result<Self, String> {
        let raw: RawSectorResponse = serde_json::from_value(json.clone())
            .map_err(|e| format!("Failed to parse sector response: {}", e))?;

        let data = raw.data;

        // Convert overview
        let overview = data.overview.map(|o| SectorOverview {
            companies_count: o.companies_count,
            market_cap: o.market_cap,
            description: o.description,
            industries_count: o.industries_count,
            market_weight: o.market_weight,
            employee_count: o.employee_count,
        });

        // Convert performance
        let performance = data.performance.map(|p| SectorPerformance {
            ytd_change_percent: p.ytd_change_percent,
            day_change_percent: p.reg_market_change_percent,
            one_year_change_percent: p.one_year_change_percent,
            three_year_change_percent: p.three_year_change_percent,
            five_year_change_percent: p.five_year_change_percent,
        });

        // Convert benchmark
        let (benchmark, benchmark_name) = match data.performance_overview_benchmark {
            Some(b) => (
                Some(SectorPerformance {
                    ytd_change_percent: b.ytd_change_percent,
                    day_change_percent: b.reg_market_change_percent,
                    one_year_change_percent: b.one_year_change_percent,
                    three_year_change_percent: b.three_year_change_percent,
                    five_year_change_percent: b.five_year_change_percent,
                }),
                b.name,
            ),
            None => (None, None),
        };

        // Convert top companies
        let top_companies = data
            .top_companies
            .into_iter()
            .map(|c| SectorCompany {
                symbol: c.symbol,
                name: c.name,
                market_cap: c.market_cap,
                market_weight: c.market_weight,
                last_price: c.last_price,
                target_price: c.target_price,
                day_change_percent: c.reg_market_change_percent,
                ytd_return: c.ytd_return,
                rating: c.rating,
            })
            .collect();

        // Convert ETFs
        let top_etfs = data
            .top_etfs
            .into_iter()
            .map(|e| SectorETF {
                symbol: e.symbol,
                name: e.name,
                net_assets: e.net_assets,
                expense_ratio: e.expense_ratio,
                last_price: e.last_price,
                ytd_return: e.ytd_return,
            })
            .collect();

        // Convert mutual funds
        let top_mutual_funds = data
            .top_mutual_funds
            .into_iter()
            .map(|f| SectorMutualFund {
                symbol: f.symbol,
                name: f.name,
                net_assets: f.net_assets,
                expense_ratio: f.expense_ratio,
                last_price: f.last_price,
                ytd_return: f.ytd_return,
            })
            .collect();

        // Convert industries
        let industries = data
            .industries
            .into_iter()
            .map(|i| SectorIndustry {
                symbol: i.symbol,
                key: i.key,
                name: i.name,
                market_weight: i.market_weight,
                day_change_percent: i.reg_market_change_percent,
                ytd_return: i.ytd_return,
            })
            .collect();

        // Convert research reports
        let research_reports = data
            .research_reports
            .into_iter()
            .map(|r| ResearchReport {
                id: r.id,
                headline: r.head_html,
                provider: r.provider,
                report_date: r.report_date,
                report_title: r.report_title,
                report_type: r.report_type,
                target_price: r.target_price,
                target_price_status: r.target_price_status,
                investment_rating: r.investment_rating,
            })
            .collect();

        Ok(Self {
            name: data.name,
            symbol: data.symbol,
            key: data.key,
            overview,
            performance,
            benchmark,
            benchmark_name,
            top_companies,
            top_etfs,
            top_mutual_funds,
            industries,
            research_reports,
        })
    }
}
