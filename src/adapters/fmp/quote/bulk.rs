//! Bulk and batch endpoints for Financial Modeling Prep.

use serde_json::Value;

use crate::error::{FinanceError, Result};

use crate::adapters::fmp::build_client;

/// Fetch bulk income statements and convert FMP's CSV response to JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Stable period (`"FY"`, `"Q1"`, `"Q2"`, `"Q3"`, or `"Q4"`)
pub async fn bulk_income_statements(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_csv_value(
            "/stable/income-statement-bulk",
            &[("year", year), ("period", period)],
        )
        .await
}

/// Fetch bulk balance sheet statements and convert FMP's CSV response to JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Stable period (`"FY"`, `"Q1"`, `"Q2"`, `"Q3"`, or `"Q4"`)
pub async fn bulk_balance_sheets(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_csv_value(
            "/stable/balance-sheet-statement-bulk",
            &[("year", year), ("period", period)],
        )
        .await
}

/// Fetch bulk cash flow statements and convert FMP's CSV response to JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Stable period (`"FY"`, `"Q1"`, `"Q2"`, `"Q3"`, or `"Q4"`)
pub async fn bulk_cash_flow(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_csv_value(
            "/stable/cash-flow-statement-bulk",
            &[("year", year), ("period", period)],
        )
        .await
}

/// Fetch current trailing-twelve-month ratios for all companies.
pub async fn bulk_ratios_ttm() -> Result<Value> {
    build_client()?
        .get_csv_value("/stable/ratios-ttm-bulk", &[])
        .await
}

/// Fetch current trailing-twelve-month key metrics for all companies.
pub async fn bulk_key_metrics_ttm() -> Result<Value> {
    build_client()?
        .get_csv_value("/stable/key-metrics-ttm-bulk", &[])
        .await
}

/// Fetch all company profiles and convert FMP's CSV responses to JSON.
pub async fn bulk_profiles() -> Result<Value> {
    let client = build_client()?;
    let mut profiles = Vec::new();
    for part in 0..4_u32 {
        let part = part.to_string();
        let response = client
            .get_csv_value("/stable/profile-bulk", &[("part", &part)])
            .await?;
        let Value::Array(mut rows) = response else {
            return Err(FinanceError::UnexpectedResponse(
                "FMP profile-bulk returned a non-array response".into(),
            ));
        };
        profiles.append(&mut rows);
    }
    Ok(Value::Array(profiles))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_bulk_income_statements_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/income-statement-bulk")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("year".into(), "2023".into()),
                mockito::Matcher::UrlEncoded("period".into(), "annual".into()),
            ]))
            .with_status(200)
            .with_body("symbol,revenue,netIncome\nAAPL,383285000000,96995000000\n")
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result = client
            .get_csv_value(
                "/stable/income-statement-bulk",
                &[("year", "2023"), ("period", "annual")],
            )
            .await
            .unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 1);
    }
}
