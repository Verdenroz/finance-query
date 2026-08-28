//! Corporate data models.
//!
//! Company profiles, officers, ownership, insider activity, and related data.

// Sub-capability directories
/// Executive compensation and employee headcount.
pub mod governance;
/// News article models.
pub mod news;

/// Provider-neutral earnings call transcript.
pub mod earnings_transcript;
pub mod press_release;
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

pub use asset_profile::{AssetProfile, CompanyOfficer};
pub use calendar_events::CalendarEvents;
pub use earnings::Earnings;
pub use earnings_history::EarningsHistory;
pub use earnings_trend::EarningsTrend;
pub use equity_performance::EquityPerformance;
pub use fund_ownership::FundOwnership;
pub use fund_performance::FundPerformance;
pub use fund_profile::FundProfile;
pub use insider_holders::InsiderHolders;
pub use insider_transactions::InsiderTransactions;
pub use institution_ownership::InstitutionOwnership;
pub use major_holders_breakdown::MajorHoldersBreakdown;
pub use net_share_purchase_activity::NetSharePurchaseActivity;
pub use recommendation_trend::RecommendationTrend;
pub use sec_filings::SecFilings;
pub use summary_profile::SummaryProfile;
pub use top_holdings::TopHoldings;
pub use upgrade_downgrade_history::UpgradeDowngradeHistory;
