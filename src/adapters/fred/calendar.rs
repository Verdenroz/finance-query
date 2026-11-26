//! Market-wide economic-release calendar events built from FRED release dates.

use super::models::ReleaseDate;
use crate::models::calendar::{CalendarEvent, EventKind};

/// FRED release IDs for the major, market-moving US economic releases surfaced
/// in the calendar.
///
/// FRED's `releases/dates` feed lists ~300 releases, most of them niche or
/// sub-national (state retail sales, research indices, etc.). Restricting to
/// this curated set keeps the calendar focused on the releases that actually
/// move markets, instead of burying per-ticker events under hundreds of rows.
const MAJOR_ECONOMIC_RELEASE_IDS: &[u64] = &[
    9,   // Advance Monthly Sales for Retail and Food Services (Retail Sales)
    10,  // Consumer Price Index (CPI)
    13,  // Industrial Production and Capacity Utilization
    46,  // Producer Price Index (PPI)
    50,  // Employment Situation (Nonfarm Payrolls)
    53,  // Gross Domestic Product (GDP)
    54,  // Personal Income and Outlays (PCE)
    101, // FOMC Press Release
    180, // Unemployment Insurance Weekly Claims (Jobless Claims)
    192, // Job Openings and Labor Turnover Survey (JOLTS)
];

/// Build market-wide economic-release events from FRED scheduled release dates.
///
/// FRED returns dates as `YYYY-MM-DD`; each is interpreted as midnight UTC.
/// Only releases in [`MAJOR_ECONOMIC_RELEASE_IDS`] that fall within `window`
/// are emitted.
pub(crate) fn build_economic_events(
    releases: Vec<ReleaseDate>,
    window: (i64, i64),
) -> Vec<CalendarEvent> {
    let (start, end) = window;
    let releases = drop_phantom_daily_fills(
        releases
            .into_iter()
            .filter(|r| MAJOR_ECONOMIC_RELEASE_IDS.contains(&r.release_id))
            .collect(),
    );
    releases
        .into_iter()
        .filter_map(|r| {
            let ts = parse_iso_date(&r.date)?;
            (ts >= start && ts <= end).then(|| {
                CalendarEvent::new(
                    ts,
                    None,
                    EventKind::EconomicRelease {
                        name: r.release_name,
                        series_id: r.release_id.to_string(),
                    },
                )
            })
        })
        .collect()
}

/// Drop FRED "no-data" phantom fills.
///
/// Queried with `include_release_dates_with_no_data`, FRED returns one row per
/// calendar day for releases that lack scheduled-date data — notably the FOMC
/// Press Release, which comes back as a run of ~30 consecutive daily rows. No
/// genuine macro release recurs on consecutive days, so any date inside a run
/// of 3+ consecutive calendar days for the same release is a phantom fill and
/// is discarded. Isolated dates and 2-day adjacencies are kept, so the filter
/// is independent of the calendar window size.
pub(super) fn drop_phantom_daily_fills(releases: Vec<ReleaseDate>) -> Vec<ReleaseDate> {
    use std::collections::{HashMap, HashSet};

    let day = |r: &ReleaseDate| parse_iso_date(&r.date).map(|ts| ts.div_euclid(86_400));

    let mut days_by_release: HashMap<u64, HashSet<i64>> = HashMap::new();
    for r in &releases {
        if let Some(ord) = day(r) {
            days_by_release.entry(r.release_id).or_default().insert(ord);
        }
    }

    releases
        .into_iter()
        .filter(|r| {
            let Some(ord) = day(r) else { return false };
            let days = &days_by_release[&r.release_id];
            let has = |d: i64| days.contains(&d);
            // Part of a 3+-day run iff one of these consecutive triples holds.
            let in_run = (has(ord - 2) && has(ord - 1))
                || (has(ord - 1) && has(ord + 1))
                || (has(ord + 1) && has(ord + 2));
            !in_run
        })
        .collect()
}

/// Parse an ISO `YYYY-MM-DD` date as the Unix timestamp of midnight UTC.
fn parse_iso_date(date: &str) -> Option<i64> {
    use chrono::NaiveDate;
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)?
        .and_utc()
        .timestamp()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn economic_events_filter_to_major_releases_in_window() {
        let releases = vec![
            ReleaseDate {
                release_id: 10,
                release_name: "Consumer Price Index".to_string(),
                date: "2026-07-15".to_string(),
            },
            // Niche release → excluded by the curated whitelist.
            ReleaseDate {
                release_id: 742,
                release_name: "Bankrate Monitor".to_string(),
                date: "2026-07-15".to_string(),
            },
            // Major release but outside the window → excluded.
            ReleaseDate {
                release_id: 50,
                release_name: "Employment Situation".to_string(),
                date: "2099-01-01".to_string(),
            },
        ];
        let window = (
            parse_iso_date("2026-07-01").unwrap(),
            parse_iso_date("2026-07-31").unwrap(),
        );
        let events = build_economic_events(releases, window);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].symbol, None);
        match &events[0].event {
            EventKind::EconomicRelease { name, series_id } => {
                assert_eq!(name, "Consumer Price Index");
                assert_eq!(series_id, "10");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn drops_phantom_daily_fills_keeps_real_schedules() {
        // FOMC (id 101): FRED returns one phantom row per day, Jul 10–20 (11
        // consecutive days) — all must be dropped. CPI (id 10): a single real
        // scheduled date survives. Jobless Claims (id 180): weekly (7-day gaps)
        // survive as isolated dates.
        let mut releases = Vec::new();
        for day in 10..=20 {
            releases.push(ReleaseDate {
                release_id: 101,
                release_name: "FOMC Press Release".to_string(),
                date: format!("2026-07-{day:02}"),
            });
        }
        releases.push(ReleaseDate {
            release_id: 10,
            release_name: "Consumer Price Index".to_string(),
            date: "2026-07-14".to_string(),
        });
        releases.push(ReleaseDate {
            release_id: 180,
            release_name: "Unemployment Insurance Weekly Claims Report".to_string(),
            date: "2026-07-09".to_string(),
        });
        releases.push(ReleaseDate {
            release_id: 180,
            release_name: "Unemployment Insurance Weekly Claims Report".to_string(),
            date: "2026-07-16".to_string(),
        });

        let window = (
            parse_iso_date("2026-07-01").unwrap(),
            parse_iso_date("2026-07-31").unwrap(),
        );
        let events = build_economic_events(releases, window);

        // No FOMC (all phantom), CPI once, two weekly-claims dates.
        assert!(
            !events
                .iter()
                .any(|e| matches!(&e.event, EventKind::EconomicRelease { series_id, .. } if series_id == "101")),
            "phantom FOMC daily fills must be dropped"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(&e.event, EventKind::EconomicRelease { series_id, .. } if series_id == "10"))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(&e.event, EventKind::EconomicRelease { series_id, .. } if series_id == "180"))
                .count(),
            2
        );
    }
}
