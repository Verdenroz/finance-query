//! Quote Summary Module
//!
//! Contains all data structures and enums for Yahoo Finance's quoteSummary endpoint.

mod asset_profile;
mod balance_sheet_history;
mod calendar_events;
mod cashflow_statement_history;
mod data;
mod default_key_statistics;
mod earnings;
mod earnings_history;
mod earnings_trend;
mod equity_performance;
mod financial_data;
mod formatted_value;
mod fund_ownership;
mod fund_performance;
mod fund_profile;
mod income_statement_history;
mod index_trend;
mod insider_holders;
mod insider_transactions;
mod institution_ownership;
mod major_holders_breakdown;
mod net_share_purchase_activity;
mod price;
mod quote_type;
mod recommendation_trend;
mod response;
mod sec_filings;
mod summary_detail;
mod summary_profile;
mod top_holdings;
mod upgrade_downgrade_history;

pub use asset_profile::{AssetProfile, CompanyOfficer};
pub use balance_sheet_history::{BalanceSheetHistory, BalanceSheetHistoryQuarterly};
pub use calendar_events::{CalendarEvents, EarningsCalendar};
pub use cashflow_statement_history::{CashflowStatementHistory, CashflowStatementHistoryQuarterly};
pub use data::Quote;
pub use default_key_statistics::DefaultKeyStatistics;
pub use earnings::{
    Earnings, EarningsChart, FinancialsChart, QuarterlyEarnings, QuarterlyFinancials,
    YearlyFinancials,
};
pub use earnings_history::{EarningsHistory, EarningsHistoryEntry};
pub use earnings_trend::{
    EarningsEstimate, EarningsTrend, EarningsTrendPeriod, EpsRevisions, EpsTrend, RevenueEstimate,
};
pub use equity_performance::{
    Benchmark, EquityPerformance, PerformanceOverview as EquityPerformanceOverview,
};
pub use financial_data::FinancialData;
pub use formatted_value::FormattedValue;
pub use fund_ownership::{FundOwner, FundOwnership};
pub use fund_performance::{
    AnnualReturn, AnnualTotalReturns, FundPerformance, PastQuarterlyReturns,
    PerformanceOverview as FundPerformanceOverview, PerformanceOverviewCat, RiskOverviewStatistics,
    RiskOverviewStatisticsCat, RiskStatistic, TrailingReturns, TrailingReturnsCat,
    TrailingReturnsNav,
};
pub use fund_profile::{FeesExpenses, FeesExpensesCat, FundProfile, ManagementInfo};
pub use income_statement_history::{IncomeStatementHistory, IncomeStatementHistoryQuarterly};
pub use index_trend::{IndexTrend, IndustryTrend, SectorTrend, TrendEstimate};
pub use insider_holders::{InsiderHolder, InsiderHolders};
pub use insider_transactions::{InsiderTransaction, InsiderTransactions};
pub use institution_ownership::{InstitutionOwner, InstitutionOwnership};
pub use major_holders_breakdown::MajorHoldersBreakdown;
pub use net_share_purchase_activity::NetSharePurchaseActivity;
pub use price::Price;
pub use quote_type::{QuoteTypeContainer, QuoteTypeData, QuoteTypeResponse, QuoteTypeResult};
pub use recommendation_trend::{RecommendationPeriod, RecommendationTrend};
pub use response::QuoteSummaryResponse;
pub use sec_filings::{SecExhibit, SecFiling, SecFilings};
pub use summary_detail::SummaryDetail;
pub use summary_profile::SummaryProfile;
pub use top_holdings::{BondRating, EquityHoldings, Holding, SectorWeighting, TopHoldings};
pub use upgrade_downgrade_history::{GradeChange, UpgradeDowngradeHistory};

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
