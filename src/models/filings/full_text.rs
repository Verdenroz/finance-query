//! Provider-neutral full-text filing search models.
//!
//! Served through the [`Capability::FILINGS`](crate::Capability::FILINGS)
//! route; EDGAR is currently the only provider. Distinct from
//! [`EdgarSearchResults`](super::EdgarSearchResults), which mirrors EDGAR's raw
//! Elasticsearch envelope — this is the flattened shape the routed API returns.

use serde::{Deserialize, Serialize};

/// Which parts of a full-text search to constrain.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct FilingSearchFilters {
    /// Restrict to these form types (e.g. `["10-K", "8-K"]`).
    pub forms: Option<Vec<String>>,
    /// Earliest filing date, `YYYY-MM-DD`.
    pub start_date: Option<String>,
    /// Latest filing date, `YYYY-MM-DD`.
    pub end_date: Option<String>,
    /// Restrict to a single filer's CIK (zero-padded or not).
    pub cik: Option<String>,
    /// Maximum hits to return. Providers cap this; EDGAR's ceiling is 100.
    pub limit: Option<u32>,
}

impl FilingSearchFilters {
    /// Restrict the search to these form types.
    pub fn forms<S: Into<String>, I: IntoIterator<Item = S>>(mut self, forms: I) -> Self {
        self.forms = Some(forms.into_iter().map(Into::into).collect());
        self
    }

    /// Restrict the search to filings on or after `date` (`YYYY-MM-DD`).
    pub fn from(mut self, date: impl Into<String>) -> Self {
        self.start_date = Some(date.into());
        self
    }

    /// Restrict the search to filings on or before `date` (`YYYY-MM-DD`).
    pub fn to(mut self, date: impl Into<String>) -> Self {
        self.end_date = Some(date.into());
        self
    }

    /// Restrict the search to one filer.
    pub fn cik(mut self, cik: impl Into<String>) -> Self {
        self.cik = Some(cik.into());
        self
    }

    /// Cap the number of hits returned.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// One filing matching a full-text search.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingSearchHit {
    /// Accession number of the filing.
    pub accession_number: Option<String>,
    /// Form type (e.g. `"10-K"`).
    pub form: Option<String>,
    /// Filing date (`YYYY-MM-DD`).
    pub filed_date: Option<String>,
    /// Period the filing covers (`YYYY-MM-DD`).
    pub period_ending: Option<String>,
    /// Filer display names, as the provider spells them.
    pub company_names: Vec<String>,
    /// Filer CIKs.
    pub ciks: Vec<String>,
    /// Relevance score assigned by the provider's search index.
    pub score: Option<f64>,
    /// Direct URL to the matching document, when it can be derived.
    pub url: Option<String>,
}
