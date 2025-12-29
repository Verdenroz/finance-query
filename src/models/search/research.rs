//! Research Report Model
//!
//! Represents research reports from search results

use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// A collection of research reports with DataFrame support.
///
/// This wrapper allows `search_results.research_reports.to_dataframe()` syntax while still
/// acting like a `Vec<ResearchReport>` for iteration, indexing, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResearchReports(pub Vec<ResearchReport>);

impl Deref for ResearchReports {
    type Target = Vec<ResearchReport>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for ResearchReports {
    type Item = ResearchReport;
    type IntoIter = std::vec::IntoIter<ResearchReport>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ResearchReports {
    type Item = &'a ResearchReport;
    type IntoIter = std::slice::Iter<'a, ResearchReport>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(feature = "dataframe")]
impl ResearchReports {
    /// Converts the research reports to a polars DataFrame.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        ResearchReport::vec_to_dataframe(&self.0)
    }
}

/// A research report result from search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct ResearchReport {
    /// Report headline/title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_headline: Option<String>,
    /// Report author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Report publication date (Unix timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_date: Option<i64>,
    /// Report unique identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Provider name (e.g., "Morningstar", "Argus")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}
