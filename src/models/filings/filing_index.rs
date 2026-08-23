//! EDGAR filing index models.
//!
//! Models for the filing directory index at:
//! `https://www.sec.gov/Archives/edgar/data/{cik}/{accession}/index.json`.

use serde::{Deserialize, Deserializer, Serialize};

/// The wire value is always a JSON string ("24417" or "" for directory
/// entries with no size, e.g. the filing's own index pages).
fn deserialize_size<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse().ok())
}

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

    /// File-icon class from the directory listing (e.g. "text.gif"), not the
    /// exhibit or form type — SEC EDGAR doesn't expose that here.
    #[serde(default, rename = "type")]
    pub item_type: String,

    /// File size in bytes; `None` for entries with no size (e.g. the
    /// filing's own index pages).
    #[serde(default, deserialize_with = "deserialize_size")]
    pub size: Option<u64>,
}
