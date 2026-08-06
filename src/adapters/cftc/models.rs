//! CFTC Commitments of Traders (disaggregated futures-only) wire types.
//!
//! `publicreporting.cftc.gov`'s Socrata API serves every numeric column as a
//! JSON string (Socrata's `number` column type round-trips through strings),
//! so every figure here is `Option<String>` and parsed on the way to the
//! canonical model. Only the columns this adapter maps are modelled — the
//! full dataset carries well over a hundred (old/other/all variants,
//! percent-of-open-interest, trader counts, concentration ratios, …).

use serde::Deserialize;

/// One weekly row of the disaggregated futures-only combined report.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CotRow {
    #[serde(default)]
    pub market_and_exchange_names: Option<String>,
    #[serde(default)]
    pub cftc_contract_market_code: Option<String>,
    #[serde(default)]
    pub report_date_as_yyyy_mm_dd: Option<String>,
    #[serde(default)]
    pub open_interest_all: Option<String>,
    #[serde(default)]
    pub prod_merc_positions_long: Option<String>,
    #[serde(default)]
    pub prod_merc_positions_short: Option<String>,
    #[serde(default)]
    pub swap_positions_long_all: Option<String>,
    // Socrata's own column name has the double underscore — not a typo.
    #[serde(default, rename = "swap__positions_short_all")]
    pub swap_positions_short_all: Option<String>,
    #[serde(default, rename = "swap__positions_spread_all")]
    pub swap_positions_spread_all: Option<String>,
    #[serde(default)]
    pub m_money_positions_long_all: Option<String>,
    #[serde(default)]
    pub m_money_positions_short_all: Option<String>,
    #[serde(default)]
    pub m_money_positions_spread: Option<String>,
    #[serde(default)]
    pub other_rept_positions_long: Option<String>,
    #[serde(default)]
    pub other_rept_positions_short: Option<String>,
    #[serde(default)]
    pub other_rept_positions_spread: Option<String>,
    #[serde(default)]
    pub tot_rept_positions_long_all: Option<String>,
    #[serde(default)]
    pub tot_rept_positions_short: Option<String>,
    #[serde(default)]
    pub nonrept_positions_long_all: Option<String>,
    #[serde(default)]
    pub nonrept_positions_short_all: Option<String>,
}

/// A rejected-query error body: `{"message": "..."}`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CftcError {
    #[serde(default)]
    pub message: Option<String>,
}
