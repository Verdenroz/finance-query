//! Lossless stock-bar downloads, separate from chart presentation.

use crate::{FinanceError, Interval, Result, SortType};
use serde::{Deserialize, Serialize};

/// Price adjustment policy. There is no implicit provider default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PriceAdjustment {
    /// Preserve historical, unadjusted prices.
    Unadjusted,
    /// Ask the provider to adjust prices for splits.
    SplitAdjusted,
}

/// A bounded minute or daily bar request.
#[derive(Debug, Clone, Serialize)]
pub struct StockBarsRequest {
    pub(crate) symbol: String,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) timespan: String,
    pub(crate) adjustment: PriceAdjustment,
    pub(crate) sort: SortType,
    pub(crate) limit: u32,
}

impl StockBarsRequest {
    /// Set inclusive date bounds and the adjustment policy. Currently supports
    /// OneMinute and OneDay. Bars are oldest first unless [`sort`](Self::sort) says otherwise.
    pub fn new(
        symbol: &str,
        from: &str,
        to: &str,
        interval: Interval,
        adjustment: PriceAdjustment,
    ) -> Result<Self> {
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
            adjustment,
            sort: SortType::Asc,
            limit: 50_000,
        })
    }

    /// Select oldest-first ([`SortType::Asc`]) or newest-first order.
    pub fn sort(mut self, order: SortType) -> Self {
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

    #[cfg(any(feature = "polygon", feature = "fmp"))]
    pub(crate) fn split_adjusted(&self) -> bool {
        self.adjustment == PriceAdjustment::SplitAdjusted
    }

    pub(crate) fn sort_param(&self) -> &'static str {
        match self.sort {
            SortType::Asc => "asc",
            SortType::Desc => "desc",
        }
    }

    /// The fields a continuation must match. Changing them requires a new cursor version.
    pub(crate) fn cursor_identity(&self) -> String {
        let adjustment = match self.adjustment {
            PriceAdjustment::Unadjusted => "unadjusted",
            PriceAdjustment::SplitAdjusted => "split",
        };
        serde_json::json!([
            self.symbol,
            self.from,
            self.to,
            self.timespan,
            adjustment,
            self.sort_param(),
            self.limit
        ])
        .to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_identity_is_pinned_to_the_query_fields() {
        let request = StockBarsRequest::new(
            "AAPL",
            "2020-01-02",
            "2020-01-03",
            Interval::OneDay,
            PriceAdjustment::Unadjusted,
        )
        .unwrap();
        assert_eq!(
            request.cursor_identity(),
            r#"["AAPL","2020-01-02","2020-01-03","day","unadjusted","asc",50000]"#
        );
        let changed = request.sort(SortType::Desc).page_size(10).unwrap();
        assert_eq!(
            changed.cursor_identity(),
            r#"["AAPL","2020-01-02","2020-01-03","day","unadjusted","desc",10]"#
        );
    }
}
