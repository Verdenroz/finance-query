//! Quote module
//!
//! Contains the fully typed Quote struct for serialization and API responses.

use serde::{Deserialize, Serialize};

use super::{
    AssetProfile, BalanceSheetHistory, BalanceSheetHistoryQuarterly, CalendarEvents,
    CashflowStatementHistory, CashflowStatementHistoryQuarterly, DefaultKeyStatistics, Earnings,
    EarningsHistory, EarningsTrend, EquityPerformance, FinancialData, FundOwnership,
    FundPerformance, FundProfile, IncomeStatementHistory, IncomeStatementHistoryQuarterly,
    IndexTrend, IndustryTrend, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, NetSharePurchaseActivity, Price, QuoteSummaryResponse, QuoteTypeData,
    RecommendationTrend, SecFilings, SectorTrend, SummaryDetail, SummaryProfile, TopHoldings,
    UpgradeDowngradeHistory,
};

/// Fully typed quote data
///
/// Aggregates all typed modules from the quoteSummary endpoint into a single
/// convenient structure. All fields are optional since Yahoo Finance may not
/// return all modules for every symbol.
///
/// This is the recommended type for serialization and API responses.
/// Used for both single quote and batch quotes endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Stock symbol
    pub symbol: String,

    /// Company logo URL (50x50px)
    ///
    /// Fetched from /v7/finance/quote endpoint when requested via logo=true parameter.
    /// Returns None if logo not available or fetch fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,

    /// Alternative company logo URL (50x50px)
    ///
    /// Fetched from /v7/finance/quote endpoint when requested via logo=true parameter.
    /// Returns None if logo not available or fetch fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_logo_url: Option<String>,

    /// Current price and trading data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,

    /// Quote type metadata (exchange, company name, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_type: Option<QuoteTypeData>,

    /// Summary detail (trading metrics, market cap, volume)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_detail: Option<SummaryDetail>,

    /// Financial data (margins, cash flow, analyst recommendations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_data: Option<FinancialData>,

    /// Key statistics (valuation metrics, shares, ratios)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_statistics: Option<DefaultKeyStatistics>,

    /// Company profile (address, sector, industry, business description)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_profile: Option<SummaryProfile>,

    /// Earnings data (quarterly earnings vs estimates, revenue/earnings history)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings: Option<Earnings>,

    /// Equity performance (returns vs benchmark across multiple time periods)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equity_performance: Option<EquityPerformance>,

    /// Calendar events (upcoming earnings dates, dividend dates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calendar_events: Option<CalendarEvents>,

    /// Analyst recommendation trends over time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_trend: Option<RecommendationTrend>,

    /// Analyst upgrades/downgrades history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_downgrade_history: Option<UpgradeDowngradeHistory>,

    /// Historical earnings data (actual vs estimate)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_history: Option<EarningsHistory>,

    /// Earnings trend data (estimates and revisions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_trend: Option<EarningsTrend>,

    /// Asset profile (company details, officers, website, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_profile: Option<AssetProfile>,

    /// Insider stock holdings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insider_holders: Option<InsiderHolders>,

    /// Insider transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insider_transactions: Option<InsiderTransactions>,

    /// Top institutional owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution_ownership: Option<InstitutionOwnership>,

    /// Top fund owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_ownership: Option<FundOwnership>,

    /// Major holders breakdown (insiders, institutions, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major_holders_breakdown: Option<MajorHoldersBreakdown>,

    /// Net share purchase activity by insiders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_share_purchase_activity: Option<NetSharePurchaseActivity>,

    /// SEC filings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_filings: Option<SecFilings>,

    /// Balance sheet history (annual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_sheet_history: Option<BalanceSheetHistory>,

    /// Balance sheet history (quarterly)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_sheet_history_quarterly: Option<BalanceSheetHistoryQuarterly>,

    /// Cash flow statement history (annual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cashflow_statement_history: Option<CashflowStatementHistory>,

    /// Cash flow statement history (quarterly)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cashflow_statement_history_quarterly: Option<CashflowStatementHistoryQuarterly>,

    /// Income statement history (annual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub income_statement_history: Option<IncomeStatementHistory>,

    /// Income statement history (quarterly)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub income_statement_history_quarterly: Option<IncomeStatementHistoryQuarterly>,

    /// Index trend (PE and PEG ratios)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_trend: Option<IndexTrend>,

    /// Industry trend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_trend: Option<IndustryTrend>,

    /// Sector trend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_trend: Option<SectorTrend>,

    /// Fund profile (for ETFs and mutual funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_profile: Option<FundProfile>,

    /// Fund performance data (for ETFs and mutual funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_performance: Option<FundPerformance>,

    /// Top holdings and sector weightings (for ETFs and mutual funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_holdings: Option<TopHoldings>,
}

impl Quote {
    /// Creates a Quote from a QuoteSummaryResponse
    ///
    /// Extracts and deserializes all typed modules from the raw response.
    ///
    /// # Arguments
    ///
    /// * `response` - The quote summary response from Yahoo Finance
    /// * `logo_url` - Optional company logo URL (fetched separately from /v7/finance/quote)
    /// * `company_logo_url` - Optional alternative company logo URL (fetched separately from /v7/finance/quote)
    pub fn from_response(
        response: &QuoteSummaryResponse,
        logo_url: Option<String>,
        company_logo_url: Option<String>,
    ) -> Self {
        Self {
            symbol: response.symbol.clone(),
            logo_url,
            company_logo_url,
            price: response.get_typed("price").ok(),
            quote_type: response.get_typed("quoteType").ok(),
            summary_detail: response.get_typed("summaryDetail").ok(),
            financial_data: response.get_typed("financialData").ok(),
            key_statistics: response.get_typed("defaultKeyStatistics").ok(),
            summary_profile: response.get_typed("summaryProfile").ok(),
            earnings: response.get_typed("earnings").ok(),
            equity_performance: response.get_typed("equityPerformance").ok(),
            calendar_events: response.get_typed("calendarEvents").ok(),
            recommendation_trend: response.get_typed("recommendationTrend").ok(),
            upgrade_downgrade_history: response.get_typed("upgradeDowngradeHistory").ok(),
            earnings_history: response.get_typed("earningsHistory").ok(),
            earnings_trend: response.get_typed("earningsTrend").ok(),
            asset_profile: response.get_typed("assetProfile").ok(),
            insider_holders: response.get_typed("insiderHolders").ok(),
            insider_transactions: response.get_typed("insiderTransactions").ok(),
            institution_ownership: response.get_typed("institutionOwnership").ok(),
            fund_ownership: response.get_typed("fundOwnership").ok(),
            major_holders_breakdown: response.get_typed("majorHoldersBreakdown").ok(),
            net_share_purchase_activity: response.get_typed("netSharePurchaseActivity").ok(),
            sec_filings: response.get_typed("secFilings").ok(),
            balance_sheet_history: response.get_typed("balanceSheetHistory").ok(),
            balance_sheet_history_quarterly: response
                .get_typed("balanceSheetHistoryQuarterly")
                .ok(),
            cashflow_statement_history: response.get_typed("cashflowStatementHistory").ok(),
            cashflow_statement_history_quarterly: response
                .get_typed("cashflowStatementHistoryQuarterly")
                .ok(),
            income_statement_history: response.get_typed("incomeStatementHistory").ok(),
            income_statement_history_quarterly: response
                .get_typed("incomeStatementHistoryQuarterly")
                .ok(),
            index_trend: response.get_typed("indexTrend").ok(),
            industry_trend: response.get_typed("industryTrend").ok(),
            sector_trend: response.get_typed("sectorTrend").ok(),
            fund_profile: response.get_typed("fundProfile").ok(),
            fund_performance: response.get_typed("fundPerformance").ok(),
            top_holdings: response.get_typed("topHoldings").ok(),
        }
    }
}
