//! Financial event calendar models.
//!
//! A [`CalendarEvent`] is a single upcoming financial event — earnings,
//! ex-dividend/dividend-payment, options expiration, or (with the `fred`
//! feature) a market-wide economic-data release.
//!
//! Construct calendars via [`Ticker::calendar`](crate::Ticker::calendar) and
//! [`Tickers::calendar`](crate::Tickers::calendar).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::options::Options;
use crate::models::quote::CalendarEvents;

/// A single upcoming financial event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct CalendarEvent {
    /// Unix timestamp (seconds) when the event occurs.
    pub timestamp: i64,
    /// ISO 8601 date string for display (e.g. `"2026-01-23"`).
    pub date: String,
    /// Ticker symbol this event belongs to. `None` for market-wide events.
    pub symbol: Option<String>,
    /// The specific event.
    pub event: EventKind,
}

/// The kind of financial event, with its event-specific payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    /// Upcoming earnings report with analyst estimate data.
    Earnings {
        /// Low analyst EPS estimate for the quarter.
        eps_estimate_low: Option<f64>,
        /// Average analyst EPS estimate for the quarter.
        eps_estimate_avg: Option<f64>,
        /// High analyst EPS estimate for the quarter.
        eps_estimate_high: Option<f64>,
        /// Average analyst revenue estimate for the quarter.
        revenue_estimate_avg: Option<i64>,
        /// Whether the date is an estimate (Yahoo flags upcoming dates as such).
        is_estimate: bool,
    },
    /// Ex-dividend date — shares must be held before this date to receive the dividend.
    ExDividend {
        /// Dividend amount per share, when known.
        amount: Option<f64>,
    },
    /// Dividend payment date — cash arrives in the account.
    DividendPayment {
        /// Dividend amount per share, when known.
        amount: Option<f64>,
    },
    /// Standard monthly options expiration (3rd Friday) for this ticker.
    ///
    /// Only standard monthly expirations are surfaced — daily and weekly
    /// expirations are omitted to keep the calendar focused on the dates that
    /// carry meaningful open interest.
    OptionsExpiration {
        /// Number of listed contracts (calls + puts) expiring on this date when
        /// a chain was loaded. `None` if only the expiration date is known.
        contract_count: Option<usize>,
    },
    /// Economic data release (requires the `fred` feature).
    #[cfg(feature = "fred")]
    EconomicRelease {
        /// Human-readable release name (e.g. "Consumer Price Index").
        name: String,
        /// FRED release identifier as a string.
        series_id: String,
    },
}

impl CalendarEvent {
    /// Construct an event, deriving the ISO date string from the timestamp.
    pub(crate) fn new(timestamp: i64, symbol: Option<String>, event: EventKind) -> Self {
        Self {
            timestamp,
            date: iso_date(timestamp),
            symbol,
            event,
        }
    }
}

/// Format a Unix-second timestamp as an ISO `YYYY-MM-DD` UTC date string.
pub(crate) fn iso_date(timestamp: i64) -> String {
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Build the per-symbol calendar events from already-fetched data.
///
/// Pure and synchronous: callers fetch `calendar` (the `calendarEvents` quote
/// module) and `options` once, then hand them here. Only events whose timestamp
/// falls within `window` (`[start, end]`, inclusive) are emitted. The result is
/// **not** sorted — callers merge across symbols and sort once.
pub(crate) fn build_symbol_events(
    symbol: &str,
    calendar: Option<&CalendarEvents>,
    options: Option<&Options>,
    window: (i64, i64),
) -> Vec<CalendarEvent> {
    let (start, end) = window;
    let in_window = |ts: i64| ts >= start && ts <= end;
    let mut events = Vec::new();

    if let Some(cal) = calendar {
        if let Some(earnings) = &cal.earnings {
            let eps_estimate_low = earnings.earnings_low.as_ref().and_then(|v| v.raw);
            let eps_estimate_avg = earnings.earnings_average.as_ref().and_then(|v| v.raw);
            let eps_estimate_high = earnings.earnings_high.as_ref().and_then(|v| v.raw);
            let revenue_estimate_avg = earnings.revenue_average.as_ref().and_then(|v| v.raw);
            if let Some(dates) = &earnings.earnings_date {
                for ts in dates
                    .iter()
                    .filter_map(|d| d.raw)
                    .filter(|&ts| in_window(ts))
                {
                    events.push(CalendarEvent::new(
                        ts,
                        Some(symbol.to_string()),
                        EventKind::Earnings {
                            eps_estimate_low,
                            eps_estimate_avg,
                            eps_estimate_high,
                            revenue_estimate_avg,
                            is_estimate: true,
                        },
                    ));
                }
            }
        }

        if let Some(ts) = cal.ex_dividend_timestamp().filter(|&ts| in_window(ts)) {
            events.push(CalendarEvent::new(
                ts,
                Some(symbol.to_string()),
                EventKind::ExDividend { amount: None },
            ));
        }

        if let Some(ts) = cal.dividend_timestamp().filter(|&ts| in_window(ts)) {
            events.push(CalendarEvent::new(
                ts,
                Some(symbol.to_string()),
                EventKind::DividendPayment { amount: None },
            ));
        }
    }

    if let Some(opts) = options {
        let counts = opts.contract_counts();
        for ts in opts
            .expiration_dates()
            .into_iter()
            .filter(|&ts| in_window(ts) && is_monthly_expiration(ts))
        {
            let contract_count = counts.get(&ts).copied();
            events.push(CalendarEvent::new(
                ts,
                Some(symbol.to_string()),
                EventKind::OptionsExpiration { contract_count },
            ));
        }
    }

    events
}

/// Whether a timestamp falls on a standard monthly options expiration — the
/// third Friday of the month (a Friday with day-of-month in 15..=21).
fn is_monthly_expiration(timestamp: i64) -> bool {
    use chrono::{Datelike, Weekday};
    DateTime::<Utc>::from_timestamp(timestamp, 0).is_some_and(|dt| {
        let d = dt.date_naive();
        d.weekday() == Weekday::Fri && (15..=21).contains(&d.day())
    })
}

/// FRED release IDs for the major, market-moving US economic releases surfaced
/// in the calendar.
///
/// FRED's `releases/dates` feed lists ~300 releases, most of them niche or
/// sub-national (state retail sales, research indices, etc.). Restricting to
/// this curated set keeps the calendar focused on the releases that actually
/// move markets, instead of burying per-ticker events under hundreds of rows.
#[cfg(feature = "fred")]
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
#[cfg(feature = "fred")]
pub(crate) fn build_economic_events(
    releases: Vec<crate::adapters::fred::ReleaseDate>,
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
#[cfg(feature = "fred")]
fn drop_phantom_daily_fills(
    releases: Vec<crate::adapters::fred::ReleaseDate>,
) -> Vec<crate::adapters::fred::ReleaseDate> {
    use std::collections::{HashMap, HashSet};

    let day = |r: &crate::adapters::fred::ReleaseDate| {
        parse_iso_date(&r.date).map(|ts| ts.div_euclid(86_400))
    };

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
#[cfg(feature = "fred")]
fn parse_iso_date(date: &str) -> Option<i64> {
    use chrono::NaiveDate;
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)?
        .and_utc()
        .timestamp()
        .into()
}

/// Sort events ascending by timestamp in place.
pub(crate) fn sort_events(events: &mut [CalendarEvent]) {
    events.sort_by_key(|e| e.timestamp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_calendar() -> CalendarEvents {
        serde_json::from_value(json!({
            "maxAge": 1,
            "earnings": {
                "earningsDate": [{"fmt": "2026-02-25", "raw": 1_772_000_000_i64}],
                "earningsAverage": {"fmt": "1.52", "raw": 1.52},
                "earningsLow": {"fmt": "1.40", "raw": 1.40},
                "earningsHigh": {"fmt": "1.65", "raw": 1.65},
                "revenueAverage": {"fmt": "120B", "raw": 120_000_000_000_i64}
            },
            "exDividendDate": {"fmt": "2026-02-10", "raw": 1_770_700_000_i64},
            "dividendDate": {"fmt": "2026-02-20", "raw": 1_771_560_000_i64}
        }))
        .unwrap()
    }

    #[test]
    fn iso_date_formats_utc() {
        assert_eq!(iso_date(1_772_000_000), "2026-02-25");
    }

    #[test]
    fn builds_earnings_dividend_events_in_window() {
        let cal = sample_calendar();
        let window = (1_770_000_000, 1_773_000_000);
        let events = build_symbol_events("AAPL", Some(&cal), None, window);

        assert_eq!(events.len(), 3);
        let earnings = events
            .iter()
            .find(|e| matches!(e.event, EventKind::Earnings { .. }))
            .unwrap();
        assert_eq!(earnings.symbol.as_deref(), Some("AAPL"));
        match &earnings.event {
            EventKind::Earnings {
                eps_estimate_avg,
                revenue_estimate_avg,
                is_estimate,
                ..
            } => {
                assert_eq!(*eps_estimate_avg, Some(1.52));
                assert_eq!(*revenue_estimate_avg, Some(120_000_000_000));
                assert!(*is_estimate);
            }
            _ => unreachable!(),
        }
        assert!(
            events
                .iter()
                .any(|e| matches!(e.event, EventKind::ExDividend { .. }))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e.event, EventKind::DividendPayment { .. }))
        );
    }

    #[test]
    fn filters_events_outside_window() {
        let cal = sample_calendar();
        // Window entirely before all sample dates.
        let window = (1_000_000_000, 1_100_000_000);
        let events = build_symbol_events("AAPL", Some(&cal), None, window);
        assert!(events.is_empty());
    }

    #[test]
    fn options_expirations_filter_to_monthly_with_optional_counts() {
        // 2026-07-17 (3rd Fri, monthly) has a loaded chain (2 calls + 1 put);
        // 2026-08-21 (3rd Fri, monthly) is listed but not detailed;
        // 2026-07-24 (weekly Fri) must be filtered out entirely.
        const JUL_17: i64 = 1_784_246_400; // 2026-07-17 Fri
        const JUL_24: i64 = 1_784_851_200; // 2026-07-24 Fri (weekly)
        const AUG_21: i64 = 1_787_270_400; // 2026-08-21 Fri
        let opts: Options = serde_json::from_value(json!({
            "optionChain": {
                "result": [{
                    "underlyingSymbol": "AAPL",
                    "expirationDates": [JUL_17, JUL_24, AUG_21],
                    "strikes": [100.0, 105.0, 110.0],
                    "options": [{
                        "expirationDate": JUL_17,
                        "calls": [{"contractSymbol":"A","strike":100.0},{"contractSymbol":"B","strike":105.0}],
                        "puts": [{"contractSymbol":"C","strike":100.0}]
                    }]
                }],
                "error": null
            }
        }))
        .unwrap();

        let window = (1_783_000_000, 1_790_000_000);
        let mut events = build_symbol_events("AAPL", None, Some(&opts), window);
        sort_events(&mut events);

        // Weekly JUL_24 excluded → only the two monthlies remain.
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].timestamp, JUL_17);
        assert_eq!(events[1].timestamp, AUG_21);
        match events[0].event {
            EventKind::OptionsExpiration { contract_count } => assert_eq!(contract_count, Some(3)),
            _ => unreachable!(),
        }
        match events[1].event {
            EventKind::OptionsExpiration { contract_count } => assert_eq!(contract_count, None),
            _ => unreachable!(),
        }
    }

    #[test]
    fn sort_events_orders_ascending() {
        let mut events = vec![
            CalendarEvent::new(300, None, EventKind::ExDividend { amount: None }),
            CalendarEvent::new(100, None, EventKind::ExDividend { amount: None }),
            CalendarEvent::new(200, None, EventKind::ExDividend { amount: None }),
        ];
        sort_events(&mut events);
        assert_eq!(
            events.iter().map(|e| e.timestamp).collect::<Vec<_>>(),
            vec![100, 200, 300]
        );
    }

    #[test]
    fn serializes_event_kind_with_type_tag() {
        let event = CalendarEvent::new(
            1_772_000_000,
            Some("TSLA".to_string()),
            EventKind::OptionsExpiration {
                contract_count: Some(312),
            },
        );
        let v = serde_json::to_value(&event).unwrap();
        assert_eq!(v["event"]["type"], "options_expiration");
        assert_eq!(v["event"]["contract_count"], 312);
        assert_eq!(v["date"], "2026-02-25");
    }

    #[cfg(feature = "fred")]
    #[test]
    fn economic_events_filter_to_major_releases_in_window() {
        use crate::adapters::fred::ReleaseDate;
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

    #[cfg(feature = "fred")]
    #[test]
    fn drops_phantom_daily_fills_keeps_real_schedules() {
        use crate::adapters::fred::ReleaseDate;
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
