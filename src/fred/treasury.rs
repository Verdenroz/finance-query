//! US Treasury yield curve data.
//!
//! Fetches the daily Treasury yield curve from the US Treasury Department.
//! No API key required. Data published daily on business days.

use crate::error::{FinanceError, Result};
use crate::fred::models::TreasuryYield;
use tracing::info;

/// Base URL for Treasury yield curve CSV downloads.
const TREASURY_CSV_BASE: &str = "https://home.treasury.gov/resource-center/data-chart-center/interest-rates/daily-treasury-rates.csv";

/// Fetch the Treasury yield curve CSV for a given year and parse into typed records.
///
/// Fetches annual data for `year`. Pass the current year to get the most recent data.
pub(crate) async fn fetch_yields(year: u32) -> Result<Vec<TreasuryYield>> {
    let url = format!(
        "{TREASURY_CSV_BASE}/{year}/all?type=daily_treasury_yield_curve&field_tdr_date_value={year}&submit=time+series+1"
    );

    info!("Fetching Treasury yields for {year}");

    let resp = reqwest::get(&url).await.map_err(FinanceError::HttpError)?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        return Err(FinanceError::ExternalApiError {
            api: "US Treasury".to_string(),
            status,
        });
    }

    let text = resp.text().await.map_err(FinanceError::HttpError)?;
    parse_csv(&text, year)
}

/// Parse the Treasury yield CSV text into [`TreasuryYield`] records.
fn parse_csv(text: &str, year: u32) -> Result<Vec<TreasuryYield>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let mut yields = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| FinanceError::MacroDataError {
            provider: "US Treasury".to_string(),
            context: format!("CSV parse error for year {year}: {e}"),
        })?;

        let date = record.get(0).unwrap_or("").trim().to_string();
        if date.is_empty() {
            continue;
        }

        let parse_col = |i: usize| -> Option<f64> { record.get(i)?.trim().parse::<f64>().ok() };

        yields.push(TreasuryYield {
            date,
            y1m: parse_col(1),
            y2m: parse_col(2),
            y3m: parse_col(3),
            y4m: parse_col(4),
            y6m: parse_col(5),
            y1: parse_col(6),
            y2: parse_col(7),
            y3: parse_col(8),
            y5: parse_col(9),
            y7: parse_col(10),
            y10: parse_col(11),
            y20: parse_col(12),
            y30: parse_col(13),
        });
    }

    Ok(yields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_row() {
        let csv = "Date,1 Mo,2 Mo,3 Mo,4 Mo,6 Mo,1 Yr,2 Yr,3 Yr,5 Yr,7 Yr,10 Yr,20 Yr,30 Yr\n\
                   01/02/2025,4.33,4.38,4.36,4.38,4.37,4.29,4.20,4.24,4.38,4.47,4.57,4.87,4.79\n";
        let yields = parse_csv(csv, 2025).unwrap();
        assert_eq!(yields.len(), 1);
        let row = &yields[0];
        assert_eq!(row.date, "01/02/2025");
        assert_eq!(row.y1m, Some(4.33));
        assert_eq!(row.y10, Some(4.57));
        assert_eq!(row.y30, Some(4.79));
    }

    #[test]
    fn test_parse_missing_values() {
        let csv = "Date,1 Mo,2 Mo,3 Mo,4 Mo,6 Mo,1 Yr,2 Yr,3 Yr,5 Yr,7 Yr,10 Yr,20 Yr,30 Yr\n\
                   01/02/2025,N/A,,,,,4.29,4.20,4.24,4.38,4.47,4.57,,4.79\n";
        let yields = parse_csv(csv, 2025).unwrap();
        assert_eq!(yields.len(), 1);
        assert!(yields[0].y1m.is_none());
        assert_eq!(yields[0].y1, Some(4.29));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_yields_current_year() {
        let yields = fetch_yields(2025).await;
        assert!(yields.is_ok(), "Expected ok, got: {:?}", yields.err());
        let yields = yields.unwrap();
        assert!(!yields.is_empty());
        for y in yields.iter().take(3) {
            assert!(!y.date.is_empty());
        }
    }
}
