//! Chart events module
//!
//! Contains dividend, split, and capital gain data structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Chart events containing dividends, splits, and capital gains
///
/// Events are deserialized from HashMaps, then lazily converted to sorted vectors
/// on first access and cached for subsequent calls.
#[derive(Debug, Default, Serialize, Deserialize)]
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

    /// Cached sorted dividend vector (computed once on first access)
    #[serde(skip)]
    dividends_cache: OnceLock<Vec<Dividend>>,
    /// Cached sorted splits vector (computed once on first access)
    #[serde(skip)]
    splits_cache: OnceLock<Vec<Split>>,
    /// Cached sorted capital gains vector (computed once on first access)
    #[serde(skip)]
    capital_gains_cache: OnceLock<Vec<CapitalGain>>,
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

impl Clone for ChartEvents {
    fn clone(&self) -> Self {
        // Helper to clone OnceLock if initialized
        fn clone_cache<T: Clone>(cache: &OnceLock<T>) -> OnceLock<T> {
            let new_cache = OnceLock::new();
            if let Some(value) = cache.get() {
                let _ = new_cache.set(value.clone());
            }
            new_cache
        }

        Self {
            dividends: self.dividends.clone(),
            splits: self.splits.clone(),
            capital_gains: self.capital_gains.clone(),
            dividends_cache: clone_cache(&self.dividends_cache),
            splits_cache: clone_cache(&self.splits_cache),
            capital_gains_cache: clone_cache(&self.capital_gains_cache),
        }
    }
}

impl ChartEvents {
    /// Get sorted list of dividends (cached after first call)
    pub(crate) fn to_dividends(&self) -> Vec<Dividend> {
        self.dividends_cache
            .get_or_init(|| {
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
            })
            .clone()
    }

    /// Get sorted list of splits (cached after first call)
    pub(crate) fn to_splits(&self) -> Vec<Split> {
        self.splits_cache
            .get_or_init(|| {
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
            })
            .clone()
    }

    /// Get sorted list of capital gains (cached after first call)
    pub(crate) fn to_capital_gains(&self) -> Vec<CapitalGain> {
        self.capital_gains_cache
            .get_or_init(|| {
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
            })
            .clone()
    }
}
