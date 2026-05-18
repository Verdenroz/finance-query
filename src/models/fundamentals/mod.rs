//! Fundamental financial statement models.
//!
//! Contains data structures for Yahoo Finance's fundamentals-timeseries endpoint
//! and key financial metrics from quoteSummary.

// FinancialStatement (timeseries endpoint)
mod response;
pub use response::FinancialStatement;
#[cfg(feature = "python")]
pub use response::PyFinancialStatement;

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
pub(crate) use default_key_statistics::DefaultKeyStatistics;
pub(crate) use financial_data::FinancialData;
pub(crate) use income_statement_history::{
    IncomeStatementHistory, IncomeStatementHistoryQuarterly,
};
pub(crate) use summary_detail::SummaryDetail;
