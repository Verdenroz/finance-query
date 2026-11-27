//! Raw per-analyst grade-action history.
//!
//! Served through the [`Capability::FUNDAMENTALS`](finance_query_types::Capability::FUNDAMENTALS)
//! route; FMP and Yahoo both implement it. Distinct from the aggregated
//! [`RatingConsensus`](super::RatingConsensus) rollup — this is the
//! individual upgrade/downgrade events behind that rollup.

use serde::{Deserialize, Serialize};

/// A single analyst upgrade/downgrade/initiation action.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct GradingAction {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Action date (`YYYY-MM-DD`).
    #[cfg_attr(feature = "dataframe", dataframe(date))]
    pub date: Option<String>,
    /// The analyst firm issuing the grade.
    pub grading_company: Option<String>,
    /// Prior grade, if this is a change rather than an initiation.
    pub previous_grade: Option<String>,
    /// New grade.
    pub new_grade: Option<String>,
}
