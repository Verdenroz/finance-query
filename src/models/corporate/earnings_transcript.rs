//! Earnings-call transcript, provider-neutral shape.
//!
//! Served through the [`Capability::CORPORATE`](crate::Capability::CORPORATE)
//! route. Distinct from the richer, Yahoo-only [`Transcript`](crate::Transcript)
//! returned by [`finance::earnings_transcript`](crate::finance::earnings_transcript) —
//! that one stays a Yahoo-pinned shortcut with its own model; this one is the
//! flat, cross-provider shape every routed provider maps into.

use serde::{Deserialize, Serialize};

/// One earnings call transcript.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsTranscript {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Fiscal quarter (e.g. `"Q4"`).
    pub quarter: Option<String>,
    /// Fiscal year.
    pub year: Option<i32>,
    /// Call date (`YYYY-MM-DD`), when the provider reports one.
    pub date: Option<String>,
    /// Full transcript text.
    pub text: String,
}
