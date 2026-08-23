//! Locally computed NYSE/NASDAQ market holidays — no network call, no
//! external source. Federal-holiday observance rules are small and
//! well-known enough to hand-verify, so this reimplements the general
//! algorithm rather than depending on (or porting) a third-party calendar
//! library.
//!
//! Full closures: New Year's Day, MLK Day, Washington's Birthday, Good
//! Friday, Memorial Day, Juneteenth (from 2022), Independence Day, Labor
//! Day, Thanksgiving, Christmas — each shifted Saturday→Friday or
//! Sunday→Monday when it falls on a weekend. Early closes (1:00 PM ET): the
//! day after Thanksgiving always; Christmas Eve and July 3rd when they fall
//! on a weekday and the holiday they precede is observed on its actual date
//! (not shifted).

use chrono::{Datelike, Duration, NaiveDate, Weekday};

use super::{CalendarProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;
use crate::models::calendar::market::{CalendarDetail, CalendarKind, MarketCalendarEntry};

pub(crate) struct LocalMarketCalendarProvider;

impl ProviderCore for LocalMarketCalendarProvider {
    fn id(&self) -> super::Provider {
        super::Provider::LocalMarketCalendar
    }
}

#[async_trait::async_trait]
impl CalendarProvider for LocalMarketCalendarProvider {
    async fn fetch_market_calendar(
        &self,
        kind: CalendarKind,
        from: &str,
        to: &str,
    ) -> Result<Vec<MarketCalendarEntry>> {
        if kind != CalendarKind::MarketHoliday {
            return Err(self.not_supported(kind.operation()));
        }
        let start = NaiveDate::parse_from_str(from, "%Y-%m-%d").map_err(|_| {
            crate::error::FinanceError::InvalidParameter {
                param: "from".to_string(),
                reason: format!("expected YYYY-MM-DD, got {from:?}"),
            }
        })?;
        let end = NaiveDate::parse_from_str(to, "%Y-%m-%d").map_err(|_| {
            crate::error::FinanceError::InvalidParameter {
                param: "to".to_string(),
                reason: format!("expected YYYY-MM-DD, got {to:?}"),
            }
        })?;
        if start > end {
            return Err(crate::error::FinanceError::InvalidParameter {
                param: "to".to_string(),
                reason: "must not be before `from`".to_string(),
            });
        }
        Ok(holidays_in_range(start, end))
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for LocalMarketCalendarProvider {
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        Some(self)
    }
}

fn holidays_in_range(start: NaiveDate, end: NaiveDate) -> Vec<MarketCalendarEntry> {
    let mut entries: Vec<MarketCalendarEntry> =
        (start.year()..=end.year())
            .flat_map(|year| {
                let (full, early) = year_calendar(year);
                full.into_iter()
                    .map(|(date, name)| holiday_entry(date, name, "closed", None))
                    .chain(early.into_iter().map(|(date, name)| {
                        holiday_entry(date, name, "early-close", Some("13:00"))
                    }))
            })
            .filter(|entry| {
                entry
                    .date
                    .as_deref()
                    .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                    .is_some_and(|d| d >= start && d <= end)
            })
            .collect();
    entries.sort_by(|a, b| a.date.cmp(&b.date));
    entries
}

fn holiday_entry(
    date: NaiveDate,
    name: &'static str,
    status: &'static str,
    close: Option<&'static str>,
) -> MarketCalendarEntry {
    MarketCalendarEntry {
        symbol: None,
        date: Some(date.format("%Y-%m-%d").to_string()),
        detail: CalendarDetail::MarketHoliday {
            name: Some(name.to_string()),
            exchange: Some("NYSE".to_string()),
            status: Some(status.to_string()),
            open: None,
            close: close.map(str::to_string),
        },
    }
}

type Holidays = Vec<(NaiveDate, &'static str)>;

/// Full-closure and early-close holidays for one calendar year.
fn year_calendar(year: i32) -> (Holidays, Holidays) {
    let mut full = vec![
        (observed(ymd(year, 1, 1)), "New Year's Day"),
        (
            nth_weekday(year, 1, Weekday::Mon, 3),
            "Martin Luther King Jr. Day",
        ),
        (
            nth_weekday(year, 2, Weekday::Mon, 3),
            "Washington's Birthday",
        ),
        (good_friday(year), "Good Friday"),
        (last_weekday(year, 5, Weekday::Mon), "Memorial Day"),
        (observed(ymd(year, 7, 4)), "Independence Day"),
        (nth_weekday(year, 9, Weekday::Mon, 1), "Labor Day"),
        (nth_weekday(year, 11, Weekday::Thu, 4), "Thanksgiving Day"),
        (observed(ymd(year, 12, 25)), "Christmas Day"),
    ];
    if year >= 2022 {
        full.push((
            observed(ymd(year, 6, 19)),
            "Juneteenth National Independence Day",
        ));
    }

    let mut early = Vec::new();
    let thanksgiving = nth_weekday(year, 11, Weekday::Thu, 4);
    early.push((thanksgiving + Duration::days(1), "Day after Thanksgiving"));

    let july4 = ymd(year, 7, 4);
    if observed(july4) == july4 {
        let july3 = july4 - Duration::days(1);
        if is_weekday(july3) {
            early.push((july3, "Independence Day (early close)"));
        }
    }
    let christmas = ymd(year, 12, 25);
    if observed(christmas) == christmas {
        let christmas_eve = christmas - Duration::days(1);
        if is_weekday(christmas_eve) {
            early.push((christmas_eve, "Christmas Eve"));
        }
    }

    (full, early)
}

fn ymd(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid calendar date")
}

fn is_weekday(date: NaiveDate) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Shift a Saturday holiday to the preceding Friday and a Sunday holiday to
/// the following Monday; any other weekday is unchanged.
fn observed(date: NaiveDate) -> NaiveDate {
    match date.weekday() {
        Weekday::Sat => date - Duration::days(1),
        Weekday::Sun => date + Duration::days(1),
        _ => date,
    }
}

/// The `n`th occurrence of `weekday` in `month` (1-indexed, e.g. `n=3` for
/// the third Monday).
fn nth_weekday(year: i32, month: u32, weekday: Weekday, n: u32) -> NaiveDate {
    let first_of_month = ymd(year, month, 1);
    let offset = (7 + weekday.num_days_from_monday() as i64
        - first_of_month.weekday().num_days_from_monday() as i64)
        % 7;
    first_of_month + Duration::days(offset + 7 * (n as i64 - 1))
}

/// The last occurrence of `weekday` in `month`.
fn last_weekday(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let first_of_next = if month == 12 {
        ymd(year + 1, 1, 1)
    } else {
        ymd(year, month + 1, 1)
    };
    let last_of_month = first_of_next - Duration::days(1);
    let offset = (7 + last_of_month.weekday().num_days_from_monday() as i64
        - weekday.num_days_from_monday() as i64)
        % 7;
    last_of_month - Duration::days(offset)
}

/// Easter Sunday via the Anonymous Gregorian algorithm (Meeus/Jones/Butcher).
/// Good Friday is two days earlier.
fn good_friday(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    ymd(year, month as u32, day as u32) - Duration::days(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easter_matches_known_dates() {
        assert_eq!(good_friday(2026) + Duration::days(2), ymd(2026, 4, 5));
        assert_eq!(good_friday(2024) + Duration::days(2), ymd(2024, 3, 31));
        assert_eq!(good_friday(2025) + Duration::days(2), ymd(2025, 4, 20));
    }

    #[test]
    fn weekend_holidays_shift_to_the_adjacent_business_day() {
        // July 4, 2026 is a Saturday.
        assert_eq!(observed(ymd(2026, 7, 4)), ymd(2026, 7, 3));
        // December 25, 2027 is a Saturday.
        assert_eq!(observed(ymd(2027, 12, 25)), ymd(2027, 12, 24));
    }

    #[test]
    fn nth_weekday_finds_the_third_monday() {
        // January 2026: Jan 1 is a Thursday, so the third Monday is Jan 19.
        assert_eq!(nth_weekday(2026, 1, Weekday::Mon, 3), ymd(2026, 1, 19));
    }

    #[test]
    fn last_weekday_finds_the_last_monday_of_may() {
        // May 2026 has 31 days; May 31, 2026 is a Sunday, so the last Monday is May 25.
        assert_eq!(last_weekday(2026, 5, Weekday::Mon), ymd(2026, 5, 25));
    }

    #[test]
    fn juneteenth_is_not_a_holiday_before_2022() {
        let (full_2021, _) = year_calendar(2021);
        assert!(
            !full_2021
                .iter()
                .any(|(_, name)| *name == "Juneteenth National Independence Day")
        );
        let (full_2022, _) = year_calendar(2022);
        assert!(
            full_2022
                .iter()
                .any(|(_, name)| *name == "Juneteenth National Independence Day")
        );
    }

    #[test]
    fn a_saturday_christmas_gets_no_separate_christmas_eve_early_close() {
        // December 25, 2027 is a Saturday, observed Friday Dec 24 as a full closure.
        let (full, early) = year_calendar(2027);
        assert!(full.iter().any(|(d, _)| *d == ymd(2027, 12, 24)));
        assert!(!early.iter().any(|(_, name)| *name == "Christmas Eve"));
    }

    #[tokio::test]
    async fn range_query_returns_sorted_entries_within_bounds() {
        let provider = LocalMarketCalendarProvider;
        let entries = provider
            .fetch_market_calendar(CalendarKind::MarketHoliday, "2026-01-01", "2026-01-31")
            .await
            .unwrap();
        assert_eq!(entries.len(), 2, "New Year's Day + MLK Day");
        assert_eq!(entries[0].date.as_deref(), Some("2026-01-01"));
        assert_eq!(entries[1].date.as_deref(), Some("2026-01-19"));
        match &entries[0].detail {
            CalendarDetail::MarketHoliday { name, status, .. } => {
                assert_eq!(name.as_deref(), Some("New Year's Day"));
                assert_eq!(status.as_deref(), Some("closed"));
            }
            other => panic!("expected MarketHoliday detail, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn other_calendar_kinds_are_not_supported() {
        let provider = LocalMarketCalendarProvider;
        let err = provider
            .fetch_market_calendar(CalendarKind::Earnings, "2026-01-01", "2026-01-31")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            crate::error::FinanceError::NotSupported { .. }
        ));
    }
}
