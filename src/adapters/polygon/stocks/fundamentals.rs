//! Stock fundamental data: balance sheets, cash flow, income statements, ratios, short interest, float.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::super::build_client;
use super::super::models::PaginatedResponse;

/// A financial statement row (balance sheet, income, or cash flow).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialResult {
    /// Ticker symbol.
    pub tickers: Option<Vec<String>>,
    /// Company name.
    pub company_name: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Filing date.
    pub filing_date: Option<String>,
    /// Period of report.
    pub period_of_report_date: Option<String>,
    /// Fiscal period (e.g., `"Q1"`, `"FY"`).
    pub fiscal_period: Option<String>,
    /// Fiscal year.
    pub fiscal_year: Option<String>,
    /// Source filing URL.
    pub source_filing_url: Option<String>,
    /// Financials data (nested by statement type).
    pub financials: Option<serde_json::Value>,
}

/// Short interest data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShortInterest {
    /// Settlement date.
    pub settlement_date: Option<String>,
    /// Short interest (shares).
    pub short_interest: Option<f64>,
    /// Average daily volume.
    pub avg_daily_volume: Option<f64>,
    /// Days to cover.
    pub days_to_cover: Option<f64>,
}

/// Short volume data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShortVolume {
    /// Date.
    pub date: Option<String>,
    /// Short volume.
    pub short_volume: Option<f64>,
    /// Short exempt volume.
    pub short_exempt_volume: Option<f64>,
    /// Total volume.
    pub total_volume: Option<f64>,
}

/// Float data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FloatData {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Float shares.
    pub float_shares: Option<f64>,
    /// Outstanding shares.
    pub outstanding_shares: Option<f64>,
    /// Date.
    pub date: Option<String>,
}

/// Financial ratios.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialRatios {
    /// Ticker.
    pub ticker: Option<String>,
    /// Period.
    pub period: Option<String>,
    /// Fiscal year.
    pub fiscal_year: Option<String>,
    /// All ratio values as key-value pairs.
    #[serde(flatten)]
    pub ratios: std::collections::HashMap<String, serde_json::Value>,
}

/// Fetch stock financials (balance sheets, income statements, cash flow).
///
/// * `ticker` - Stock ticker symbol
/// * `params` - Optional: `type` (Y, Q, YA, QA, T), `filing_date`, `period_of_report_date`, `limit`, `sort`, `order`
pub async fn stock_financials(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<FinancialResult>> {
    let client = build_client()?;
    let path = format!("/vX/reference/financials");
    let mut query: Vec<(&str, &str)> = vec![("ticker", ticker)];
    query.extend_from_slice(params);
    client.get(&path, &query).await
}

/// Fetch short interest data for a stock ticker.
pub async fn stock_short_interest(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<ShortInterest>> {
    let client = build_client()?;
    let path = format!("/v3/reference/short-interest/{}", ticker);
    client.get(&path, params).await
}

/// Fetch short volume data for a stock ticker.
pub async fn stock_short_volume(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<ShortVolume>> {
    let client = build_client()?;
    let path = format!("/v3/reference/short-volume/{}", ticker);
    client.get(&path, params).await
}

/// Fetch float data for a stock ticker.
pub async fn stock_float(ticker: &str) -> Result<PaginatedResponse<FloatData>> {
    let client = build_client()?;
    let path = format!("/v3/reference/float/{}", ticker);
    client.get(&path, &[]).await
}

/// Fetch financial ratios for a stock ticker.
pub async fn stock_ratios(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<FinancialRatios>> {
    let client = build_client()?;
    let path = format!("/vX/reference/financials/ratios");
    let mut query: Vec<(&str, &str)> = vec![("ticker", ticker)];
    query.extend_from_slice(params);
    client.get(&path, &query).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stock_financials_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/vX/reference/financials")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("ticker".into(), "AAPL".into()),
            ]))
            .with_status(200)
            .with_body(serde_json::json!({
                "status": "OK",
                "request_id": "abc",
                "results": [{
                    "tickers": ["AAPL"],
                    "company_name": "Apple Inc",
                    "fiscal_period": "Q1",
                    "fiscal_year": "2024",
                    "filing_date": "2024-02-01"
                }]
            }).to_string())
            .create_async().await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<FinancialResult> = client
            .get("/vX/reference/financials", &[("ticker", "AAPL")])
            .await.unwrap();
        let results = resp.results.unwrap();
        assert_eq!(results[0].company_name.as_deref(), Some("Apple Inc"));
    }
}
