//! CFTC Commitments of Traders models.
//!
//! Populated by the CFTC adapter (`cftc` feature). Reports weekly futures
//! positioning by trader category: commercial hedgers (producer/merchant),
//! swap dealers, managed money (large speculators), other reportables, and
//! small traders (the "nonreportable" residual below CFTC reporting
//! thresholds). Source: the disaggregated futures-only combined report —
//! physical commodities only (agriculture, energy, metals). Equity, rate,
//! and currency futures are reported separately by the CFTC in the Traders
//! in Financial Futures report, which is not covered here.

use serde::{Deserialize, Serialize};

/// Weekly Commitments of Traders positioning for one futures market.
///
/// Obtain via [`FuturesContract::commitments_of_traders`](crate::FuturesContract::commitments_of_traders).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CommitmentsOfTraders {
    /// The symbol this was requested with (e.g. `"GC=F"`, or a raw CFTC
    /// `cftc_contract_market_code`).
    pub symbol: String,
    /// CFTC's own market and exchange name (e.g. `"GOLD - COMMODITY EXCHANGE INC."`).
    pub market_and_exchange_name: String,
    /// CFTC contract market code identifying this market.
    pub cftc_contract_market_code: String,
    /// Weekly observations, oldest first.
    pub observations: Vec<CotObservation>,
}

/// One weekly report row, broken down by trader category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CotObservation {
    /// Report date (`YYYY-MM-DD`) — the Tuesday the report is as of.
    pub report_date: String,
    /// Total open interest, all reporting categories combined.
    pub open_interest: Option<i64>,
    /// Commercial hedgers (producers/merchants/processors/users): long side.
    pub producer_merchant_long: Option<i64>,
    /// Commercial hedgers: short side.
    pub producer_merchant_short: Option<i64>,
    /// Swap dealers: long side.
    pub swap_dealer_long: Option<i64>,
    /// Swap dealers: short side.
    pub swap_dealer_short: Option<i64>,
    /// Swap dealers: spread positions.
    pub swap_dealer_spread: Option<i64>,
    /// Managed money (large speculators): long side.
    pub managed_money_long: Option<i64>,
    /// Managed money: short side.
    pub managed_money_short: Option<i64>,
    /// Managed money: spread positions.
    pub managed_money_spread: Option<i64>,
    /// Other reportable traders: long side.
    pub other_reportable_long: Option<i64>,
    /// Other reportable traders: short side.
    pub other_reportable_short: Option<i64>,
    /// Other reportable traders: spread positions.
    pub other_reportable_spread: Option<i64>,
    /// Sum of all reportable categories: long side.
    pub total_reportable_long: Option<i64>,
    /// Sum of all reportable categories: short side.
    pub total_reportable_short: Option<i64>,
    /// Small traders below CFTC reporting thresholds (residual): long side.
    pub nonreportable_long: Option<i64>,
    /// Small traders below CFTC reporting thresholds: short side.
    pub nonreportable_short: Option<i64>,
}
