//! SEC EDGAR data models.
//!
//! Models for EDGAR API responses including filing history (submissions),
//! structured XBRL financial data (company facts), and full-text search results.

mod cik;
mod company_facts;
pub mod filing_index;
mod search;
mod submissions;

pub use cik::CikEntry;
pub use company_facts::{CompanyFacts, FactConcept, FactUnit, FactsByTaxonomy};
pub use filing_index::EdgarFilingIndex;
pub use search::{
    EdgarSearchHit, EdgarSearchHitsContainer, EdgarSearchResults, EdgarSearchSource,
    EdgarSearchTotal,
};
pub use submissions::{
    EdgarFiling, EdgarFilingFile, EdgarFilingRecent, EdgarFilings, EdgarSubmissions,
};
