//! Stock fundamental data: balance sheets, cash flow, income statements, ratios, short interest, float.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::fundamentals::FinancialStatement;
use crate::providers::build_financial_statement;
use crate::{Frequency, Provider, StatementType};

use super::build_client;
use super::models::PaginatedResponseDTO;

/// A financial statement row (balance sheet, income, or cash flow).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialResultDTO {
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
pub struct ShortInterestDTO {
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
pub struct ShortVolumeDTO {
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
pub struct FloatDataDTO {
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
pub struct FinancialRatiosDTO {
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
) -> Result<PaginatedResponseDTO<FinancialResultDTO>> {
    let client = build_client()?;
    let path = "/vX/reference/financials".to_string();
    let mut query: Vec<(&str, &str)> = vec![("ticker", ticker)];
    query.extend_from_slice(params);
    client.get(&path, &query).await
}

/// Fetch financial statements (canonical) for a stock ticker.
pub async fn fetch_financials_response(
    symbol: &str,
    stmt_type: StatementType,
    frequency: Frequency,
) -> Result<FinancialStatement> {
    let poly_type = match frequency {
        Frequency::Annual => "Y",
        Frequency::Quarterly => "Q",
    };
    let paginated = stock_financials(symbol, &[("type", poly_type), ("limit", "100")]).await?;
    let results = paginated.results.unwrap_or_default();

    let statement_key = match stmt_type {
        StatementType::Income => "income_statement",
        StatementType::Balance => "balance_sheet",
        StatementType::CashFlow => "cash_flow_statement",
    };

    let mut data: std::collections::HashMap<
        String,
        std::collections::HashMap<String, serde_json::Value>,
    > = std::collections::HashMap::new();

    for result in &results {
        let period = result
            .period_of_report_date
            .as_deref()
            .or(result.filing_date.as_deref())
            .unwrap_or("unknown");

        if let Some(ref financials) = result.financials
            && let Some(stmt_section) = financials.get(statement_key)
            && let Some(section_obj) = stmt_section.as_object()
        {
            for (metric, metric_obj) in section_obj {
                let metric_value = metric_obj
                    .get("value")
                    .cloned()
                    .unwrap_or_else(|| metric_obj.clone());
                data.entry(metric.clone())
                    .or_default()
                    .insert(period.to_string(), metric_value);
            }
        }
    }

    Ok(build_financial_statement(
        symbol.to_string(),
        stmt_type.as_str().to_string(),
        frequency.as_str().to_string(),
        Provider::Polygon,
        data,
    ))
}

/// Fetch short interest data for a stock ticker.
pub async fn stock_short_interest(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<ShortInterestDTO>> {
    let client = build_client()?;
    let path = format!(
        "/v3/reference/short-interest/{}",
        encode_path_segment(ticker)
    );
    client.get(&path, params).await
}

/// Fetch short volume data for a stock ticker.
pub async fn stock_short_volume(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<ShortVolumeDTO>> {
    let client = build_client()?;
    let path = format!("/v3/reference/short-volume/{}", encode_path_segment(ticker));
    client.get(&path, params).await
}

/// Fetch float data for a stock ticker.
pub async fn stock_float(ticker: &str) -> Result<PaginatedResponseDTO<FloatDataDTO>> {
    let client = build_client()?;
    let path = format!("/v3/reference/float/{}", encode_path_segment(ticker));
    client.get(&path, &[]).await
}

/// Fetch financial ratios for a stock ticker.
pub async fn stock_ratios(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FinancialRatiosDTO>> {
    let client = build_client()?;
    let path = "/vX/reference/financials/ratios".to_string();
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
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc",
                    "results": [{
                        "tickers": ["AAPL"],
                        "company_name": "Apple Inc",
                        "fiscal_period": "Q1",
                        "fiscal_year": "2024",
                        "filing_date": "2024-02-01"
                    }]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponseDTO<FinancialResultDTO> = client
            .get("/vX/reference/financials", &[("ticker", "AAPL")])
            .await
            .unwrap();
        let results = resp.results.unwrap();
        assert_eq!(results[0].company_name.as_deref(), Some("Apple Inc"));
    }
}
