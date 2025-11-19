//! Quote Summary Module
//!
//! Contains all data structures and enums for Yahoo Finance's quoteSummary endpoint.

mod price;
mod response;

pub use price::Price;
pub use response::{QuoteSummaryData, QuoteSummaryResponse};

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
    /// Environmental, social, and governance metrics
    EsgScores,
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
    /// Detailed pricing data (exchange, quote type, currency, market cap, etc.)
    Price,
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
            Module::EsgScores => "esgScores",
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
            Module::Price => "price",
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
            Module::EsgScores,
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
            Module::Price,
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
            Module::Price,
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
        assert_eq!(Module::Price.as_str(), "price");
        assert_eq!(Module::SummaryDetail.as_str(), "summaryDetail");
        assert_eq!(Module::KeyStats.as_str(), "defaultKeyStatistics");
    }

    #[test]
    fn test_module_all() {
        let all_modules = Module::all();
        assert!(all_modules.len() > 30);
        assert!(all_modules.contains(&Module::Price));
    }

    #[test]
    fn test_module_core() {
        let core_modules = Module::core();
        assert_eq!(core_modules.len(), 5);
        assert!(core_modules.contains(&Module::Price));
    }
}
