//! Data models for Yahoo Finance responses.
//!
//! This module contains all the data structures returned by the library's API methods.
//! Types are organized by category (chart, quote, options, etc.).

/// Chart/historical data models.
pub mod chart;
/// Financials (fundamentals-timeseries) models.
pub mod financials;
/// Market hours models.
pub mod hours;
/// Technical indicators models.
pub mod indicators;
/// Industry models for market industry data.
pub mod industries;
/// News models.
pub mod news;
/// Options models.
pub mod options;
/// Quote models for detailed stock information.
pub mod quote;
/// Recommendation models.
pub mod recommendation;
/// Screener models for predefined Yahoo Finance screeners.
pub mod screeners;
/// Search models.
pub mod search;
/// Sector models for market sector data.
pub mod sectors;
