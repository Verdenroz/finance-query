//! House Clerk bulk archive wire types.
//!
//! The yearly ZIP's `{year}FD.txt` is a tab-delimited index, one row per
//! filing: `Prefix Last First Suffix FilingType StateDst Year FilingDate DocID`.
//! `FilingType == "P"` marks a Periodic Transaction Report; every other code
//! (annual reports, amendments, candidate filings, …) is out of scope here.

/// One row of the yearly filing index, already filtered to PTRs.
#[derive(Debug, Clone)]
pub(super) struct PtrIndexEntry {
    pub last: String,
    pub first: String,
    /// `"AL04"` — two-letter state plus zero-padded district.
    pub state_dst: String,
    pub year: i32,
    /// As filed, `M/D/YYYY`.
    pub filing_date: String,
    pub doc_id: String,
}
