//! World Bank Open Data wire types.
//!
//! The v2 API answers every indicator query with a two-element JSON array:
//! `[metadata, observations]` — or a single-element array carrying `message`
//! when the request was rejected. Both shapes are modelled here.

use serde::Deserialize;

/// A `{ "id": ..., "value": ... }` pair — how the API names indicators and
/// countries. Only the display half is kept; the id half is already known
/// from the request.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct NamedRef {
    pub value: Option<String>,
}

/// One observation of one indicator for one country in one period.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorldBankObservation {
    pub indicator: Option<NamedRef>,
    pub country: Option<NamedRef>,
    /// Period label: `"2023"`, `"2023Q1"`, or `"2023M01"`.
    pub date: String,
    pub value: Option<f64>,
    /// Usually empty — most indicators carry their unit inside the title.
    #[serde(default)]
    pub unit: Option<String>,
}

/// Header returned as the first array element. Its pagination fields are
/// ignored — one page is requested large enough to hold any series.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorldBankPage {
    /// Present instead of the pagination fields when the request was rejected.
    #[serde(default)]
    pub message: Option<Vec<WorldBankMessage>>,
}

/// An error entry from a rejected request.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorldBankMessage {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
}

impl WorldBankMessage {
    /// Human-readable form for error reporting, preferring the detail text.
    pub(crate) fn describe(&self) -> String {
        match (&self.key, &self.value) {
            (Some(k), Some(v)) => format!("{k}: {v}"),
            (Some(k), None) => k.clone(),
            (None, Some(v)) => v.clone(),
            (None, None) => "unspecified error".to_string(),
        }
    }
}

/// The full `[metadata, observations]` envelope.
///
/// The observations element is absent on error responses, hence the `Option`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorldBankResponse(
    pub WorldBankPage,
    #[serde(default)] pub Option<Vec<WorldBankObservation>>,
);
