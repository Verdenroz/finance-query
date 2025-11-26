//! Quote Summary Module
//!
//! Contains all data structures and enums for Yahoo Finance's quoteSummary endpoint.

mod data;
mod default_key_statistics;
mod financial_data;
mod formatted_value;
mod price;
mod quote_type_module;
mod response;
mod summary_detail;
mod summary_profile;

pub use data::Quote;
pub use default_key_statistics::DefaultKeyStatistics;
pub use financial_data::FinancialData;
pub use formatted_value::FormattedValue;
pub use price::Price;
pub use quote_type_module::QuoteTypeData;
pub use response::QuoteSummaryResponse;
pub use summary_detail::SummaryDetail;
pub use summary_profile::SummaryProfile;

/// All available modules from Yahoo Finance's quoteSummary endpoint
///
/// These correspond to the different data categories available for a stock symbol.
/// See: https://yahooquery.dpguthrie.com/guide/ticker/modules/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Module {
    /// Company information, location, operations, and officers
    AssetProfile,
    /// Earnings and revenue expectations for upcoming earnings date
    CalendarEvents,
    /// Top executives and their compensation
    CompanyOfficers,
    /// Historical earnings (actual vs. estimate)
    EarningHistory,
    /// Historical earnings data
    Earnings,
    /// Historical trend data for earnings and revenue estimations
    EarningsTrend,
    /// Financial KPIs (PE, enterprise value, EPS, EBITA, etc.)
    FinancialData,
    /// Aggregated maturity and duration information (funds/ETFs)
    FundBondHoldings,
    /// Bond rating information (funds/ETFs)
    FundBondRatings,
    /// Equity holdings information (funds/ETFs)
    FundEquityHoldings,
    /// Fund holdings information including top holdings
    FundHoldingInfo,
    /// Top 10 fund owners
    FundOwnership,
    /// Historical return data for funds
    FundPerformance,
    /// Summary level information for funds
    FundProfile,
    /// Sector weightings for funds
    FundSectorWeightings,
    /// Top 10 holdings for funds
    FundTopHoldings,
    /// Upgrades/downgrades by companies
    GradingHistory,
    /// Trend data related to symbol's index (PE and PEG ratios)
    IndexTrend,
    /// Industry trend data
    IndustryTrend,
    /// Stock holdings of insiders
    InsiderHolders,
    /// Transactions by insiders
    InsiderTransactions,
    /// Top 10 institutional owners
    InstitutionOwnership,
    /// Key performance indicators
    KeyStats,
    /// Breakdown of owners (insiders, institutions, etc.)
    MajorHolders,
    /// Short, mid, and long-term trend data for page views
    PageViews,
    /// Stock exchange specific data
    QuoteType,
    /// Historical buy/hold/sell recommendations
    RecommendationTrend,
    /// Historical SEC filings
    SecFilings,
    /// High-level buy/sell data for insiders
    SharePurchaseActivity,
    /// Summary tab information
    SummaryDetail,
    /// Company location and business summary
    SummaryProfile,
}

impl Module {
    /// Converts the module enum to the API parameter string
    pub fn as_str(&self) -> &'static str {
        match self {
            Module::AssetProfile => "assetProfile",
            Module::CalendarEvents => "calendarEvents",
            Module::CompanyOfficers => "companyOfficers",
            Module::EarningHistory => "earningsHistory",
            Module::Earnings => "earnings",
            Module::EarningsTrend => "earningsTrend",
            Module::FinancialData => "financialData",
            Module::FundBondHoldings => "fundBondHoldings",
            Module::FundBondRatings => "fundBondRatings",
            Module::FundEquityHoldings => "fundEquityHoldings",
            Module::FundHoldingInfo => "fundHoldingInfo",
            Module::FundOwnership => "fundOwnership",
            Module::FundPerformance => "fundPerformance",
            Module::FundProfile => "fundProfile",
            Module::FundSectorWeightings => "fundSectorWeightings",
            Module::FundTopHoldings => "fundTopHoldings",
            Module::GradingHistory => "upgradeDowngradeHistory",
            Module::IndexTrend => "indexTrend",
            Module::IndustryTrend => "industryTrend",
            Module::InsiderHolders => "insiderHolders",
            Module::InsiderTransactions => "insiderTransactions",
            Module::InstitutionOwnership => "institutionOwnership",
            Module::KeyStats => "defaultKeyStatistics",
            Module::MajorHolders => "majorHoldersBreakdown",
            Module::PageViews => "pageViews",
            Module::QuoteType => "quoteType",
            Module::RecommendationTrend => "recommendationTrend",
            Module::SecFilings => "secFilings",
            Module::SharePurchaseActivity => "netSharePurchaseActivity",
            Module::SummaryDetail => "summaryDetail",
            Module::SummaryProfile => "summaryProfile",
        }
    }

    /// Returns all available modules
    pub fn all() -> Vec<Module> {
        vec![
            Module::AssetProfile,
            Module::CalendarEvents,
            Module::CompanyOfficers,
            Module::EarningHistory,
            Module::Earnings,
            Module::EarningsTrend,
            Module::FinancialData,
            Module::FundBondHoldings,
            Module::FundBondRatings,
            Module::FundEquityHoldings,
            Module::FundHoldingInfo,
            Module::FundOwnership,
            Module::FundPerformance,
            Module::FundProfile,
            Module::FundSectorWeightings,
            Module::FundTopHoldings,
            Module::GradingHistory,
            Module::IndexTrend,
            Module::IndustryTrend,
            Module::InsiderHolders,
            Module::InsiderTransactions,
            Module::InstitutionOwnership,
            Module::KeyStats,
            Module::MajorHolders,
            Module::PageViews,
            Module::QuoteType,
            Module::RecommendationTrend,
            Module::SecFilings,
            Module::SharePurchaseActivity,
            Module::SummaryDetail,
            Module::SummaryProfile,
        ]
    }

    /// Returns the most commonly used core modules
    pub fn core() -> Vec<Module> {
        vec![
            Module::SummaryDetail,
            Module::FinancialData,
            Module::KeyStats,
            Module::AssetProfile,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_as_str() {
        assert_eq!(Module::SummaryDetail.as_str(), "summaryDetail");
        assert_eq!(Module::KeyStats.as_str(), "defaultKeyStatistics");
    }

    #[test]
    fn test_module_all() {
        let all_modules = Module::all();
        assert!(all_modules.len() > 30);
    }

    #[test]
    fn test_module_core() {
        let core_modules = Module::core();
        assert_eq!(core_modules.len(), 4);
    }
}
