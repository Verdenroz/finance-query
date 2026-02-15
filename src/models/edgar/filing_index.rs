//! EDGAR filing index models.
//!
//! Models for the filing directory index at:
//! `https://data.sec.gov/Archives/edgar/data/{cik}/{accession}/index.json`.

use serde::{Deserialize, Serialize};

/// Filing index response for a specific EDGAR accession.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarFilingIndex {
    /// Directory listing metadata.
    #[serde(default)]
    pub directory: EdgarFilingIndexDirectory,
}

/// Directory metadata for an EDGAR filing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct EdgarFilingIndexDirectory {
    /// Listing of files for the filing.
    #[serde(default)]
    pub item: Vec<EdgarFilingIndexItem>,
}

/// Single file entry within an EDGAR filing index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarFilingIndexItem {
    /// File name (e.g., "aapl-20240928.htm").
    pub name: String,

    /// File type (often the form type like "10-K").
    #[serde(default, rename = "type")]
    pub item_type: String,

    /// File size in bytes.
    #[serde(default)]
    pub size: u64,
}
