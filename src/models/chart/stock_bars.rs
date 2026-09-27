//! Lossless stock-bar downloads, separate from chart presentation.

use crate::{FinanceError, Interval, Result};
use serde::{Deserialize, Serialize};

/// Price adjustment policy. There is no implicit provider default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceAdjustment {
    /// Preserve historical, unadjusted prices.
    Unadjusted,
    /// Ask the provider to adjust prices for splits.
    SplitAdjusted,
}

/// Order of bars within a download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    /// Oldest first.
    Ascending,
    /// Newest first.
    Descending,
}

/// A bounded minute or daily bar request.
#[derive(Debug, Clone, Serialize)]
pub struct StockBarsRequest {
    pub(crate) symbol: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) timespan: String,
    pub(crate) adjustment: Option<PriceAdjustment>,
    pub(crate) sort: SortOrder,
    pub(crate) limit: u32,
}

impl StockBarsRequest {
    /// Set inclusive date bounds. Currently supports OneMinute and OneDay.
    pub fn new(symbol: &str, from: &str, to: &str, interval: Interval) -> Result<Self> {
        crate::models::discovery::listings::date(from)?;
        crate::models::discovery::listings::date(to)?;
        if symbol.trim().is_empty() || symbol == "." || symbol == ".." || from > to {
            return Err(FinanceError::InvalidParameter {
                param: "stock_bars".into(),
                reason: "symbol and ordered date bounds are required".into(),
            });
        }
        let timespan = match interval {
            Interval::OneMinute => "minute",
            Interval::OneDay => "day",
            _ => {
                return Err(FinanceError::InvalidParameter {
                    param: "interval".into(),
                    reason: "stock pages support OneMinute and OneDay".into(),
                });
            }
        };
        Ok(Self {
            symbol: symbol.into(),
            from: from.into(),
            to: to.into(),
            timespan: timespan.into(),
            adjustment: None,
            sort: SortOrder::Ascending,
            limit: 50_000,
        })
    }

    /// Select the adjustment policy. Required before execution.
    pub fn adjustment(mut self, policy: PriceAdjustment) -> Self {
        self.adjustment = Some(policy);
        self
    }
    /// Select oldest-first or newest-first order.
    pub fn sort(mut self, order: SortOrder) -> Self {
        self.sort = order;
        self
    }
    /// Set the provider page size in 1–50000.
    pub fn page_size(mut self, limit: u32) -> Result<Self> {
        if !(1..=50_000).contains(&limit) {
            return Err(FinanceError::InvalidParameter {
                param: "page_size".into(),
                reason: "expected 1..=50000".into(),
            });
        }
        self.limit = limit;
        Ok(self)
    }

    pub(crate) fn adjusted(&self) -> Result<bool> {
        self.adjustment
            .map(|p| p == PriceAdjustment::SplitAdjusted)
            .ok_or_else(|| FinanceError::InvalidParameter {
                param: "adjustment".into(),
                reason: "choose an explicit price adjustment policy".into(),
            })
    }
}

/// A provider price bar with exact timestamp units and fractional volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockBar {
    /// UTC Unix milliseconds, without rounding or timezone conversion.
    pub timestamp_ms: i64,
    /// Opening price.
    pub open: f64,
    /// Highest price.
    pub high: f64,
    /// Lowest price.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Volume, preserving fractional provider values.
    pub volume: f64,
    /// Number of transactions, absent when not supplied.
    pub transactions: Option<u64>,
    /// Volume-weighted average price, when supplied.
    pub vwap: Option<f64>,
}
