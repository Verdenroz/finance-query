use crate::Provider;
use serde::{Deserialize, Serialize};

/// Public dividend data
///
/// Note: This struct cannot be manually constructed - obtain via `Ticker::dividends()`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    feature = "dataframe",
    derive(crate::ToDataFrame),
    dataframe(crate = "crate")
)]
pub struct Dividend {
    /// Timestamp (Unix)
    #[cfg_attr(feature = "dataframe", dataframe(datetime = "s"))]
    pub timestamp: i64,
    /// Dividend amount per share
    pub amount: f64,

    /// Which data provider served this data (e.g., "yahoo", "polygon").
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub provider_id: Option<Provider>,
}

impl Dividend {
    /// Builds a dividend from every field. Internal support for sibling crates, not public API.
    #[doc(hidden)]
    pub fn __new(timestamp: i64, amount: f64, provider_id: Option<Provider>) -> Self {
        Self {
            timestamp,
            amount,
            provider_id,
        }
    }
}
