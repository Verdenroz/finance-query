//! Company press-release model.
//!
//! Served through the [`Capability::CORPORATE`](crate::Capability::CORPORATE)
//! route; FMP is currently the only provider. Distinct from
//! [`News`](super::news::News) — these are the company's own releases, not
//! press coverage.

use serde::{Deserialize, Serialize};

/// A company press release.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PressRelease {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Publication date/time as reported by the provider.
    pub date: Option<String>,
    /// Release title.
    pub title: Option<String>,
    /// Full release text.
    pub text: Option<String>,
}
