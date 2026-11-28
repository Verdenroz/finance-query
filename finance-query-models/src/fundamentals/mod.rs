//! Fundamental financial statement models.
//!
//! Contains data structures for Yahoo Finance's fundamentals-timeseries endpoint
//! and key financial metrics from quoteSummary.

// FinancialStatement (timeseries endpoint)
mod response;
pub use response::FinancialStatement;

// Short interest / short volume / share float (provider-routed)
mod short_activity;
pub use short_activity::{
    ShareFloat, ShareFloatFieldsInit, ShortInterest, ShortInterestFieldsInit, ShortVolume,
    ShortVolumeFieldsInit,
};

// Aggregated analyst consensus (provider-routed)
mod consensus;
pub use consensus::{
    PriceTargetConsensus, PriceTargetConsensusFieldsInit, PriceTargetSummary,
    PriceTargetSummaryFieldsInit, RatingConsensus, RatingConsensusFieldsInit,
};

// Trailing-twelve-month snapshots (provider-routed)
mod ttm;
pub use ttm::{
    FinancialRatiosTtm, FinancialRatiosTtmFieldsInit, KeyMetricsTtm, KeyMetricsTtmFieldsInit,
};

// ETF profile and holdings (provider-routed)
mod etf;
pub use etf::{
    EtfCountryWeighting, EtfCountryWeightingFieldsInit, EtfHolding, EtfHoldingFieldsInit,
    EtfProfile, EtfProfileFieldsInit, EtfSectorWeighting, EtfSectorWeightingFieldsInit,
};

// Earnings-surprise history (provider-routed)
mod earnings_surprise;
pub use earnings_surprise::{EarningsSurprise, EarningsSurpriseFieldsInit};

// Raw per-analyst grade-action history (provider-routed)
mod grading_history;
pub use grading_history::{GradingAction, GradingActionFieldsInit};

// Company identity/classification profile (provider-routed)
mod company_profile;
pub use company_profile::{CompanyProfile, CompanyProfileFieldsInit};

// quoteSummary modules (canonical home, re-exported from quote/ for backward compat)
pub mod balance_sheet_history;
pub mod cashflow_statement_history;
pub mod default_key_statistics;
pub mod financial_data;
pub mod income_statement_history;
pub mod summary_detail;

pub use balance_sheet_history::{BalanceSheetHistory, BalanceSheetHistoryQuarterly};
pub use cashflow_statement_history::{CashflowStatementHistory, CashflowStatementHistoryQuarterly};
pub use default_key_statistics::{DefaultKeyStatistics, DefaultKeyStatisticsFieldsInit};
pub use financial_data::FinancialData;
pub use income_statement_history::{IncomeStatementHistory, IncomeStatementHistoryQuarterly};
pub use summary_detail::{SummaryDetail, SummaryDetailFieldsInit};

#[cfg(all(test, feature = "dataframe"))]
mod dataframe_tests;
