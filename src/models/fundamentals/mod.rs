//! Fundamental financial statement models.
//!
//! Contains data structures for Yahoo Finance's fundamentals-timeseries endpoint
//! and key financial metrics from quoteSummary.

// FinancialStatement (timeseries endpoint)
mod response;
pub use response::FinancialStatement;

// Short interest / short volume / share float (provider-routed)
mod short_activity;
pub use short_activity::{ShareFloat, ShortInterest, ShortVolume};

// Aggregated analyst consensus (provider-routed)
mod consensus;
pub use consensus::{PriceTargetConsensus, PriceTargetSummary, RatingConsensus};

// Trailing-twelve-month snapshots (provider-routed)
mod ttm;
pub use ttm::{FinancialRatiosTtm, KeyMetricsTtm};

// ETF profile and holdings (provider-routed)
mod etf;
pub use etf::{EtfCountryWeighting, EtfHolding, EtfProfile, EtfSectorWeighting};

// Earnings-surprise history (provider-routed)
mod earnings_surprise;
pub use earnings_surprise::EarningsSurprise;

// Raw per-analyst grade-action history (provider-routed)
mod grading_history;
pub use grading_history::GradingAction;

// Company identity/classification profile (provider-routed)
mod company_profile;
pub use company_profile::CompanyProfile;

// quoteSummary modules (canonical home, re-exported from quote/ for backward compat)
pub(crate) mod balance_sheet_history;
pub(crate) mod cashflow_statement_history;
pub(crate) mod default_key_statistics;
pub(crate) mod financial_data;
pub(crate) mod income_statement_history;
pub(crate) mod summary_detail;

pub(crate) use balance_sheet_history::{BalanceSheetHistory, BalanceSheetHistoryQuarterly};
pub(crate) use cashflow_statement_history::{
    CashflowStatementHistory, CashflowStatementHistoryQuarterly,
};
pub use default_key_statistics::DefaultKeyStatistics;
pub use financial_data::FinancialData;
pub(crate) use income_statement_history::{
    IncomeStatementHistory, IncomeStatementHistoryQuarterly,
};
pub use summary_detail::SummaryDetail;
