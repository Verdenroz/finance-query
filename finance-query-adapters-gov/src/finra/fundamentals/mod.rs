//! `FUNDAMENTALS` capability for FINRA — daily short-sale volume.

use std::collections::BTreeMap;

use finance_query_adapters_core::error::Result;
use finance_query_models::fundamentals::{ShortVolume, ShortVolumeFieldsInit};

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
/// Since 2026-02-23 FINRA reports fractional-share volume (e.g.
/// `shortParQuantity: 5866862.70509`). Each consolidated figure is rounded to
/// whole shares after summing, so facility fractions do not compound.
///
/// A `BTreeMap` keyed by ISO date gives chronological order for free, which
/// matters because FINRA rejects server-side sorting on a date *range* query.
pub(super) fn consolidate(rows: Vec<RegShoDailyRow>) -> Vec<ShortVolume> {
    let mut by_date: BTreeMap<String, ShortVolume> = BTreeMap::new();

    for row in rows {
        let entry = by_date
            .entry(row.trade_report_date.clone())
            .or_insert_with(|| {
                ShortVolume::from(ShortVolumeFieldsInit {
                    date: Some(row.trade_report_date.clone()),
                    short_volume: None,
                    short_exempt_volume: None,
                    total_volume: None,
                })
            });
        add(&mut entry.short_volume, row.short_par_quantity);
        add(
            &mut entry.short_exempt_volume,
            row.short_exempt_par_quantity,
        );
        add(&mut entry.total_volume, row.total_par_quantity);
    }

    by_date
        .into_values()
        .map(|v| {
            ShortVolume::from(ShortVolumeFieldsInit {
                short_volume: v.short_volume.map(f64::round),
                short_exempt_volume: v.short_exempt_volume.map(f64::round),
                total_volume: v.total_volume.map(f64::round),
                ..ShortVolumeFieldsInit::from(v)
            })
        })
        .collect()
}

/// Accumulate an optional addend, leaving the total `None` only when no
/// facility reported the figure at all.
fn add(total: &mut Option<f64>, addend: Option<f64>) {
    if let Some(value) = addend {
        *total = Some(total.unwrap_or(0.0) + value);
    }
}

/// Fetch a symbol's daily short-sale volume series, oldest first.
pub async fn fetch_short_volume_response(symbol: &str) -> Result<Vec<ShortVolume>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn row(date: &str, short: f64, exempt: f64, total: f64) -> RegShoDailyRow {
        RegShoDailyRow {
            trade_report_date: date.to_string(),
            short_par_quantity: Some(short),
            short_exempt_par_quantity: Some(exempt),
            total_par_quantity: Some(total),
        }
    }

    #[test]
    fn fractional_facility_rows_sum_to_whole_shares() {
        let rows = vec![
            row("2026-11-11", 88868.810764, 21.0, 155017.189953),
            row("2026-11-11", 5866862.70509, 47184.0, 10993647.679657),
            row("2026-11-11", 590394.02746, 227.0, 1022697.49758),
        ];
        let out = consolidate(rows);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].short_volume, Some(6546126.0));
        assert_eq!(out[0].short_exempt_volume, Some(47432.0));
        assert_eq!(out[0].total_volume, Some(12171362.0));
    }

    #[test]
    fn integer_rows_are_unchanged() {
        let out = consolidate(vec![row("2026-02-17", 103698.0, 0.0, 237320.0)]);
        assert_eq!(out[0].short_volume, Some(103698.0));
    }
}
