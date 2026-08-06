//! `FUTURES` capability for CFTC — Commitments of Traders positioning.
//!
//! CFTC publishes no price quotes, so this adapter only ever serves
//! [`fetch_commitments_of_traders_response`]; the provider bridge reports
//! `NotSupported` for a plain futures quote.

use crate::error::{FinanceError, Result};
use crate::models::futures::cot::{CommitmentsOfTraders, CotObservation};

use super::models::CotRow;

/// Curated root-symbol → CFTC `cftc_contract_market_code` table, covering
/// the benchmark contract per commodity group in the disaggregated
/// futures-only report. Keys are the Yahoo-style continuous futures symbols
/// used elsewhere in this library (e.g. `providers.futures("GC=F")`).
///
/// Anything not listed here is treated as a literal CFTC contract code
/// already — the passthrough form, e.g. `providers.futures("067651")` for
/// NYMEX WTI directly, or any other code from CFTC's own market list.
const CURATED_CODES: &[(&str, &str)] = &[
    ("GC=F", "088691"), // Gold - COMMODITY EXCHANGE INC.
    ("SI=F", "084691"), // Silver - COMMODITY EXCHANGE INC.
    ("PL=F", "076651"), // Platinum - NEW YORK MERCANTILE EXCHANGE
    ("HG=F", "085692"), // Copper- #1 - COMMODITY EXCHANGE INC.
    ("CL=F", "067651"), // WTI-PHYSICAL - NEW YORK MERCANTILE EXCHANGE
    ("NG=F", "03565B"), // HENRY HUB - NEW YORK MERCANTILE EXCHANGE
    ("ZC=F", "002602"), // CORN - CHICAGO BOARD OF TRADE
    ("ZW=F", "001602"), // WHEAT-SRW - CHICAGO BOARD OF TRADE
    ("ZS=F", "005602"), // SOYBEANS - CHICAGO BOARD OF TRADE
];

/// Resolve a futures symbol to a CFTC `cftc_contract_market_code`.
///
/// Recognised Yahoo-style continuous futures roots (`"GC=F"`, `"CL=F"`, …)
/// resolve through [`CURATED_CODES`]; anything else is treated as a literal
/// contract code, uppercased and trimmed.
pub(crate) fn resolve_contract_code(symbol: &str) -> String {
    let upper = symbol.trim().to_uppercase();
    CURATED_CODES
        .iter()
        .find(|(root, _)| *root == upper)
        .map(|(_, code)| (*code).to_string())
        .unwrap_or(upper)
}

/// Parse a Socrata numeric-as-string field into `Option<i64>`. Values arrive
/// as decimal strings (e.g. `"316244"`); a missing or unparseable field
/// becomes `None` rather than `0`, so "not reported" is never confused with
/// "reported as zero".
fn parse_i64(value: &Option<String>) -> Option<i64> {
    value.as_deref().and_then(|v| v.trim().parse::<i64>().ok())
}

/// The report date column arrives as a full timestamp
/// (`"2026-07-28T00:00:00.000"`); only the date part is documented.
fn report_date(row: &CotRow) -> String {
    row.report_date_as_yyyy_mm_dd
        .as_deref()
        .map(|d| d.split('T').next().unwrap_or(d).to_string())
        .unwrap_or_default()
}

fn to_observation(row: &CotRow) -> CotObservation {
    CotObservation {
        report_date: report_date(row),
        open_interest: parse_i64(&row.open_interest_all),
        producer_merchant_long: parse_i64(&row.prod_merc_positions_long),
        producer_merchant_short: parse_i64(&row.prod_merc_positions_short),
        swap_dealer_long: parse_i64(&row.swap_positions_long_all),
        swap_dealer_short: parse_i64(&row.swap_positions_short_all),
        swap_dealer_spread: parse_i64(&row.swap_positions_spread_all),
        managed_money_long: parse_i64(&row.m_money_positions_long_all),
        managed_money_short: parse_i64(&row.m_money_positions_short_all),
        managed_money_spread: parse_i64(&row.m_money_positions_spread),
        other_reportable_long: parse_i64(&row.other_rept_positions_long),
        other_reportable_short: parse_i64(&row.other_rept_positions_short),
        other_reportable_spread: parse_i64(&row.other_rept_positions_spread),
        total_reportable_long: parse_i64(&row.tot_rept_positions_long_all),
        total_reportable_short: parse_i64(&row.tot_rept_positions_short),
        nonreportable_long: parse_i64(&row.nonrept_positions_long_all),
        nonreportable_short: parse_i64(&row.nonrept_positions_short_all),
    }
}

/// Map raw rows (newest-first, the API's own order) onto the canonical
/// [`CommitmentsOfTraders`], reversed into chronological order to match
/// every other history-shaped model in the library.
pub(crate) fn to_canonical(symbol: &str, mut rows: Vec<CotRow>) -> Result<CommitmentsOfTraders> {
    if rows.is_empty() {
        return Err(FinanceError::SymbolNotFound {
            symbol: Some(symbol.to_string()),
            context: "CFTC reported no Commitments of Traders rows for this contract code/symbol"
                .to_string(),
        });
    }
    rows.reverse();

    let market_and_exchange_name = rows
        .last()
        .and_then(|r| r.market_and_exchange_names.clone())
        .unwrap_or_default();
    let cftc_contract_market_code = rows
        .last()
        .and_then(|r| r.cftc_contract_market_code.clone())
        .unwrap_or_default();
    let observations = rows.iter().map(to_observation).collect();

    Ok(CommitmentsOfTraders {
        symbol: symbol.to_string(),
        market_and_exchange_name,
        cftc_contract_market_code,
        observations,
    })
}

/// Fetch the canonical Commitments of Traders series for a futures symbol.
pub async fn fetch_commitments_of_traders_response(symbol: &str) -> Result<CommitmentsOfTraders> {
    let code = resolve_contract_code(symbol);
    let rows = super::client()?.commitments_of_traders(&code).await?;
    to_canonical(symbol, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_root_symbols_resolve_to_their_contract_code() {
        assert_eq!(resolve_contract_code("GC=F"), "088691");
        assert_eq!(resolve_contract_code("cl=f"), "067651");
        assert_eq!(resolve_contract_code(" ng=f "), "03565B");
    }

    #[test]
    fn unrecognised_symbol_passes_through_as_a_literal_contract_code() {
        assert_eq!(resolve_contract_code("088691"), "088691");
        assert_eq!(resolve_contract_code("06765a"), "06765A");
    }

    #[test]
    fn missing_or_unparseable_figures_are_none_not_zero() {
        let row = CotRow {
            market_and_exchange_names: None,
            cftc_contract_market_code: None,
            report_date_as_yyyy_mm_dd: None,
            open_interest_all: None,
            prod_merc_positions_long: Some("not-a-number".to_string()),
            prod_merc_positions_short: None,
            swap_positions_long_all: None,
            swap_positions_short_all: None,
            swap_positions_spread_all: None,
            m_money_positions_long_all: None,
            m_money_positions_short_all: None,
            m_money_positions_spread: None,
            other_rept_positions_long: None,
            other_rept_positions_short: None,
            other_rept_positions_spread: None,
            tot_rept_positions_long_all: None,
            tot_rept_positions_short: None,
            nonrept_positions_long_all: None,
            nonrept_positions_short_all: None,
        };
        let obs = to_observation(&row);
        assert_eq!(obs.open_interest, None);
        assert_eq!(obs.producer_merchant_long, None);
    }

    #[test]
    fn report_date_trims_the_time_of_day_component() {
        let mut row = blank_row();
        row.report_date_as_yyyy_mm_dd = Some("2026-07-28T00:00:00.000".to_string());
        assert_eq!(report_date(&row), "2026-07-28");
    }

    #[test]
    fn empty_rows_map_to_symbol_not_found() {
        let err = to_canonical("NOSUCHCODE", Vec::new()).unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn rows_are_reversed_into_chronological_order() {
        let mut newest = blank_row();
        newest.report_date_as_yyyy_mm_dd = Some("2026-07-28T00:00:00.000".to_string());
        newest.market_and_exchange_names = Some("GOLD - COMMODITY EXCHANGE INC.".to_string());
        newest.cftc_contract_market_code = Some("088691".to_string());
        newest.open_interest_all = Some("384603".to_string());

        let mut oldest = blank_row();
        oldest.report_date_as_yyyy_mm_dd = Some("2026-07-14T00:00:00.000".to_string());
        oldest.market_and_exchange_names = Some("GOLD - COMMODITY EXCHANGE INC.".to_string());
        oldest.cftc_contract_market_code = Some("088691".to_string());
        oldest.open_interest_all = Some("383689".to_string());

        // API order: newest first.
        let series = to_canonical("GC=F", vec![newest, oldest]).unwrap();
        assert_eq!(
            series.market_and_exchange_name,
            "GOLD - COMMODITY EXCHANGE INC."
        );
        assert_eq!(series.cftc_contract_market_code, "088691");
        assert_eq!(series.observations.len(), 2);
        assert_eq!(series.observations[0].report_date, "2026-07-14");
        assert_eq!(series.observations[1].report_date, "2026-07-28");
    }

    fn blank_row() -> CotRow {
        CotRow {
            market_and_exchange_names: None,
            cftc_contract_market_code: None,
            report_date_as_yyyy_mm_dd: None,
            open_interest_all: None,
            prod_merc_positions_long: None,
            prod_merc_positions_short: None,
            swap_positions_long_all: None,
            swap_positions_short_all: None,
            swap_positions_spread_all: None,
            m_money_positions_long_all: None,
            m_money_positions_short_all: None,
            m_money_positions_spread: None,
            other_rept_positions_long: None,
            other_rept_positions_short: None,
            other_rept_positions_spread: None,
            tot_rept_positions_long_all: None,
            tot_rept_positions_short: None,
            nonrept_positions_long_all: None,
            nonrept_positions_short_all: None,
        }
    }
}
