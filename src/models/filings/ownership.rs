//! Ownership models parsed from primary-source SEC filings.
//!
//! Served through the [`Capability::FILINGS`](crate::Capability::FILINGS)
//! route; EDGAR is currently the only provider. Unlike the aggregated holder
//! summaries on [`Ticker`](crate::Ticker), these come straight from the filed
//! XML — Forms 3/4/5 for insider activity and 13F-HR information tables for
//! institutional positions.

use serde::{Deserialize, Serialize};

/// One transaction line from a Form 3, 4, or 5.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InsiderTrade {
    /// Issuer's trading symbol, as filed.
    pub symbol: Option<String>,
    /// Issuer name, as filed.
    pub issuer_name: Option<String>,
    /// Reporting insider's name.
    pub insider_name: Option<String>,
    /// Reporting insider's CIK.
    pub insider_cik: Option<String>,
    /// Whether the insider is a director of the issuer.
    pub is_director: bool,
    /// Whether the insider is an officer of the issuer.
    pub is_officer: bool,
    /// Whether the insider holds 10% or more of a class of equity.
    pub is_ten_percent_owner: bool,
    /// Officer title, when one was filed.
    pub officer_title: Option<String>,
    /// Form the transaction was reported on (`"3"`, `"4"`, `"5"`).
    pub form_type: Option<String>,
    /// Accession number of the source filing.
    pub accession_number: Option<String>,
    /// URL of the source XML document.
    pub url: Option<String>,
    /// Security traded (e.g. `"Common Stock"`).
    pub security_title: Option<String>,
    /// Transaction date (`YYYY-MM-DD`).
    pub transaction_date: Option<String>,
    /// SEC transaction code (`"P"` purchase, `"S"` sale, `"M"` option exercise…).
    pub transaction_code: Option<String>,
    /// `"A"` if shares were acquired, `"D"` if disposed.
    pub acquired_disposed: Option<String>,
    /// Number of shares (or derivative units) transacted.
    pub shares: Option<f64>,
    /// Price per share, when reported.
    pub price_per_share: Option<f64>,
    /// Shares beneficially owned after the transaction.
    pub shares_owned_after: Option<f64>,
    /// Whether the line came from the derivative rather than non-derivative table.
    pub is_derivative: bool,
}

/// One position from a 13F-HR information table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InstitutionalHolding {
    /// Issuer name, as filed.
    pub issuer_name: Option<String>,
    /// Class of security (e.g. `"COM"`).
    pub title_of_class: Option<String>,
    /// CUSIP of the security.
    pub cusip: Option<String>,
    /// Market value as filed. Filings before 2023 report thousands of dollars;
    /// later ones report whole dollars. The value is passed through unscaled.
    pub value: Option<f64>,
    /// Share or principal amount held.
    pub shares: Option<f64>,
    /// Whether `shares` is a share count (`"SH"`) or principal amount (`"PRN"`).
    pub share_type: Option<String>,
    /// `"CALL"` or `"PUT"` when the position is an option.
    pub put_call: Option<String>,
    /// Investment discretion (`"SOLE"`, `"DFND"`, `"OTR"`).
    pub investment_discretion: Option<String>,
    /// Shares over which the filer has sole voting authority.
    pub voting_sole: Option<f64>,
    /// Shares over which voting authority is shared.
    pub voting_shared: Option<f64>,
    /// Shares over which the filer has no voting authority.
    pub voting_none: Option<f64>,
    /// Accession number of the source filing.
    pub accession_number: Option<String>,
    /// Period the holdings are reported as of (`YYYY-MM-DD`).
    pub report_date: Option<String>,
}
