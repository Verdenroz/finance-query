//! `[from, to]` range iteration and DTO → canonical mapping for Nasdaq's
//! calendar endpoints.
//!
//! Nasdaq takes no date-range parameter: earnings/dividends/splits are
//! queried one calendar day at a time, IPOs one calendar month at a time. A
//! range request fans that out into one HTTP call per day/month, dropping
//! individual failures (logged) rather than failing the whole range — the
//! same fan-out-and-drop pattern `YahooProvider::fetch_sector_performance`
//! uses for its per-sector requests.

use chrono::{Datelike, NaiveDate};

use super::models::{NasdaqDividendRow, NasdaqEarningsRow, NasdaqIpoRow, NasdaqSplitRow};
use crate::error::{FinanceError, Result};
use crate::models::calendar::market::{CalendarDetail, CalendarKind, MarketCalendarEntry};

/// Earnings/dividends/splits are fetched one day at a time; cap the range so
/// a broad request doesn't fan out into hundreds of HTTP calls.
const MAX_DAYS_PER_RANGE: i64 = 92;

/// IPOs are fetched one month at a time.
const MAX_MONTHS_PER_RANGE: i64 = 13;

pub(crate) async fn fetch_market_calendar_response(
    kind: CalendarKind,
    from: &str,
    to: &str,
) -> Result<Vec<MarketCalendarEntry>> {
    match kind {
        CalendarKind::Earnings => {
            day_range(from, to, MAX_DAYS_PER_RANGE, |date| async move {
                let rows = super::client()?.earnings_for_date(&date).await?;
                Ok(rows.into_iter().map(|r| earnings_entry(&date, r)).collect())
            })
            .await
        }
        CalendarKind::Dividend => {
            day_range(from, to, MAX_DAYS_PER_RANGE, |date| async move {
                let rows = super::client()?.dividends_for_date(&date).await?;
                Ok(rows.into_iter().map(dividend_entry).collect())
            })
            .await
        }
        CalendarKind::Split => {
            day_range(from, to, MAX_DAYS_PER_RANGE, |date| async move {
                let rows = super::client()?.splits_for_date(&date).await?;
                Ok(rows.into_iter().map(split_entry).collect())
            })
            .await
        }
        CalendarKind::Ipo => ipo_calendar(from, to).await,
        _ => Err(kind.operation().not_supported(crate::Provider::Nasdaq)),
    }
}

async fn ipo_calendar(from: &str, to: &str) -> Result<Vec<MarketCalendarEntry>> {
    let (start, end) = parse_range(from, to)?;
    let months = months_between(start, end, MAX_MONTHS_PER_RANGE)?;

    let fetches = months.into_iter().map(|month| async move {
        let client = match super::client() {
            Ok(client) => client,
            Err(err) => {
                tracing::warn!("failed to build Nasdaq client: {err}");
                return Vec::new();
            }
        };
        match client.ipos_for_month(&month).await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::warn!("failed to fetch Nasdaq IPO calendar for {month}: {err}");
                Vec::new()
            }
        }
    });

    let entries: Vec<MarketCalendarEntry> = futures::future::join_all(fetches)
        .await
        .into_iter()
        .flatten()
        .map(|(action, row)| ipo_entry(action, row))
        .filter(|entry| {
            entry
                .date
                .as_deref()
                .is_some_and(|d| date_in_range(d, start, end))
        })
        .collect();
    Ok(entries)
}

async fn day_range<F, Fut>(
    from: &str,
    to: &str,
    max_days: i64,
    fetch_day: F,
) -> Result<Vec<MarketCalendarEntry>>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<MarketCalendarEntry>>>,
{
    let (start, end) = parse_range(from, to)?;
    if (end - start).num_days() + 1 > max_days {
        return Err(FinanceError::InvalidParameter {
            param: "to".to_string(),
            reason: format!(
                "Nasdaq calendars are queried one day at a time; a range over {max_days} days is too broad"
            ),
        });
    }

    let days: Vec<String> = start
        .iter_days()
        .take_while(|d| *d <= end)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .collect();

    let fetches = days.into_iter().map(|date| {
        let fetch_day = &fetch_day;
        async move {
            match fetch_day(date.clone()).await {
                Ok(entries) => entries,
                Err(err) => {
                    tracing::warn!("failed to fetch Nasdaq calendar for {date}: {err}");
                    Vec::new()
                }
            }
        }
    });

    Ok(futures::future::join_all(fetches)
        .await
        .into_iter()
        .flatten()
        .collect())
}

fn parse_range(from: &str, to: &str) -> Result<(NaiveDate, NaiveDate)> {
    let start = NaiveDate::parse_from_str(from, "%Y-%m-%d").map_err(|_| {
        FinanceError::InvalidParameter {
            param: "from".to_string(),
            reason: format!("expected YYYY-MM-DD, got {from:?}"),
        }
    })?;
    let end =
        NaiveDate::parse_from_str(to, "%Y-%m-%d").map_err(|_| FinanceError::InvalidParameter {
            param: "to".to_string(),
            reason: format!("expected YYYY-MM-DD, got {to:?}"),
        })?;
    if start > end {
        return Err(FinanceError::InvalidParameter {
            param: "to".to_string(),
            reason: "must not be before `from`".to_string(),
        });
    }
    Ok((start, end))
}

fn months_between(start: NaiveDate, end: NaiveDate, max_months: i64) -> Result<Vec<String>> {
    let months = (end.year() as i64 - start.year() as i64) * 12
        + (end.month() as i64 - start.month() as i64)
        + 1;
    if months > max_months {
        return Err(FinanceError::InvalidParameter {
            param: "to".to_string(),
            reason: format!(
                "Nasdaq's IPO calendar is queried one month at a time; a range over {max_months} months is too broad"
            ),
        });
    }

    let mut out = Vec::new();
    let mut year = start.year();
    let mut month = start.month();
    for _ in 0..months {
        out.push(format!("{year:04}-{month:02}"));
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }
    Ok(out)
}

fn date_in_range(date: &str, start: NaiveDate, end: NaiveDate) -> bool {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok_and(|d| d >= start && d <= end)
}

/// Parse Nasdaq's `M/D/YYYY` dates (inconsistently zero-padded) to `YYYY-MM-DD`.
fn parse_mdy(s: &str) -> Option<String> {
    let mut parts = s.trim().split('/');
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    let year: i32 = parts.next()?.parse().ok()?;
    NaiveDate::from_ymd_opt(year, month, day).map(|d| d.format("%Y-%m-%d").to_string())
}

/// Parse Nasdaq's `Mon/YYYY` fiscal-quarter label to the last calendar day
/// of that month — the standard approximation for a quarter-end date when
/// only month/year precision is published.
fn parse_month_year_to_last_day(s: &str) -> Option<String> {
    let (month_str, year_str) = s.trim().split_once('/')?;
    let year: i32 = year_str.parse().ok()?;
    let month = match month_str.to_ascii_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };
    let first_of_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    first_of_next
        .pred_opt()
        .map(|d| d.format("%Y-%m-%d").to_string())
}

/// Parse a `$1,234,567.89`-style Nasdaq number into its raw value.
fn parse_currency(s: &str) -> Option<f64> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    if cleaned.is_empty() {
        None
    } else {
        cleaned.parse().ok()
    }
}

/// Parse Nasdaq's `"4 : 1"` / `"3:1"` split ratio into `(numerator, denominator)`.
fn parse_ratio(s: &str) -> Option<(f64, f64)> {
    let (num, den) = s.split_once(':')?;
    Some((num.trim().parse().ok()?, den.trim().parse().ok()?))
}

fn earnings_entry(date: &str, row: NasdaqEarningsRow) -> MarketCalendarEntry {
    let time = match row.time.as_deref() {
        Some("time-pre-market") => Some("bmo".to_string()),
        Some("time-after-hours") => Some("amc".to_string()),
        _ => None,
    };
    MarketCalendarEntry {
        symbol: row.symbol,
        date: Some(date.to_string()),
        detail: CalendarDetail::Earnings {
            eps: row.eps.as_deref().and_then(parse_currency),
            eps_estimated: row.eps_forecast.as_deref().and_then(parse_currency),
            revenue: None,
            revenue_estimated: None,
            fiscal_date_ending: row
                .fiscal_quarter_ending
                .as_deref()
                .and_then(parse_month_year_to_last_day),
            time,
        },
    }
}

fn dividend_entry(row: NasdaqDividendRow) -> MarketCalendarEntry {
    MarketCalendarEntry {
        symbol: row.symbol,
        date: row.ex_date.as_deref().and_then(parse_mdy),
        detail: CalendarDetail::Dividend {
            dividend: row.dividend_rate,
            adj_dividend: None,
            record_date: row.record_date.as_deref().and_then(parse_mdy),
            payment_date: row.payment_date.as_deref().and_then(parse_mdy),
            declaration_date: row.announcement_date.as_deref().and_then(parse_mdy),
        },
    }
}

fn split_entry(row: NasdaqSplitRow) -> MarketCalendarEntry {
    let (numerator, denominator) = row
        .ratio
        .as_deref()
        .and_then(parse_ratio)
        .map(|(n, d)| (Some(n), Some(d)))
        .unwrap_or((None, None));
    MarketCalendarEntry {
        symbol: row.symbol,
        date: row.execution_date.as_deref().and_then(parse_mdy),
        detail: CalendarDetail::Split {
            numerator,
            denominator,
        },
    }
}

fn ipo_entry(action: &str, row: NasdaqIpoRow) -> MarketCalendarEntry {
    let date = row
        .priced_date
        .as_deref()
        .or(row.expected_price_date.as_deref())
        .or(row.withdraw_date.as_deref())
        .or(row.filed_date.as_deref())
        .and_then(parse_mdy);
    MarketCalendarEntry {
        symbol: row.proposed_ticker_symbol,
        date,
        detail: CalendarDetail::Ipo {
            company: row.company_name,
            exchange: row.proposed_exchange,
            actions: Some(action.to_string()),
            shares: row.shares_offered.as_deref().and_then(parse_currency),
            price_range: row.proposed_share_price,
            market_cap: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mdy_dates_parse_regardless_of_padding() {
        assert_eq!(parse_mdy("8/25/2026"), Some("2026-08-25".to_string()));
        assert_eq!(parse_mdy("9/08/2026"), Some("2026-09-08".to_string()));
        assert_eq!(parse_mdy("not-a-date"), None);
    }

    #[test]
    fn fiscal_quarter_labels_map_to_month_end() {
        assert_eq!(
            parse_month_year_to_last_day("Jul/2026"),
            Some("2026-07-31".to_string())
        );
        assert_eq!(
            parse_month_year_to_last_day("Feb/2024"),
            Some("2024-02-29".to_string()),
            "leap year"
        );
        assert_eq!(
            parse_month_year_to_last_day("Dec/2026"),
            Some("2026-12-31".to_string()),
            "year rollover"
        );
        assert_eq!(parse_month_year_to_last_day("garbage"), None);
    }

    #[test]
    fn currency_strings_strip_symbol_and_commas() {
        assert_eq!(parse_currency("$121,887,293,613"), Some(121_887_293_613.0));
        assert_eq!(parse_currency("$2.72"), Some(2.72));
        assert_eq!(parse_currency(""), None);
    }

    #[test]
    fn ratios_parse_with_or_without_spacing() {
        assert_eq!(parse_ratio("4 : 1"), Some((4.0, 1.0)));
        assert_eq!(parse_ratio("3:1"), Some((3.0, 1.0)));
        assert_eq!(parse_ratio("garbage"), None);
    }

    #[tokio::test]
    async fn a_range_over_the_day_cap_is_rejected() {
        let err = day_range("2026-01-01", "2026-12-31", MAX_DAYS_PER_RANGE, |_| async {
            Ok(Vec::new())
        })
        .await
        .unwrap_err();
        assert!(matches!(err, FinanceError::InvalidParameter { .. }));
    }

    #[test]
    fn to_before_from_is_rejected() {
        let err = parse_range("2026-08-25", "2026-08-01").unwrap_err();
        assert!(matches!(err, FinanceError::InvalidParameter { .. }));
    }

    #[test]
    fn months_between_spans_year_boundaries() {
        let start = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2027, 1, 10).unwrap();
        let months = months_between(start, end, MAX_MONTHS_PER_RANGE).unwrap();
        assert_eq!(months, vec!["2026-11", "2026-12", "2027-01"]);
    }

    #[test]
    fn earnings_row_maps_time_and_fiscal_quarter() {
        let row: NasdaqEarningsRow = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "time": "time-after-hours",
            "fiscalQuarterEnding": "Jun/2026",
            "epsForecast": "$1.50",
            "eps": "$1.62"
        }))
        .unwrap();
        let entry = earnings_entry("2026-07-30", row);
        assert_eq!(entry.symbol.as_deref(), Some("AAPL"));
        assert_eq!(entry.date.as_deref(), Some("2026-07-30"));
        match entry.detail {
            CalendarDetail::Earnings {
                eps,
                eps_estimated,
                fiscal_date_ending,
                time,
                ..
            } => {
                assert_eq!(eps, Some(1.62));
                assert_eq!(eps_estimated, Some(1.50));
                assert_eq!(fiscal_date_ending.as_deref(), Some("2026-06-30"));
                assert_eq!(time.as_deref(), Some("amc"));
            }
            other => panic!("expected Earnings detail, got {other:?}"),
        }
    }

    #[test]
    fn dividend_row_maps_rate_and_dates() {
        let row: NasdaqDividendRow = serde_json::from_value(serde_json::json!({
            "symbol": "CDW",
            "dividend_Ex_Date": "8/25/2026",
            "payment_Date": "9/10/2026",
            "record_Date": "8/25/2026",
            "dividend_Rate": 0.63,
            "announcement_Date": "8/22/2026"
        }))
        .unwrap();
        let entry = dividend_entry(row);
        assert_eq!(entry.symbol.as_deref(), Some("CDW"));
        assert_eq!(entry.date.as_deref(), Some("2026-08-25"));
        match entry.detail {
            CalendarDetail::Dividend {
                dividend,
                payment_date,
                declaration_date,
                ..
            } => {
                assert_eq!(dividend, Some(0.63));
                assert_eq!(payment_date.as_deref(), Some("2026-09-10"));
                assert_eq!(declaration_date.as_deref(), Some("2026-08-22"));
            }
            other => panic!("expected Dividend detail, got {other:?}"),
        }
    }

    #[test]
    fn split_row_parses_ratio() {
        let row: NasdaqSplitRow = serde_json::from_value(serde_json::json!({
            "symbol": "APH",
            "ratio": "2 : 1",
            "executionDate": "9/08/2026"
        }))
        .unwrap();
        let entry = split_entry(row);
        assert_eq!(entry.date.as_deref(), Some("2026-09-08"));
        match entry.detail {
            CalendarDetail::Split {
                numerator,
                denominator,
            } => {
                assert_eq!(numerator, Some(2.0));
                assert_eq!(denominator, Some(1.0));
            }
            other => panic!("expected Split detail, got {other:?}"),
        }
    }

    #[test]
    fn ipo_row_prefers_priced_date_and_labels_the_action() {
        let row: NasdaqIpoRow = serde_json::from_value(serde_json::json!({
            "proposedTickerSymbol": "LYNX",
            "companyName": "Lyntris Inc.",
            "proposedExchange": "NYSE",
            "proposedSharePrice": "17.50",
            "sharesOffered": "17,000,000",
            "pricedDate": "8/19/2026"
        }))
        .unwrap();
        let entry = ipo_entry("priced", row);
        assert_eq!(entry.symbol.as_deref(), Some("LYNX"));
        assert_eq!(entry.date.as_deref(), Some("2026-08-19"));
        match entry.detail {
            CalendarDetail::Ipo {
                company,
                actions,
                shares,
                price_range,
                ..
            } => {
                assert_eq!(company.as_deref(), Some("Lyntris Inc."));
                assert_eq!(actions.as_deref(), Some("priced"));
                assert_eq!(shares, Some(17_000_000.0));
                assert_eq!(price_range.as_deref(), Some("17.50"));
            }
            other => panic!("expected Ipo detail, got {other:?}"),
        }
    }
}
