//! Security-identifier mapping models.
//!
//! Populated by the OpenFIGI adapter, which resolves a CUSIP, ISIN, SEDOL, or
//! FIGI to the instruments carrying it.

use serde::{Deserialize, Serialize};

/// One instrument matching a security identifier.
///
/// A single CUSIP or ISIN maps to **many** instruments — one per venue the
/// security trades on — which is why resolution returns a list. Entries that
/// share a `composite_figi` are the same security on different venues; the
/// `share_class_figi` groups share classes across countries.
///
/// Obtain via [`openfigi::resolve_cusip`](crate::openfigi::resolve_cusip) and
/// friends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SecurityMapping {
    /// The instrument's Financial Instrument Global Identifier.
    pub figi: String,
    /// Ticker symbol, as the venue lists it.
    pub ticker: Option<String>,
    /// Security name (usually the issuer).
    pub name: Option<String>,
    /// Exchange code — `"US"` for the composite, otherwise a venue code.
    pub exchange_code: Option<String>,
    /// FIGI of the country-level composite this instrument rolls up to.
    pub composite_figi: Option<String>,
    /// FIGI shared by every listing of this share class worldwide.
    pub share_class_figi: Option<String>,
    /// Instrument type, e.g. `"Common Stock"`, `"ETP"`.
    pub security_type: Option<String>,
    /// Market sector, e.g. `"Equity"`, `"Corp"`, `"Govt"`.
    pub market_sector: Option<String>,
}

/// The kind of identifier being resolved.
///
/// Maps onto OpenFIGI's `idType` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SecurityIdKind {
    /// North American security identifier (9 characters).
    Cusip,
    /// International Securities Identification Number (12 characters).
    Isin,
    /// UK/Ireland security identifier (7 characters).
    Sedol,
    /// A FIGI, resolved to its siblings.
    Figi,
    /// Exchange ticker symbol.
    Ticker,
}

impl SecurityIdKind {
    /// The `idType` string OpenFIGI expects.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cusip => "ID_CUSIP",
            Self::Isin => "ID_ISIN",
            Self::Sedol => "ID_SEDOL",
            Self::Figi => "ID_BB_GLOBAL",
            Self::Ticker => "TICKER",
        }
    }
}

impl std::fmt::Display for SecurityIdKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
