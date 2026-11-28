//! Financial event calendar models.
//!
//! A [`CalendarEvent`] is a single upcoming financial event — earnings,
//! ex-dividend/dividend-payment, options expiration, or (with the `fred`
//! feature) a market-wide economic-data release.
//!
//! Construct calendars via `Ticker::calendar` and
//! `Tickers::calendar`.

/// Market-wide calendars routed through `Capability::CALENDAR`.
pub mod market;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::options::Options;
use crate::quote::{CalendarEvents, DefaultKeyStatistics};

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
        /// Most recent declared dividend per share. The upcoming amount may differ.
        amount: Option<f64>,
    },
    /// Dividend payment date — cash arrives in the account.
    DividendPayment {
        /// Most recent declared dividend per share. The upcoming amount may differ.
        amount: Option<f64>,
    },
    /// Standard monthly options expiration (3rd Friday) for this ticker.
    ///
    /// Only standard monthly expirations are surfaced — daily and weekly
    /// expirations are omitted to keep the calendar focused on the dates that
    /// carry meaningful open interest.
    OptionsExpiration {
        /// Number of listed contracts (calls + puts) expiring on this date.
        /// Loaded for monthly expirations inside the requested window only;
        /// `None` when that chain could not be fetched.
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
    #[doc(hidden)]
    pub fn new(timestamp: i64, symbol: Option<String>, event: EventKind) -> Self {
        Self {
            timestamp,
            date: iso_date(timestamp),
            symbol,
            event,
        }
    }
}

#[cfg(feature = "dataframe")]
impl EventKind {
    fn type_name(&self) -> &'static str {
        match self {
            Self::Earnings { .. } => "earnings",
            Self::ExDividend { .. } => "ex_dividend",
            Self::DividendPayment { .. } => "dividend_payment",
            Self::OptionsExpiration { .. } => "options_expiration",
            #[cfg(feature = "fred")]
            Self::EconomicRelease { .. } => "economic_release",
        }
    }
}

#[cfg(feature = "dataframe")]
impl CalendarEvent {
    /// Converts the events to a polars DataFrame, one row per event.
    ///
    /// Columns are `timestamp` (`Datetime(us, UTC)`), `date` (`Date`), `symbol`
    /// and `event_type` (strings). The event payload is not expanded because
    /// its fields differ per type.
    pub fn vec_to_dataframe(
        events: &[Self],
    ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use crate::__dataframe::{EpochUnit, epoch_datetime, str_date};
        use polars::prelude::*;

        DataFrame::new(
            events.len(),
            vec![
                epoch_datetime(
                    "timestamp".into(),
                    EpochUnit::Seconds,
                    false,
                    events.iter().map(|e| Some(e.timestamp)),
                ),
                str_date("date".into(), events.iter().map(|e| Some(e.date.as_str()))),
                Series::new(
                    "symbol".into(),
                    events
                        .iter()
                        .map(|e| e.symbol.as_deref())
                        .collect::<Vec<_>>(),
                )
                .into(),
                Series::new(
                    "event_type".into(),
                    events
                        .iter()
                        .map(|e| e.event.type_name())
                        .collect::<Vec<_>>(),
                )
                .into(),
            ],
        )
    }

    /// Converts this event to a one-row polars DataFrame; see [`vec_to_dataframe`](Self::vec_to_dataframe).
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        Self::vec_to_dataframe(std::slice::from_ref(self))
    }
}

/// Format a Unix-second timestamp as an ISO `YYYY-MM-DD` UTC date string.
#[doc(hidden)]
pub fn iso_date(timestamp: i64) -> String {
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Most recent declared per-share dividend from the quote's key statistics.
#[doc(hidden)]
pub fn last_dividend_value(stats: &DefaultKeyStatistics) -> Option<f64> {
    stats.last_dividend_value.as_ref()?.raw.filter(|&v| v > 0.0)
}

/// Monthly expirations inside `window` whose chain is not yet loaded in `options`.
#[doc(hidden)]
pub fn uncounted_monthly_expirations(options: &Options, window: (i64, i64)) -> Vec<i64> {
    let counts = options.contract_counts();
    options
        .expiration_dates()
        .into_iter()
        .filter(|&ts| {
            ts >= window.0
                && ts <= window.1
                && is_monthly_expiration(ts)
                && !counts.contains_key(&ts)
        })
        .collect()
}

/// Build the per-symbol calendar events from already-fetched data.
///
/// Pure and synchronous: callers fetch `calendar` (the `calendarEvents` quote
/// module) and `options` once, then hand them here. `dividend_amount` is the
/// most recent declared per-share dividend, reported for the upcoming ex-dividend and
/// payment dates because Yahoo publishes no forward amount. `extra_chains` are
/// chains fetched for further expirations, merged into the contract counts. Only events whose timestamp
/// falls within `window` (`[start, end]`, inclusive) are emitted. The result is
/// **not** sorted — callers merge across symbols and sort once.
#[doc(hidden)]
pub fn build_symbol_events(
    symbol: &str,
    calendar: Option<&CalendarEvents>,
    options: Option<&Options>,
    extra_chains: &[Options],
    dividend_amount: Option<f64>,
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
                EventKind::ExDividend {
                    amount: dividend_amount,
                },
            ));
        }

        if let Some(ts) = cal.dividend_timestamp().filter(|&ts| in_window(ts)) {
            events.push(CalendarEvent::new(
                ts,
                Some(symbol.to_string()),
                EventKind::DividendPayment {
                    amount: dividend_amount,
                },
            ));
        }
    }

    if let Some(opts) = options {
        let mut counts = opts.contract_counts();
        counts.extend(extra_chains.iter().flat_map(Options::contract_counts));
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

/// Sort events ascending by timestamp in place.
#[doc(hidden)]
pub fn sort_events(events: &mut [CalendarEvent]) {
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
        let events = build_symbol_events("AAPL", Some(&cal), None, &[], None, window);

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
    fn dividend_events_carry_last_dividend_amount() {
        let cal = sample_calendar();
        let window = (1_770_000_000, 1_773_000_000);
        let events = build_symbol_events("AAPL", Some(&cal), None, &[], Some(0.25), window);

        let amounts: Vec<_> = events
            .iter()
            .filter_map(|e| match e.event {
                EventKind::ExDividend { amount } | EventKind::DividendPayment { amount } => amount,
                _ => None,
            })
            .collect();
        assert_eq!(amounts, vec![0.25, 0.25]);
    }

    #[test]
    fn last_dividend_value_ignores_zero() {
        let stats: DefaultKeyStatistics = serde_json::from_value(json!({
            "lastDividendValue": {"fmt": "0.00", "raw": 0.0}
        }))
        .unwrap();
        assert_eq!(last_dividend_value(&stats), None);
    }

    #[test]
    fn last_dividend_value_reads_raw() {
        let stats: DefaultKeyStatistics = serde_json::from_value(json!({
            "lastDividendValue": {"fmt": "0.25", "raw": 0.25}
        }))
        .unwrap();
        assert_eq!(last_dividend_value(&stats), Some(0.25));
    }

    #[test]
    fn filters_events_outside_window() {
        let cal = sample_calendar();
        // Window entirely before all sample dates.
        let window = (1_000_000_000, 1_100_000_000);
        let events = build_symbol_events("AAPL", Some(&cal), None, &[], None, window);
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
        let mut events = build_symbol_events("AAPL", None, Some(&opts), &[], None, window);
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
    fn extra_chains_fill_other_monthly_contract_counts() {
        const JUL_17: i64 = 1_784_246_400;
        const AUG_21: i64 = 1_787_270_400;
        let chain = |ts: i64, calls: usize| -> Options {
            let calls: Vec<_> = (0..calls)
                .map(|i| json!({"contractSymbol": format!("C{i}"), "strike": 100.0}))
                .collect();
            serde_json::from_value(json!({
                "optionChain": {
                    "result": [{
                        "underlyingSymbol": "AAPL",
                        "expirationDates": [JUL_17, AUG_21],
                        "strikes": [100.0],
                        "options": [{"expirationDate": ts, "calls": calls, "puts": []}]
                    }],
                    "error": null
                }
            }))
            .unwrap()
        };
        let nearest = chain(JUL_17, 2);
        let window = (1_783_000_000, 1_790_000_000);
        assert_eq!(
            uncounted_monthly_expirations(&nearest, window),
            vec![AUG_21]
        );

        let mut events = build_symbol_events(
            "AAPL",
            None,
            Some(&nearest),
            &[chain(AUG_21, 5)],
            None,
            window,
        );
        sort_events(&mut events);
        let counts: Vec<_> = events
            .iter()
            .map(|e| match e.event {
                EventKind::OptionsExpiration { contract_count } => contract_count,
                _ => None,
            })
            .collect();
        assert_eq!(counts, vec![Some(2), Some(5)]);
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
}

#[cfg(all(test, feature = "dataframe"))]
mod dataframe_tests;
