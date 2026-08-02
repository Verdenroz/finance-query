//! Sectioned filing text and risk-factor models.
//!
//! Served through the [`Capability::FILINGS`](crate::Capability::FILINGS)
//! route; Polygon is currently the only provider (EDGAR serves filing
//! metadata, not sectioned text).

use serde::{Deserialize, Serialize};

/// Which filing form to fetch sectioned text for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FilingSectionForm {
    /// Annual report (10-K) sections.
    TenK,
    /// Current report (8-K) text.
    EightK,
}

/// One section of an SEC filing's text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingSection {
    /// Section key/name (e.g. `"risk_factors"`, `"mdna"`).
    pub section: Option<String>,
    /// Section text content.
    pub content: Option<String>,
}

/// A risk factor extracted from SEC filings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RiskFactor {
    /// Risk factor title.
    pub title: Option<String>,
    /// Risk factor text.
    pub text: Option<String>,
    /// Risk category.
    pub category: Option<String>,
    /// Date of the filing the factor was extracted from (`YYYY-MM-DD`).
    pub filing_date: Option<String>,
}
