//! Financial statement models.
//!
//! Contains data structures for Yahoo Finance's fundamentals-timeseries endpoint.
//! Provides historical financial statements (income, balance sheet, cash flow).

mod response;

pub use response::FinancialStatement;
