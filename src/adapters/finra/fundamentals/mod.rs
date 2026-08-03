//! `FUNDAMENTALS` capability for FINRA — daily short-sale volume.

use std::collections::BTreeMap;

use crate::error::Result;
use crate::models::fundamentals::ShortVolume;

use super::models::RegShoDailyRow;

/// Days of history requested. FINRA keeps roughly a rolling year online.
const LOOKBACK_DAYS: i64 = 365;

/// Consolidate FINRA's per-facility rows into one figure per trade date.
///
/// FINRA reports each symbol separately for every reporting facility it traded
/// on that day (`B`/`Q`/`N`), so the raw rows are three-per-day for a listed
/// name. Summing them reproduces the consolidated figure FINRA itself
/// publishes in its daily `CNMSshvol` file.
///
/// A `BTreeMap` keyed by ISO date gives chronological order for free, which
/// matters because FINRA rejects server-side sorting on a date *range* query.
pub(super) fn consolidate(rows: Vec<RegShoDailyRow>) -> Vec<ShortVolume> {
    let mut by_date: BTreeMap<String, ShortVolume> = BTreeMap::new();

    for row in rows {
        let entry = by_date
            .entry(row.trade_report_date.clone())
            .or_insert_with(|| ShortVolume {
                date: Some(row.trade_report_date.clone()),
                short_volume: None,
                short_exempt_volume: None,
                total_volume: None,
            });
        add(&mut entry.short_volume, row.short_par_quantity);
        add(
            &mut entry.short_exempt_volume,
            row.short_exempt_par_quantity,
        );
        add(&mut entry.total_volume, row.total_par_quantity);
    }

    by_date.into_values().collect()
}

/// Accumulate an optional addend, leaving the total `None` only when no
/// facility reported the figure at all.
fn add(total: &mut Option<f64>, addend: Option<f64>) {
    if let Some(value) = addend {
        *total = Some(total.unwrap_or(0.0) + value);
    }
}

/// Fetch a symbol's daily short-sale volume series, oldest first.
pub(crate) async fn fetch_short_volume_response(symbol: &str) -> Result<Vec<ShortVolume>> {
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::days(LOOKBACK_DAYS);

    let rows = super::client()?
        .reg_sho_daily(
            &symbol.to_uppercase(),
            start.format("%Y-%m-%d").to_string(),
            end.format("%Y-%m-%d").to_string(),
        )
        .await?;

    Ok(consolidate(rows))
}
