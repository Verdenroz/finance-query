//! Quote Summary Response
//!
//! Handles parsing of Yahoo Finance quoteSummary API responses

use crate::models::quote::*;

/// Response from the quoteSummary endpoint
///
/// Deserializes all requested modules once on construction to avoid repeated
/// JSON parsing on every accessor call. Uses `Option<T>` for each module since
/// Yahoo Finance may not return all modules for all symbols.
///
/// The return type of `QuoteProvider::fetch_quote`,
/// so an implementor populates the modules it can serve and leaves the rest
/// `None`. Each field mirrors one Yahoo `quoteSummary` module.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct QuoteSummaryResponse {
    /// The symbol this response is for
    pub symbol: String,

    /// Last price, change, and market state.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub price: Option<Price>,
    /// Day range, volume, yield, and valuation summary.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub summary_detail: Option<SummaryDetail>,
    /// Margins, returns, cash flow, and analyst targets.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub financial_data: Option<FinancialData>,
    /// Shares outstanding, float, beta, and per-share statistics.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub default_key_statistics: Option<DefaultKeyStatistics>,
    /// Company address, sector, industry, and officers.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub asset_profile: Option<AssetProfile>,
    /// Upcoming earnings and dividend dates.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub calendar_events: Option<CalendarEvents>,
    /// Quarterly and annual earnings history with estimates.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub earnings: Option<Earnings>,
    /// Analyst estimate trend by period.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub earnings_trend: Option<EarningsTrend>,
    /// Reported versus estimated EPS per quarter.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub earnings_history: Option<EarningsHistory>,
    /// Analyst buy, hold, and sell counts over time.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub recommendation_trend: Option<RecommendationTrend>,
    /// Insiders and their reported positions.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub insider_holders: Option<InsiderHolders>,
    /// Recent insider buys and sells.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub insider_transactions: Option<InsiderTransactions>,
    /// Institutional holders and position sizes.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub institution_ownership: Option<InstitutionOwnership>,
    /// Fund holders and position sizes.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub fund_ownership: Option<FundOwnership>,
    /// Insider and institutional ownership percentages.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub major_holders_breakdown: Option<MajorHoldersBreakdown>,
    /// Net insider share purchase totals.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub net_share_purchase_activity: Option<NetSharePurchaseActivity>,
    /// Instrument type, exchange, and naming metadata.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub quote_type: Option<QuoteTypeData>,
    /// Business description and contact details.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub summary_profile: Option<SummaryProfile>,
    /// Recent SEC filings with links.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub sec_filings: Option<SecFilings>,
    /// Analyst rating changes over time.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub upgrade_downgrade_history: Option<UpgradeDowngradeHistory>,
    /// Trailing and annual returns for a fund.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub fund_performance: Option<FundPerformance>,
    /// Fund category, family, and fee structure.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub fund_profile: Option<FundProfile>,
    /// A fund's largest positions and sector weights.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub top_holdings: Option<TopHoldings>,
    /// Index-level growth and valuation estimates.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub index_trend: Option<IndexTrend>,
    /// Industry-level growth and valuation estimates.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub industry_trend: Option<IndustryTrend>,
    /// Sector-level growth and valuation estimates.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub sector_trend: Option<SectorTrend>,
    /// Performance of the instrument against its peers.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub equity_performance: Option<EquityPerformance>,
}
