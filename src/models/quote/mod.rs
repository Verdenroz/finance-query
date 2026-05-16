//! Quote models.
//!
//! Contains all data structures and enums for Yahoo Finance's quoteSummary endpoint.

// Internal modules that remain in quote/
pub(crate) mod price;
pub(crate) mod quote_type;
pub(crate) mod response;

// Public modules
pub mod data;
/// Formatted value wrapper for Yahoo Finance numeric fields.
pub mod formatted_value;

// Re-export only the final flattened Quote struct and FormattedValue (used in Quote's public fields)
pub use data::Quote;
pub use formatted_value::FormattedValue;

// ── Re-exports from new canonical locations (backward compat within crate) ───

// From fundamentals/
pub(crate) use crate::models::fundamentals::{
    BalanceSheetHistory, BalanceSheetHistoryQuarterly, CashflowStatementHistory,
    CashflowStatementHistoryQuarterly, DefaultKeyStatistics, FinancialData, IncomeStatementHistory,
    IncomeStatementHistoryQuarterly, SummaryDetail,
};

// From corporate/
pub(crate) use crate::models::corporate::{
    AssetProfile, CalendarEvents, CompanyOfficer, Earnings, EarningsHistory, EarningsTrend,
    EquityPerformance, FundOwnership, FundPerformance, FundProfile, InsiderHolders,
    InsiderTransactions, InstitutionOwnership, MajorHoldersBreakdown, NetSharePurchaseActivity,
    RecommendationTrend, SecFilings, SummaryProfile, TopHoldings, UpgradeDowngradeHistory,
};

// From market/
pub(crate) use crate::models::market::{IndexTrend, IndustryTrend, SectorTrend};

// Internal re-exports for crate use
pub(crate) use price::Price;
pub(crate) use quote_type::QuoteTypeData;
pub(crate) use response::QuoteSummaryResponse;

/// All available modules from Yahoo Finance's quoteSummary endpoint
///
/// These correspond to the different data categories available for a stock symbol.
/// See: https://yahooquery.dpguthrie.com/guide/ticker/modules/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Module {
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
    /// Equity performance vs benchmark across multiple time periods
    EquityPerformance,
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
    /// Historical balance sheets (annual)
    BalanceSheetHistory,
    /// Quarterly balance sheets
    BalanceSheetHistoryQuarterly,
    /// Historical cash flow statements (annual)
    CashflowStatementHistory,
    /// Quarterly cash flow statements
    CashflowStatementHistoryQuarterly,
    /// Historical income statements (annual)
    IncomeStatementHistory,
    /// Quarterly income statements
    IncomeStatementHistoryQuarterly,
    /// ESG (Environmental, Social, Governance) scores
    EsgScores,
    /// Real-time price data
    Price,
    /// Sector trend data
    SectorTrend,
    /// Top holdings for funds
    TopHoldings,
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
            Module::EquityPerformance => "equityPerformance",
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
            Module::QuoteType => "quoteType",
            Module::RecommendationTrend => "recommendationTrend",
            Module::SecFilings => "secFilings",
            Module::SharePurchaseActivity => "netSharePurchaseActivity",
            Module::SummaryDetail => "summaryDetail",
            Module::SummaryProfile => "summaryProfile",
            Module::BalanceSheetHistory => "balanceSheetHistory",
            Module::BalanceSheetHistoryQuarterly => "balanceSheetHistoryQuarterly",
            Module::CashflowStatementHistory => "cashflowStatementHistory",
            Module::CashflowStatementHistoryQuarterly => "cashflowStatementHistoryQuarterly",
            Module::IncomeStatementHistory => "incomeStatementHistory",
            Module::IncomeStatementHistoryQuarterly => "incomeStatementHistoryQuarterly",
            Module::EsgScores => "esgScores",
            Module::Price => "price",
            Module::SectorTrend => "sectorTrend",
            Module::TopHoldings => "topHoldings",
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
            Module::EquityPerformance,
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
            Module::QuoteType,
            Module::RecommendationTrend,
            Module::SecFilings,
            Module::SharePurchaseActivity,
            Module::SummaryDetail,
            Module::SummaryProfile,
            Module::BalanceSheetHistory,
            Module::BalanceSheetHistoryQuarterly,
            Module::CashflowStatementHistory,
            Module::CashflowStatementHistoryQuarterly,
            Module::IncomeStatementHistory,
            Module::IncomeStatementHistoryQuarterly,
            Module::EsgScores,
            Module::Price,
            Module::SectorTrend,
            Module::TopHoldings,
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
}
