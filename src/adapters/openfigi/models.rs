//! OpenFIGI v3 mapping wire types.

use serde::{Deserialize, Serialize};

/// One mapping job in a request body.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MappingJob<'a> {
    #[serde(rename = "idType")]
    pub id_type: &'a str,
    #[serde(rename = "idValue")]
    pub id_value: &'a str,
}

/// One element of the response array, positionally paired with the job that
/// produced it.
///
/// Exactly one of the three arms is present per element: `data` on a hit,
/// `warning` when nothing matched, `error` when the job itself was malformed.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MappingResult {
    #[serde(default)]
    pub data: Option<Vec<FigiRecord>>,
    /// Set to `"No identifier found."` when the identifier is valid but
    /// matches nothing — not an error.
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// One instrument OpenFIGI knows about.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FigiRecord {
    pub figi: String,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "exchCode")]
    pub exch_code: Option<String>,
    // OpenFIGI capitalises the acronym (`compositeFIGI`), which plain
    // camelCase would render as `compositeFigi` and silently never match.
    #[serde(default, rename = "compositeFIGI")]
    pub composite_figi: Option<String>,
    #[serde(default, rename = "shareClassFIGI")]
    pub share_class_figi: Option<String>,
    #[serde(default)]
    pub security_type: Option<String>,
    #[serde(default)]
    pub market_sector: Option<String>,
}
