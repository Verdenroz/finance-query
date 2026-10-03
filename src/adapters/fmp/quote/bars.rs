//! Resumable minute and daily stock-bar pages.
//!
//! FMP answers a minute request with only the last three calendar days of the
//! requested range, and a daily request with at most 5,000 rows, so each page
//! asks for one window small enough to come back whole and the cursor carries
//! the next window.

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, TimeZone, Weekday};
use chrono_tz::America::New_York;
use serde::{Deserialize, Serialize};

use crate::adapters::fmp::models::{HistoricalPriceDTO, IntradayPriceDTO};
use crate::error::FinanceError;
use crate::{PageCursor, Provider, ProviderPage, Result, SortType, StockBar, StockBarsRequest};

const MINUTE_WINDOW_DAYS: i64 = 3;
/// About 4,800 trading days, inside the 5,000-row cap on daily history.
const DAILY_WINDOW_DAYS: i64 = 7_000;
const DAILY_ROW_CAP: usize = 5_000;

#[derive(Serialize, Deserialize)]
struct WindowCursor {
    /// The edge of the next window: its first day ascending, its last day descending.
    next: String,
}

fn invalid_cursor() -> FinanceError {
    FinanceError::InvalidParameter {
        param: "cursor".into(),
        reason: "invalid FMP stock-bar continuation".into(),
    }
}

fn invalid_row(context: &str) -> FinanceError {
    FinanceError::ResponseStructureError {
        field: "bars".into(),
        context: format!("FMP returned {context}"),
    }
}

fn parse_day(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| FinanceError::InvalidParameter {
        param: "date".into(),
        reason: format!("expected YYYY-MM-DD, got {value}"),
    })
}

fn weekend(day: NaiveDate) -> bool {
    matches!(day.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Minute windows start and end on weekdays, so a week takes two requests.
fn skip_weekend(mut day: NaiveDate, step: i64) -> NaiveDate {
    while weekend(day) {
        day += Duration::days(step);
    }
    day
}

struct Window {
    first: NaiveDate,
    last: NaiveDate,
    next: Option<NaiveDate>,
}

fn window(
    edge: NaiveDate,
    from: NaiveDate,
    to: NaiveDate,
    days: i64,
    minute: bool,
    ascending: bool,
) -> Option<Window> {
    let span = Duration::days(days - 1);
    if ascending {
        let first = if minute { skip_weekend(edge, 1) } else { edge };
        if first > to {
            return None;
        }
        let last = (first + span).min(to);
        let next = Some(last + Duration::days(1)).filter(|d| *d <= to);
        Some(Window { first, last, next })
    } else {
        let last = if minute { skip_weekend(edge, -1) } else { edge };
        if last < from {
            return None;
        }
        let first = (last - span).max(from);
        let next = Some(first - Duration::days(1)).filter(|d| *d >= from);
        Some(Window { first, last, next })
    }
}

/// FMP reports minute bars in New York local time.
fn new_york_ms(local: NaiveDateTime) -> Option<i64> {
    New_York
        .from_local_datetime(&local)
        .earliest()
        .map(|t| t.timestamp_millis())
}

fn bar(
    timestamp_ms: i64,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    close: Option<f64>,
    volume: Option<f64>,
) -> Result<StockBar> {
    let (Some(open), Some(high), Some(low), Some(close)) = (open, high, low, close) else {
        return Err(invalid_row("a bar without prices"));
    };
    let volume = volume.unwrap_or(0.0);
    if [open, high, low, close, volume]
        .iter()
        .any(|v| !v.is_finite())
        || volume < 0.0
    {
        return Err(invalid_row("a bar with non-finite values"));
    }
    Ok(StockBar {
        timestamp_ms,
        open,
        high,
        low,
        close,
        volume,
        transactions: None,
        vwap: None,
    })
}

fn minute_bars(
    rows: Vec<IntradayPriceDTO>,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<StockBar>> {
    rows.into_iter()
        .map(|row| {
            let local = row
                .date
                .as_deref()
                .and_then(|d| NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M:%S").ok())
                .filter(|d| (first..=last).contains(&d.date()))
                .ok_or_else(|| invalid_row("a minute outside the requested window"))?;
            let timestamp =
                new_york_ms(local).ok_or_else(|| invalid_row("an invalid local time"))?;
            bar(
                timestamp, row.open, row.high, row.low, row.close, row.volume,
            )
        })
        .collect()
}

/// Daily bars are stamped at midnight New York time, as Polygon stamps them.
fn daily_bars(
    rows: Vec<HistoricalPriceDTO>,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<StockBar>> {
    if rows.len() >= DAILY_ROW_CAP {
        return Err(invalid_row("a truncated daily history"));
    }
    rows.into_iter()
        .map(|row| {
            let day = row
                .date
                .as_deref()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                .filter(|d| (first..=last).contains(d))
                .ok_or_else(|| invalid_row("a day outside the requested window"))?;
            let midnight = day
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| invalid_row("an invalid day"))?;
            let timestamp = new_york_ms(midnight).ok_or_else(|| invalid_row("an invalid day"))?;
            bar(
                timestamp, row.open, row.high, row.low, row.close, row.volume,
            )
        })
        .collect()
}

pub(crate) async fn fetch_stock_bars_page(
    request: &StockBarsRequest,
    cursor: Option<&PageCursor>,
) -> Result<ProviderPage<StockBar>> {
    if !request.split_adjusted() {
        return Err(FinanceError::InvalidParameter {
            param: "adjustment".into(),
            reason: "FMP serves only split-adjusted stock bars".into(),
        });
    }
    let from = parse_day(&request.from)?;
    let to = parse_day(&request.to)?;
    let ascending = request.sort == SortType::Asc;
    let edge = match cursor {
        Some(cursor) => {
            let state: WindowCursor =
                serde_json::from_str(cursor.target()).map_err(|_| invalid_cursor())?;
            let edge = parse_day(&state.next).map_err(|_| invalid_cursor())?;
            if !(from..=to).contains(&edge) {
                return Err(invalid_cursor());
            }
            edge
        }
        None if ascending => from,
        None => to,
    };
    let minute = request.timespan == "minute";
    let days = if minute {
        MINUTE_WINDOW_DAYS
    } else {
        DAILY_WINDOW_DAYS
    };
    let Some(window) = window(edge, from, to, days, minute, ascending) else {
        return Ok(page(Vec::new(), None, &request.symbol));
    };
    let client = crate::adapters::fmp::build_client()?;
    let first = window.first.to_string();
    let last = window.last.to_string();
    let params = [
        ("symbol", request.symbol.as_str()),
        ("from", first.as_str()),
        ("to", last.as_str()),
    ];
    let mut items = if minute {
        let rows: Vec<IntradayPriceDTO> =
            client.get("/stable/historical-chart/1min", &params).await?;
        minute_bars(rows, window.first, window.last)?
    } else {
        let rows: Vec<HistoricalPriceDTO> = client
            .get("/stable/historical-price-eod/full", &params)
            .await?;
        daily_bars(rows, window.first, window.last)?
    };
    items.sort_by_key(|b| b.timestamp_ms);
    if !ascending {
        items.reverse();
    }
    let next = window
        .next
        .map(|next| {
            serde_json::to_string(&WindowCursor {
                next: next.to_string(),
            })
            .map(|target| PageCursor::continuation(Provider::Fmp, target))
        })
        .transpose()?;
    Ok(page(items, next, &request.symbol))
}

fn page(items: Vec<StockBar>, next: Option<PageCursor>, symbol: &str) -> ProviderPage<StockBar> {
    ProviderPage {
        results_count: Some(items.len()),
        items,
        next,
        provider_id: Provider::Fmp,
        request_id: None,
        query_count: None,
        reported_symbol: Some(symbol.to_string()),
        adjusted: Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn minute_windows_take_two_requests_a_week() {
        let (from, to) = (day("2025-06-02"), day("2025-06-13"));
        let mut edge = Some(from);
        let mut windows = Vec::new();
        while let Some(current) = edge {
            let w = window(current, from, to, MINUTE_WINDOW_DAYS, true, true).unwrap();
            windows.push((w.first.to_string(), w.last.to_string()));
            edge = w.next;
        }
        assert_eq!(
            windows,
            [
                ("2025-06-02", "2025-06-04"),
                ("2025-06-05", "2025-06-07"),
                ("2025-06-09", "2025-06-11"),
                ("2025-06-12", "2025-06-13"),
            ]
            .map(|(a, b)| (a.to_string(), b.to_string()))
        );
    }

    #[test]
    fn descending_minute_windows_walk_back_from_the_end() {
        let (from, to) = (day("2025-06-02"), day("2025-06-08"));
        let first = window(to, from, to, MINUTE_WINDOW_DAYS, true, false).unwrap();
        assert_eq!(
            (first.first, first.last),
            (day("2025-06-04"), day("2025-06-06"))
        );
        let second = window(
            first.next.unwrap(),
            from,
            to,
            MINUTE_WINDOW_DAYS,
            true,
            false,
        )
        .unwrap();
        assert_eq!(
            (second.first, second.last),
            (day("2025-06-02"), day("2025-06-03"))
        );
        assert_eq!(second.next, None);
    }

    #[test]
    fn a_weekend_only_range_has_no_minute_window() {
        let saturday = day("2025-06-07");
        assert!(window(saturday, saturday, day("2025-06-08"), 3, true, true).is_none());
    }

    #[test]
    fn new_york_times_become_utc_across_daylight_saving() {
        let summer =
            NaiveDateTime::parse_from_str("2025-06-02 09:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let winter =
            NaiveDateTime::parse_from_str("2025-01-02 09:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(new_york_ms(summer), Some(1_748_871_000_000));
        assert_eq!(new_york_ms(winter), Some(1_735_828_200_000));
    }
}
