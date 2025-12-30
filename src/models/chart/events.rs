//! Chart events module
//!
//! Contains dividend, split, and capital gain data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Chart events containing dividends, splits, and capital gains
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChartEvents {
    /// Dividend events keyed by timestamp
    #[serde(default)]
    pub dividends: HashMap<String, DividendEvent>,
    /// Stock split events keyed by timestamp
    #[serde(default)]
    pub splits: HashMap<String, SplitEvent>,
    /// Capital gain events keyed by timestamp
    #[serde(default)]
    pub capital_gains: HashMap<String, CapitalGainEvent>,
}

/// Raw dividend event from Yahoo Finance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DividendEvent {
    /// Dividend amount per share
    pub amount: f64,
    /// Timestamp of the dividend
    pub date: i64,
}

/// Raw split event from Yahoo Finance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SplitEvent {
    /// Timestamp of the split
    pub date: i64,
    /// Numerator of the split ratio
    pub numerator: f64,
    /// Denominator of the split ratio
    pub denominator: f64,
    /// Split ratio as string (e.g., "2:1", "10:1")
    pub split_ratio: String,
}

/// Raw capital gain event from Yahoo Finance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CapitalGainEvent {
    /// Capital gain amount per share
    pub amount: f64,
    /// Timestamp of the capital gain distribution
    pub date: i64,
}

/// Public dividend data
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::dividends()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
pub struct Dividend {
    /// Timestamp (Unix)
    pub timestamp: i64,
    /// Dividend amount per share
    pub amount: f64,
}

/// Public stock split data
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::splits()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
pub struct Split {
    /// Timestamp (Unix)
    pub timestamp: i64,
    /// Numerator of the split ratio
    pub numerator: f64,
    /// Denominator of the split ratio
    pub denominator: f64,
    /// Split ratio as string (e.g., "2:1", "10:1")
    pub ratio: String,
}

/// Public capital gain data
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::capital_gains()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
pub struct CapitalGain {
    /// Timestamp (Unix)
    pub timestamp: i64,
    /// Capital gain amount per share
    pub amount: f64,
}

impl ChartEvents {
    /// Convert to sorted list of dividends
    pub(crate) fn to_dividends(&self) -> Vec<Dividend> {
        let mut dividends: Vec<Dividend> = self
            .dividends
            .values()
            .map(|d| Dividend {
                timestamp: d.date,
                amount: d.amount,
            })
            .collect();
        dividends.sort_by_key(|d| d.timestamp);
        dividends
    }

    /// Convert to sorted list of splits
    pub(crate) fn to_splits(&self) -> Vec<Split> {
        let mut splits: Vec<Split> = self
            .splits
            .values()
            .map(|s| Split {
                timestamp: s.date,
                numerator: s.numerator,
                denominator: s.denominator,
                ratio: s.split_ratio.clone(),
            })
            .collect();
        splits.sort_by_key(|s| s.timestamp);
        splits
    }

    /// Convert to sorted list of capital gains
    pub(crate) fn to_capital_gains(&self) -> Vec<CapitalGain> {
        let mut gains: Vec<CapitalGain> = self
            .capital_gains
            .values()
            .map(|g| CapitalGain {
                timestamp: g.date,
                amount: g.amount,
            })
            .collect();
        gains.sort_by_key(|g| g.timestamp);
        gains
    }
}
