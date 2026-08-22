//! Raw per-analyst grade-action history.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; FMP is currently the only provider. Distinct from the aggregated
//! [`RatingConsensus`](super::RatingConsensus) rollup — this is the
//! individual upgrade/downgrade events behind that rollup.

use serde::{Deserialize, Serialize};

/// A single analyst upgrade/downgrade/initiation action.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GradingAction {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Action date (`YYYY-MM-DD`).
    pub date: Option<String>,
    /// The analyst firm issuing the grade.
    pub grading_company: Option<String>,
    /// Prior grade, if this is a change rather than an initiation.
    pub previous_grade: Option<String>,
    /// New grade.
    pub new_grade: Option<String>,
}
