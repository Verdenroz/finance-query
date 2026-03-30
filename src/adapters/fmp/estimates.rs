//! Analyst estimates, recommendations, earnings surprises, grades, and transcripts.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::Period;

// ============================================================================
// Response types
// ============================================================================

/// Analyst estimate entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystEstimate {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Estimated revenue low.
    #[serde(rename = "estimatedRevenueLow")]
    pub estimated_revenue_low: Option<f64>,
    /// Estimated revenue high.
    #[serde(rename = "estimatedRevenueHigh")]
    pub estimated_revenue_high: Option<f64>,
    /// Estimated revenue avg.
    #[serde(rename = "estimatedRevenueAvg")]
    pub estimated_revenue_avg: Option<f64>,
    /// Estimated EBITDA low.
    #[serde(rename = "estimatedEbitdaLow")]
    pub estimated_ebitda_low: Option<f64>,
    /// Estimated EBITDA high.
    #[serde(rename = "estimatedEbitdaHigh")]
    pub estimated_ebitda_high: Option<f64>,
    /// Estimated EBITDA avg.
    #[serde(rename = "estimatedEbitdaAvg")]
    pub estimated_ebitda_avg: Option<f64>,
    /// Estimated EPS avg.
    #[serde(rename = "estimatedEpsAvg")]
    pub estimated_eps_avg: Option<f64>,
    /// Estimated EPS high.
    #[serde(rename = "estimatedEpsHigh")]
    pub estimated_eps_high: Option<f64>,
    /// Estimated EPS low.
    #[serde(rename = "estimatedEpsLow")]
    pub estimated_eps_low: Option<f64>,
    /// Number of analysts for revenue.
    #[serde(rename = "numberAnalystEstimatedRevenue")]
    pub number_analyst_estimated_revenue: Option<i32>,
    /// Number of analysts for EPS.
    #[serde(rename = "numberAnalystsEstimatedEps")]
    pub number_analysts_estimated_eps: Option<i32>,
}

/// Analyst recommendation entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystRecommendation {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Analyst ratings buy count.
    #[serde(rename = "analystRatingsBuy")]
    pub analyst_ratings_buy: Option<i32>,
    /// Analyst ratings hold count.
    #[serde(rename = "analystRatingsHold")]
    pub analyst_ratings_hold: Option<i32>,
    /// Analyst ratings sell count.
    #[serde(rename = "analystRatingsSell")]
    pub analyst_ratings_sell: Option<i32>,
    /// Analyst ratings strong buy count.
    #[serde(rename = "analystRatingsStrongBuy")]
    pub analyst_ratings_strong_buy: Option<i32>,
    /// Analyst ratings strong sell count.
    #[serde(rename = "analystRatingsStrongSell")]
    pub analyst_ratings_strong_sell: Option<i32>,
}

/// Earnings surprise entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsSurprise {
    /// Date.
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Actual earning result.
    #[serde(rename = "actualEarningResult")]
    pub actual_earning_result: Option<f64>,
    /// Estimated earning.
    #[serde(rename = "estimatedEarning")]
    pub estimated_earning: Option<f64>,
}

/// Stock grade entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockGrade {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Grading company.
    #[serde(rename = "gradingCompany")]
    pub grading_company: Option<String>,
    /// Previous grade.
    #[serde(rename = "previousGrade")]
    pub previous_grade: Option<String>,
    /// New grade.
    #[serde(rename = "newGrade")]
    pub new_grade: Option<String>,
}

/// Earnings call transcript entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsTranscript {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Quarter.
    pub quarter: Option<i32>,
    /// Year.
    pub year: Option<i32>,
    /// Date.
    pub date: Option<String>,
    /// Transcript content.
    pub content: Option<String>,
}

/// Earnings transcript list entry (available transcripts).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsTranscriptRef {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Quarter.
    pub quarter: Option<i32>,
    /// Year.
    pub year: Option<i32>,
    /// Date.
    pub date: Option<String>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch analyst estimates for a symbol.
///
/// * `period` - Annual or Quarter
/// * `limit` - Number of results
pub async fn analyst_estimates(
    symbol: &str,
    period: Period,
    limit: u32,
) -> Result<Vec<AnalystEstimate>> {
    let client = build_client()?;
    let path = format!("/api/v3/analyst-estimates/{symbol}");
    let limit_str = limit.to_string();
    client
        .get(&path, &[("period", period.as_str()), ("limit", &limit_str)])
        .await
}

/// Fetch analyst stock recommendations.
pub async fn analyst_recommendations(symbol: &str) -> Result<Vec<AnalystRecommendation>> {
    let client = build_client()?;
    let path = format!("/api/v3/analyst-stock-recommendations/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch earnings surprises for a symbol.
pub async fn earnings_surprises(symbol: &str) -> Result<Vec<EarningsSurprise>> {
    let client = build_client()?;
    let path = format!("/api/v3/earnings-surprises/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch stock grade history for a symbol.
pub async fn stock_grade(symbol: &str, limit: u32) -> Result<Vec<StockGrade>> {
    let client = build_client()?;
    let path = format!("/api/v3/grade/{symbol}");
    let limit_str = limit.to_string();
    client.get(&path, &[("limit", &*limit_str)]).await
}

/// Fetch an earnings call transcript.
///
/// * `quarter` - Quarter number (1-4)
/// * `year` - Year (e.g., 2024)
pub async fn earnings_transcript(
    symbol: &str,
    quarter: u32,
    year: u32,
) -> Result<Vec<EarningsTranscript>> {
    let client = build_client()?;
    let path = format!("/api/v3/earning_call_transcript/{symbol}");
    let q = quarter.to_string();
    let y = year.to_string();
    client.get(&path, &[("quarter", &*q), ("year", &*y)]).await
}

/// Fetch a list of available earnings transcripts for a symbol.
pub async fn earnings_transcript_list(symbol: &str) -> Result<Vec<EarningsTranscriptRef>> {
    let client = build_client()?;
    client
        .get(
            "/api/v4/earning_call_transcript",
            &[("symbol", symbol)],
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyst_estimates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/analyst-estimates/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "quarter".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "4".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "date": "2024-03-31",
                        "estimatedRevenueAvg": 90000000000.0,
                        "estimatedEpsAvg": 1.50,
                        "numberAnalystEstimatedRevenue": 30,
                        "numberAnalystsEstimatedEps": 28
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<AnalystEstimate> = client
            .get(
                "/api/v3/analyst-estimates/AAPL",
                &[("period", "quarter"), ("limit", "4")],
            )
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].symbol.as_deref(), Some("AAPL"));
        assert!((resp[0].estimated_eps_avg.unwrap() - 1.50).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_earnings_surprises_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/earnings-surprises/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "date": "2024-01-25",
                        "symbol": "AAPL",
                        "actualEarningResult": 2.18,
                        "estimatedEarning": 2.10
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<EarningsSurprise> = client
            .get("/api/v3/earnings-surprises/AAPL", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert!((resp[0].actual_earning_result.unwrap() - 2.18).abs() < 0.01);
    }
}
