//! Bulk and batch endpoints for Financial Modeling Prep.

use serde_json::Value;

use crate::error::Result;

use super::build_client;

/// Fetch bulk income statements as raw JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Period: `"annual"` or `"quarter"`
pub async fn bulk_income_statements(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_raw(
            "/api/v4/income-statement-bulk",
            &[("year", year), ("period", period)],
        )
        .await
}

/// Fetch bulk balance sheet statements as raw JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Period: `"annual"` or `"quarter"`
pub async fn bulk_balance_sheets(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_raw(
            "/api/v4/balance-sheet-statement-bulk",
            &[("year", year), ("period", period)],
        )
        .await
}

/// Fetch bulk cash flow statements as raw JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Period: `"annual"` or `"quarter"`
pub async fn bulk_cash_flow(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_raw(
            "/api/v4/cash-flow-statement-bulk",
            &[("year", year), ("period", period)],
        )
        .await
}

/// Fetch bulk financial ratios as raw JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Period: `"annual"` or `"quarter"`
pub async fn bulk_ratios(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_raw("/api/v4/ratios-bulk", &[("year", year), ("period", period)])
        .await
}

/// Fetch bulk key metrics as raw JSON.
///
/// * `year` - Fiscal year (e.g., `"2023"`)
/// * `period` - Period: `"annual"` or `"quarter"`
pub async fn bulk_key_metrics(year: &str, period: &str) -> Result<Value> {
    let client = build_client()?;
    client
        .get_raw(
            "/api/v4/key-metrics-bulk",
            &[("year", year), ("period", period)],
        )
        .await
}

/// Fetch all company profiles as raw JSON.
pub async fn bulk_profiles() -> Result<Value> {
    let client = build_client()?;
    client.get_raw("/api/v4/profile/all", &[]).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_bulk_income_statements_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v4/income-statement-bulk")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("year".into(), "2023".into()),
                mockito::Matcher::UrlEncoded("period".into(), "annual".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "revenue": 383285000000_i64,
                        "netIncome": 96995000000_i64
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client
            .get_raw(
                "/api/v4/income-statement-bulk",
                &[("year", "2023"), ("period", "annual")],
            )
            .await
            .unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 1);
    }
}
