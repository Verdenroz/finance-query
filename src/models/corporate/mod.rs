//! Corporate data models.
//!
//! Company profiles, officers, ownership, insider activity, and related data.

// Sub-capability directories
/// News article models.
pub mod news;
/// Recommendation/similar symbol models.
pub mod recommendation;
/// Earnings call transcripts.
pub mod transcript;

// quoteSummary modules (canonical home, re-exported from quote/ for backward compat)
pub(crate) mod asset_profile;
pub(crate) mod calendar_events;
pub(crate) mod earnings;
pub(crate) mod earnings_history;
pub(crate) mod earnings_trend;
pub(crate) mod equity_performance;
pub(crate) mod fund_ownership;
pub(crate) mod fund_performance;
pub(crate) mod fund_profile;
pub(crate) mod insider_holders;
pub(crate) mod insider_transactions;
pub(crate) mod institution_ownership;
pub(crate) mod major_holders_breakdown;
pub(crate) mod net_share_purchase_activity;
pub(crate) mod recommendation_trend;
pub(crate) mod sec_filings;
pub(crate) mod summary_profile;
pub(crate) mod top_holdings;
pub(crate) mod upgrade_downgrade_history;

pub(crate) use asset_profile::{AssetProfile, CompanyOfficer};
pub(crate) use calendar_events::CalendarEvents;
pub(crate) use earnings::Earnings;
pub(crate) use earnings_history::EarningsHistory;
pub(crate) use earnings_trend::EarningsTrend;
pub(crate) use equity_performance::EquityPerformance;
pub(crate) use fund_ownership::FundOwnership;
pub(crate) use fund_performance::FundPerformance;
pub(crate) use fund_profile::FundProfile;
pub(crate) use insider_holders::InsiderHolders;
pub(crate) use insider_transactions::InsiderTransactions;
pub(crate) use institution_ownership::InstitutionOwnership;
pub(crate) use major_holders_breakdown::MajorHoldersBreakdown;
pub(crate) use net_share_purchase_activity::NetSharePurchaseActivity;
pub(crate) use recommendation_trend::RecommendationTrend;
pub(crate) use sec_filings::SecFilings;
pub(crate) use summary_profile::SummaryProfile;
pub(crate) use top_holdings::TopHoldings;
pub(crate) use upgrade_downgrade_history::UpgradeDowngradeHistory;
