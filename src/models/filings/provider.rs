//! Provider-based SEC filing data models (Polygon EDGAR integration).
//!
//! Unlike [`crate::models::filings::EdgarFiling`] which comes from the direct
//! EDGAR adapter, these types represent the provider-agnostic canonical shape
//! returned by the FILINGS capability through ProviderSet.

use serde::{Deserialize, Serialize};

/// A collection of SEC filings from a provider (e.g., Polygon EDGAR).
///
/// Obtain via [`Ticker::filings`](crate::Ticker::filings).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct ProviderFilings {
    /// Ticker symbol these filings belong to.
    pub symbol: String,
    /// Individual filing entries.
    pub filings: Vec<ProviderFiling>,
}

/// A single SEC filing entry from a provider.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct ProviderFiling {
    /// SEC accession number (unique filing ID).
    pub accession_number: Option<String>,
    /// Filing date as `YYYY-MM-DD`.
    pub filing_date: Option<String>,
    /// Filing type (e.g., `"10-K"`, `"10-Q"`, `"8-K"`).
    pub filing_type: Option<String>,
    /// URL to the filing document.
    pub filing_url: Option<String>,
    /// Company name at time of filing.
    pub company_name: Option<String>,
    /// SEC CIK number.
    pub cik: Option<String>,
}

#[cfg(feature = "dataframe")]
impl ProviderFilings {
    /// Converts the filings to a polars DataFrame, one row per filing.
    ///
    /// Columns are `symbol` followed by the columns of `ProviderFiling::to_dataframe`.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use polars::prelude::*;

        let mut frame = ProviderFiling::vec_to_dataframe(&self.filings)?;
        let symbol = Series::new("symbol".into(), vec![self.symbol.as_str(); frame.height()]);
        frame.insert_column(0, symbol.into())?;
        Ok(frame)
    }
}
