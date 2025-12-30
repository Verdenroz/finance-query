//! Quote models.
//!
//! Contains all data structures and enums for Yahoo Finance's quoteSummary endpoint.

// Internal modules - used for from_response() parsing.
pub(crate) mod asset_profile;
pub(crate) mod balance_sheet_history;
pub(crate) mod calendar_events;
pub(crate) mod cashflow_statement_history;
pub(crate) mod default_key_statistics;
pub(crate) mod earnings;
pub(crate) mod earnings_history;
pub(crate) mod earnings_trend;
pub(crate) mod equity_performance;
pub(crate) mod financial_data;
pub(crate) mod fund_ownership;
pub(crate) mod fund_performance;
pub(crate) mod fund_profile;
pub(crate) mod income_statement_history;
pub(crate) mod index_trend;
pub(crate) mod insider_holders;
pub(crate) mod insider_transactions;
pub(crate) mod institution_ownership;
pub(crate) mod major_holders_breakdown;
pub(crate) mod net_share_purchase_activity;
pub(crate) mod price;
pub(crate) mod quote_type;
pub(crate) mod recommendation_trend;
pub(crate) mod response;
pub(crate) mod sec_filings;
pub(crate) mod summary_detail;
pub(crate) mod summary_profile;
pub(crate) mod top_holdings;
pub(crate) mod upgrade_downgrade_history;

// Public modules
pub mod data;
/// Formatted value wrapper for Yahoo Finance numeric fields.
pub mod formatted_value;

// Re-export only the final flattened Quote struct and FormattedValue (used in Quote's public fields)
pub use data::Quote;
pub use formatted_value::FormattedValue;

// Internal re-exports for crate use
pub(crate) use asset_profile::{AssetProfile, CompanyOfficer};
pub(crate) use balance_sheet_history::{BalanceSheetHistory, BalanceSheetHistoryQuarterly};
pub(crate) use calendar_events::CalendarEvents;
pub(crate) use cashflow_statement_history::{
    CashflowStatementHistory, CashflowStatementHistoryQuarterly,
};
pub(crate) use default_key_statistics::DefaultKeyStatistics;
pub(crate) use earnings::Earnings;
pub(crate) use earnings_history::EarningsHistory;
pub(crate) use earnings_trend::EarningsTrend;
pub(crate) use equity_performance::EquityPerformance;
pub(crate) use financial_data::FinancialData;
pub(crate) use fund_ownership::FundOwnership;
pub(crate) use fund_performance::FundPerformance;
pub(crate) use fund_profile::FundProfile;
pub(crate) use income_statement_history::{
    IncomeStatementHistory, IncomeStatementHistoryQuarterly,
};
pub(crate) use index_trend::{IndexTrend, IndustryTrend, SectorTrend};
pub(crate) use insider_holders::InsiderHolders;
pub(crate) use insider_transactions::InsiderTransactions;
pub(crate) use institution_ownership::InstitutionOwnership;
pub(crate) use major_holders_breakdown::MajorHoldersBreakdown;
pub(crate) use net_share_purchase_activity::NetSharePurchaseActivity;
pub(crate) use price::Price;
pub(crate) use quote_type::QuoteTypeData;
pub(crate) use recommendation_trend::RecommendationTrend;
pub(crate) use response::QuoteSummaryResponse;
pub(crate) use sec_filings::SecFilings;
pub(crate) use summary_detail::SummaryDetail;
pub(crate) use summary_profile::SummaryProfile;
pub(crate) use top_holdings::TopHoldings;
pub(crate) use upgrade_downgrade_history::UpgradeDowngradeHistory;

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
