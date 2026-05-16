//! CIK (Central Index Key) lookup models.

use serde::{Deserialize, Serialize};

/// An entry from the SEC ticker-to-CIK mapping.
///
/// Maps a stock ticker symbol to its SEC CIK number and company name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CikEntry {
    /// CIK number (unique SEC identifier)
    pub cik: u64,
    /// Ticker symbol
    pub ticker: String,
    /// Company name
    pub title: String,
}
