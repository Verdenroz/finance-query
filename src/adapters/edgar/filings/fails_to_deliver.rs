//! SEC fails-to-deliver flat files (`sec.gov/data/foiadocsfailsdatahtm`).
//!
//! SEC publishes consolidated fails-to-deliver data as bi-monthly pipe-delimited
//! text archives, one ZIP per settlement half-month (`cnsfails{YYYYMM}{a|b}.zip`,
//! `a` = 1st-15th, `b` = 16th-end). There is no per-symbol query endpoint, so each
//! period's archive is downloaded whole and filtered locally. The archive for the
//! most recent half-month is typically published with a few weeks' lag, so lookup
//! walks backward from the current period until it collects enough periods.

use std::io::{Cursor, Read};

use chrono::{Datelike, NaiveDate, Utc};

use crate::adapters::edgar::build_client;
use crate::error::{FinanceError, Result};
use crate::models::filings::FailToDeliver;

const SEC_WWW_BASE: &str = "https://www.sec.gov";

/// Bi-monthly periods to collect. Three periods cover roughly six weeks.
const LOOKBACK_PERIODS: usize = 3;

/// Caps walk-back so an unpublished current period doesn't loop forever.
const MAX_ATTEMPTS: usize = 8;

/// Descending bi-monthly period ids (`YYYYMMa`/`YYYYMMb`) starting at `from`.
fn periods_desc(from: NaiveDate) -> impl Iterator<Item = String> {
    let mut year = from.year();
    let mut month = from.month();
    let mut half = if from.day() <= 15 { 'a' } else { 'b' };
    std::iter::from_fn(move || {
        let period = format!("{year:04}{month:02}{half}");
        if half == 'b' {
            half = 'a';
        } else {
            half = 'b';
            month -= 1;
            if month == 0 {
                month = 12;
                year -= 1;
            }
        }
        Some(period)
    })
}

fn zip_url(base: &str, period: &str) -> String {
    format!("{base}/files/data/fails-deliver-data/cnsfails{period}.zip")
}

/// `SETTLEMENT DATE` (`YYYYMMDD`) to `YYYY-MM-DD`; malformed values pass through as `None`.
fn format_settlement_date(raw: &str) -> Option<String> {
    (raw.len() == 8).then(|| format!("{}-{}-{}", &raw[0..4], &raw[4..6], &raw[6..8]))
}

/// Parse one period's `SETTLEMENT DATE|CUSIP|SYMBOL|QUANTITY (FAILS)|DESCRIPTION|PRICE`
/// rows, keeping only those matching `symbol` (already uppercased).
fn parse_rows(text: &str, symbol: &str) -> Vec<FailToDeliver> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut cols = line.split('|');
            let date = cols.next()?;
            let _cusip = cols.next()?;
            let sym = cols.next()?;
            if sym != symbol {
                return None;
            }
            let quantity = cols.next()?;
            let description = cols.next()?;
            let price = cols.next()?;
            let name = description.trim();
            Some(FailToDeliver {
                symbol: Some(sym.to_string()),
                date: format_settlement_date(date),
                quantity: quantity.trim().parse().ok(),
                price: price.trim().parse().ok(),
                name: (!name.is_empty()).then(|| name.to_string()),
                description: None,
            })
        })
        .collect()
}

fn parse_period(bytes: &[u8], symbol: &str) -> Result<Vec<FailToDeliver>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| {
        FinanceError::ResponseStructureError {
            field: "secftd.zip".to_string(),
            context: format!("failed to open fails-to-deliver archive: {e}"),
        }
    })?;
    let mut entry = archive
        .by_index(0)
        .map_err(|e| FinanceError::ResponseStructureError {
            field: "secftd.zip".to_string(),
            context: format!("empty fails-to-deliver archive: {e}"),
        })?;
    let mut text = String::new();
    entry
        .read_to_string(&mut text)
        .map_err(|e| FinanceError::ResponseStructureError {
            field: "secftd.zip".to_string(),
            context: format!("non-UTF8 fails-to-deliver data: {e}"),
        })?;
    Ok(parse_rows(&text, symbol))
}

async fn fetch_fails_to_deliver_with(
    base: &str,
    symbol: &str,
    lookback_periods: usize,
    max_attempts: usize,
) -> Result<Vec<FailToDeliver>> {
    let client = build_client()?;
    let symbol = symbol.to_uppercase();
    let mut records = Vec::new();
    let mut periods_found = 0;

    for period in periods_desc(Utc::now().date_naive()).take(max_attempts) {
        if periods_found >= lookback_periods {
            break;
        }
        let bytes = match client.get_document(&zip_url(base, &period)).await {
            Ok(bytes) => bytes,
            // Not yet published for this half-month; walk further back.
            Err(FinanceError::SymbolNotFound { .. }) => continue,
            Err(e) => return Err(e),
        };
        records.extend(parse_period(&bytes, &symbol)?);
        periods_found += 1;
    }

    records.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(records)
}

/// Fetch fails-to-deliver records for a symbol across the most recent
/// available bi-monthly periods, oldest first.
pub(crate) async fn fetch_fails_to_deliver_response(symbol: &str) -> Result<Vec<FailToDeliver>> {
    fetch_fails_to_deliver_with(SEC_WWW_BASE, symbol, LOOKBACK_PERIODS, MAX_ATTEMPTS).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn periods_walk_back_across_year_and_half_boundaries() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
        let periods: Vec<String> = periods_desc(start).take(4).collect();
        assert_eq!(periods, vec!["202601a", "202512b", "202512a", "202511b"]);
    }

    #[test]
    fn periods_split_on_the_15th() {
        let first_half = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let second_half = NaiveDate::from_ymd_opt(2026, 3, 16).unwrap();
        assert_eq!(periods_desc(first_half).next().unwrap(), "202603a");
        assert_eq!(periods_desc(second_half).next().unwrap(), "202603b");
    }

    #[test]
    fn rows_are_filtered_by_symbol_and_mapped() {
        let text = "SETTLEMENT DATE|CUSIP|SYMBOL|QUANTITY (FAILS)|DESCRIPTION|PRICE\n\
                     20250602|037833100|AAPL|69|APPLE INC;COM NPV|201.70\n\
                     20250602|B38564108|CMBT|51003|CMB.TECH NV (BEL)|8.83\n";

        let rows = parse_rows(text, "AAPL");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(rows[0].date.as_deref(), Some("2025-06-02"));
        assert_eq!(rows[0].quantity, Some(69.0));
        assert!((rows[0].price.unwrap() - 201.70).abs() < 1e-9);
        assert_eq!(rows[0].name.as_deref(), Some("APPLE INC;COM NPV"));
    }

    #[test]
    fn a_symbol_absent_from_the_period_yields_no_rows() {
        let text = "SETTLEMENT DATE|CUSIP|SYMBOL|QUANTITY (FAILS)|DESCRIPTION|PRICE\n\
                     20250602|B38564108|CMBT|51003|CMB.TECH NV (BEL)|8.83\n";
        assert!(parse_rows(text, "AAPL").is_empty());
    }

    fn zip_bytes(inner_name: &str, contents: &str) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer.start_file(inner_name, options).unwrap();
        writer.write_all(contents.as_bytes()).unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[tokio::test]
    async fn fetches_the_current_period_when_it_is_already_published() {
        let _ = crate::adapters::edgar::init("test@example.com");
        let mut server = mockito::Server::new_async().await;
        let period = periods_desc(Utc::now().date_naive()).next().unwrap();
        let body = zip_bytes(
            &format!("cnsfails{period}"),
            "SETTLEMENT DATE|CUSIP|SYMBOL|QUANTITY (FAILS)|DESCRIPTION|PRICE\n\
             20250602|037833100|AAPL|69|APPLE INC;COM NPV|201.70\n",
        );
        let _m = server
            .mock(
                "GET",
                format!("/files/data/fails-deliver-data/cnsfails{period}.zip").as_str(),
            )
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(body)
            .create_async()
            .await;

        let records = fetch_fails_to_deliver_with(&server.url(), "AAPL", 1, 1)
            .await
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].symbol.as_deref(), Some("AAPL"));
    }

    #[tokio::test]
    async fn walks_back_one_period_when_the_current_one_is_unpublished() {
        let _ = crate::adapters::edgar::init("test@example.com");
        let mut server = mockito::Server::new_async().await;
        let mut periods = periods_desc(Utc::now().date_naive());
        let current = periods.next().unwrap();
        let previous = periods.next().unwrap();

        let _missing = server
            .mock(
                "GET",
                format!("/files/data/fails-deliver-data/cnsfails{current}.zip").as_str(),
            )
            .with_status(404)
            .create_async()
            .await;
        let body = zip_bytes(
            &format!("cnsfails{previous}"),
            "SETTLEMENT DATE|CUSIP|SYMBOL|QUANTITY (FAILS)|DESCRIPTION|PRICE\n\
             20250602|037833100|AAPL|69|APPLE INC;COM NPV|201.70\n",
        );
        let _m = server
            .mock(
                "GET",
                format!("/files/data/fails-deliver-data/cnsfails{previous}.zip").as_str(),
            )
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(body)
            .create_async()
            .await;

        let records = fetch_fails_to_deliver_with(&server.url(), "AAPL", 1, 4)
            .await
            .unwrap();
        assert_eq!(records.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_live_fails_to_deliver() {
        let _ = crate::adapters::edgar::init("test@example.com");
        let records = fetch_fails_to_deliver_response("AAPL").await.unwrap();
        assert!(!records.is_empty());
    }
}
