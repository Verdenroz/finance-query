//! Senate eFD DOM-derived row types.

/// One row of the search results table: a filer plus their PTR report link.
#[derive(Debug, Clone)]
pub(super) struct SenateFilingEntry {
    pub first_name: String,
    pub last_name: String,
    /// `MM/DD/YYYY`, as rendered in the "Date Received/Filed" column.
    pub filed_date: String,
    /// Relative path to the report, e.g. `/search/view/ptr/{uuid}/`.
    pub path: String,
}
