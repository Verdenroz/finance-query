//! Stock market index data models.
//!
//! Canonical public types for index quotes and constituents,
//! shared across Polygon and FMP providers.

use serde::{Deserialize, Serialize};

/// A stock market index quote (e.g., S&P 500, NASDAQ, Dow Jones).
///
/// Obtain via [`Ticker::quote`](crate::Ticker::quote) using the index symbol (e.g., `"^GSPC"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexQuote {
    /// Index ticker symbol (e.g., `"^GSPC"`, `"^IXIC"`)
    pub symbol: String,
    /// Human-readable index name (e.g., `"S&P 500"`)
    pub name: Option<String>,
    /// Current index value
    pub price: Option<f64>,
    /// Price change
    pub change: Option<f64>,
    /// Price change percentage
    pub change_percent: Option<f64>,
    /// Unix timestamp of the last update
    pub timestamp: Option<i64>,
}

/// A major stock market index whose constituent lists providers can serve.
///
/// Passed to [`Index::constituents`](crate::Index::constituents) (derived from
/// the handle's symbol) and the INDICES provider route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MajorIndex {
    /// S&P 500 (`^GSPC` / `SPX`).
    Sp500,
    /// Nasdaq 100 (`^NDX` / `NDX`).
    Nasdaq100,
    /// Dow Jones Industrial Average (`^DJI` / `DJIA`).
    DowJones,
}

impl MajorIndex {
    /// Short lowercase identifier (`"sp500"`, `"nasdaq100"`, `"dowjones"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sp500 => "sp500",
            Self::Nasdaq100 => "nasdaq100",
            Self::DowJones => "dowjones",
        }
    }

    /// Map a common index symbol or name to the major index it denotes.
    ///
    /// Accepts the Yahoo caret form, the bare ticker, and common names,
    /// case-insensitively (e.g. `"^GSPC"`, `"SPX"`, `"sp500"`, `"^DJI"`,
    /// `"DJIA"`, `"^NDX"`, `"nasdaq 100"`). Returns `None` for anything else.
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        let s: String = symbol
            .trim()
            .trim_start_matches('^')
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '&' && *c != '-')
            .collect::<String>()
            .to_ascii_lowercase();
        match s.as_str() {
            "gspc" | "spx" | "sp500" => Some(Self::Sp500),
            "ndx" | "nasdaq100" => Some(Self::Nasdaq100),
            "dji" | "djia" | "dowjones" | "dow" | "dowjones30" => Some(Self::DowJones),
            _ => None,
        }
    }
}

impl std::fmt::Display for MajorIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A constituent (member) of a major stock market index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexConstituent {
    /// Ticker symbol of the constituent company
    pub symbol: String,
    /// Company name
    pub name: Option<String>,
    /// Sector classification
    pub sector: Option<String>,
    /// Sub-sector classification
    pub sub_sector: Option<String>,
    /// Headquarters location
    pub headquarters: Option<String>,
    /// Date the company was first added to the index (`YYYY-MM-DD`)
    pub date_first_added: Option<String>,
    /// SEC CIK number
    pub cik: Option<String>,
    /// Year the company was founded
    pub founded: Option<String>,
}

/// A historical change in a major index's constituency.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexConstituentChange {
    /// Date of the change (`YYYY-MM-DD`)
    pub date: Option<String>,
    /// Ticker symbol the change concerns
    pub symbol: Option<String>,
    /// Security that was added
    pub added_security: Option<String>,
    /// Ticker that was removed
    pub removed_ticker: Option<String>,
    /// Security that was removed
    pub removed_security: Option<String>,
    /// Reason for the change
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn major_index_maps_common_symbols() {
        for s in ["^GSPC", "GSPC", "SPX", "sp500", "S&P 500"] {
            assert_eq!(MajorIndex::from_symbol(s), Some(MajorIndex::Sp500), "{s}");
        }
        for s in ["^NDX", "ndx", "NASDAQ 100", "nasdaq100"] {
            assert_eq!(
                MajorIndex::from_symbol(s),
                Some(MajorIndex::Nasdaq100),
                "{s}"
            );
        }
        for s in ["^DJI", "DJIA", "dow", "Dow Jones"] {
            assert_eq!(
                MajorIndex::from_symbol(s),
                Some(MajorIndex::DowJones),
                "{s}"
            );
        }
        assert_eq!(MajorIndex::from_symbol("AAPL"), None);
        assert_eq!(MajorIndex::from_symbol("^IXIC"), None);
    }
}
