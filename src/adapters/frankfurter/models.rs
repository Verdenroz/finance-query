//! Frankfurter (ECB reference rates) wire types.

use serde::Deserialize;
use std::collections::BTreeMap;

/// A date-range response: `rates` maps each published date to that day's
/// rates. `BTreeMap` so iteration is already chronological.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FrankfurterTimeSeries {
    pub base: String,
    pub rates: BTreeMap<String, BTreeMap<String, f64>>,
}

/// The error body Frankfurter returns for an unknown or invalid pair.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct FrankfurterError {
    #[serde(default)]
    pub message: Option<String>,
}
