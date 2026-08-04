//! US Treasury FiscalData wire types.
//!
//! Every dataset answers with the same envelope: a `data` array of flat
//! string-valued rows, plus a `meta` block that self-describes the columns.

use serde::Deserialize;
use std::collections::HashMap;

/// One row of a dataset. Every column arrives as a JSON string, including
/// numbers and the literal `"null"` used for a missing figure.
pub(crate) type FiscalRow = HashMap<String, String>;

/// Column self-description returned alongside every response.
#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct FiscalMeta {
    /// Human-readable column titles, e.g. `tot_pub_debt_out_amt` →
    /// `"Total Public Debt Outstanding"`.
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Column types, e.g. `"CURRENCY"`, `"PERCENTAGE"`, `"DATE"`.
    #[serde(default, rename = "dataTypes")]
    pub data_types: HashMap<String, String>,
    #[serde(default, rename = "total-pages")]
    pub total_pages: Option<u32>,
}

/// A successful dataset response.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FiscalResponse {
    #[serde(default)]
    pub data: Vec<FiscalRow>,
    #[serde(default)]
    pub meta: FiscalMeta,
}

/// The error body FiscalData returns for a malformed query (bad field name,
/// unparseable filter, …) — served with a non-2xx status.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FiscalError {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

impl FiscalError {
    /// Human-readable form for error reporting, preferring the detail message.
    pub(crate) fn describe(&self) -> Option<String> {
        match (&self.error, &self.message) {
            (Some(e), Some(m)) => Some(format!("{e}: {m}")),
            (Some(e), None) => Some(e.clone()),
            (None, Some(m)) => Some(m.clone()),
            (None, None) => None,
        }
    }
}
