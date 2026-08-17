//! Analyst estimates, recommendations, earnings surprises, grades, and transcripts.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::models::Period;

// ============================================================================
// Response types
// ============================================================================

/// Analyst estimate entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystEstimateDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Estimated revenue low.
    #[serde(rename = "revenueLow")]
    pub estimated_revenue_low: Option<f64>,
    /// Estimated revenue high.
    #[serde(rename = "revenueHigh")]
    pub estimated_revenue_high: Option<f64>,
    /// Estimated revenue avg.
    #[serde(rename = "revenueAvg")]
    pub estimated_revenue_avg: Option<f64>,
    /// Estimated EBITDA low.
    #[serde(rename = "ebitdaLow")]
    pub estimated_ebitda_low: Option<f64>,
    /// Estimated EBITDA high.
    #[serde(rename = "ebitdaHigh")]
    pub estimated_ebitda_high: Option<f64>,
    /// Estimated EBITDA avg.
    #[serde(rename = "ebitdaAvg")]
    pub estimated_ebitda_avg: Option<f64>,
    /// Estimated EPS avg.
    #[serde(rename = "epsAvg")]
    pub estimated_eps_avg: Option<f64>,
    /// Estimated EPS high.
    #[serde(rename = "epsHigh")]
    pub estimated_eps_high: Option<f64>,
    /// Estimated EPS low.
    #[serde(rename = "epsLow")]
    pub estimated_eps_low: Option<f64>,
    /// Number of analysts for revenue.
    #[serde(rename = "numAnalystsRevenue")]
    pub number_analyst_estimated_revenue: Option<i32>,
    /// Number of analysts for EPS.
    #[serde(rename = "numAnalystsEps")]
    pub number_analysts_estimated_eps: Option<i32>,
}

/// Analyst recommendation entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystRecommendationDTO {
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
#[allow(dead_code)] // unrouted: analyst-consensus rollups land with #241
pub struct EarningsSurpriseDTO {
    /// Date.
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Actual earning result.
    #[serde(rename = "epsActual", alias = "actualEarningResult")]
    pub actual_earning_result: Option<f64>,
    /// Estimated earning.
    #[serde(rename = "epsEstimated", alias = "estimatedEarning")]
    pub estimated_earning: Option<f64>,
}

/// Stock grade entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)] // unrouted: analyst-consensus rollups land with #241
pub struct StockGradeDTO {
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
#[allow(dead_code)] // unrouted: analyst-consensus rollups land with #241
pub struct EarningsTranscriptDTO {
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
#[allow(dead_code)] // unrouted: analyst-consensus rollups land with #241
pub struct EarningsTranscriptRefDTO {
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
) -> Result<Vec<AnalystEstimateDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/analyst-estimates",
            &[
                ("symbol", symbol),
                ("period", period.as_str()),
                ("limit", &limit_str),
                ("page", "0"),
            ],
        )
        .await
}

/// Fetch the dated history of analyst recommendation counts for a symbol.
pub async fn analyst_recommendations(symbol: &str) -> Result<Vec<AnalystRecommendationDTO>> {
    let client = build_client()?;
    client
        .get("/stable/grades-historical", &[("symbol", symbol)])
        .await
}

/// Fetch earnings surprises for a symbol.
#[allow(dead_code)] // unrouted: analyst-consensus rollups land with #241
pub async fn earnings_surprises(symbol: &str) -> Result<Vec<EarningsSurpriseDTO>> {
    let client = build_client()?;
    client.get("/stable/earnings", &[("symbol", symbol)]).await
}

/// Fetch stock grade history for a symbol.
#[allow(dead_code)] // unrouted: analyst-consensus rollups land with #241
pub async fn stock_grade(symbol: &str, limit: u32) -> Result<Vec<StockGradeDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/grades",
            &[("symbol", symbol), ("limit", &limit_str)],
        )
        .await
}

/// Fetch an earnings call transcript.
///
/// * `quarter` - Quarter number (1-4)
/// * `year` - Year (e.g., 2024)
#[allow(dead_code)] // unrouted: awaiting a capability route; see #264.
pub async fn earnings_transcript(
    symbol: &str,
    quarter: u32,
    year: u32,
) -> Result<Vec<EarningsTranscriptDTO>> {
    let client = build_client()?;
    let q = quarter.to_string();
    let y = year.to_string();
    client
        .get(
            "/stable/earning-call-transcript",
            &[("symbol", symbol), ("quarter", &q), ("year", &y)],
        )
        .await
}

/// Fetch a list of available earnings transcripts for a symbol.
#[allow(dead_code)] // unrouted: awaiting a capability route; see #264.
pub async fn earnings_transcript_list(symbol: &str) -> Result<Vec<EarningsTranscriptRefDTO>> {
    let client = build_client()?;
    client
        .get(
            "/stable/earning-call-transcript-dates",
            &[("symbol", symbol)],
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stable_transcript_route_uses_symbol_year_and_quarter() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/earning-call-transcript")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                mockito::Matcher::UrlEncoded("year".into(), "2024".into()),
                mockito::Matcher::UrlEncoded("quarter".into(), "2".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"symbol":"AAPL","quarter":2,"year":2024,"content":"Call"}]"#)
            .create_async()
            .await;
        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let rows: Vec<EarningsTranscriptDTO> = client
            .get(
                "/stable/earning-call-transcript",
                &[("symbol", "AAPL"), ("quarter", "2"), ("year", "2024")],
            )
            .await
            .unwrap();

        assert_eq!(rows[0].symbol.as_deref(), Some("AAPL"));
    }

    #[tokio::test]
    async fn test_analyst_estimates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/analyst-estimates")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "quarter".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "4".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"[{
                    "symbol": "AAPL",
                    "date": "2024-03-31",
                    "revenueLow": 85000000000.0,
                    "revenueHigh": 95000000000.0,
                    "revenueAvg": 90000000000.0,
                    "ebitdaLow": 28000000000.0,
                    "ebitdaHigh": 33000000000.0,
                    "ebitdaAvg": 30500000000.0,
                    "epsAvg": 1.50,
                    "epsHigh": 1.62,
                    "epsLow": 1.41,
                    "numAnalystsRevenue": 30,
                    "numAnalystsEps": 28
                }]"#,
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<AnalystEstimateDTO> = client
            .get(
                "/stable/analyst-estimates",
                &[("period", "quarter"), ("limit", "4")],
            )
            .await
            .unwrap();

        let row = &resp[0];
        assert_eq!(row.symbol.as_deref(), Some("AAPL"));
        assert_eq!(row.date.as_deref(), Some("2024-03-31"));
        assert_eq!(row.estimated_revenue_low, Some(85_000_000_000.0));
        assert_eq!(row.estimated_revenue_high, Some(95_000_000_000.0));
        assert_eq!(row.estimated_revenue_avg, Some(90_000_000_000.0));
        assert_eq!(row.estimated_ebitda_low, Some(28_000_000_000.0));
        assert_eq!(row.estimated_ebitda_high, Some(33_000_000_000.0));
        assert_eq!(row.estimated_ebitda_avg, Some(30_500_000_000.0));
        assert_eq!(row.estimated_eps_avg, Some(1.50));
        assert_eq!(row.estimated_eps_high, Some(1.62));
        assert_eq!(row.estimated_eps_low, Some(1.41));
        assert_eq!(row.number_analyst_estimated_revenue, Some(30));
        assert_eq!(row.number_analysts_estimated_eps, Some(28));
    }

    #[tokio::test]
    async fn test_earnings_surprises_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/earnings")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "date": "2024-01-25",
                        "symbol": "AAPL",
                        "epsActual": 2.18,
                        "epsEstimated": 2.10
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<EarningsSurpriseDTO> = client.get("/stable/earnings", &[]).await.unwrap();
        assert_eq!(resp.len(), 1);
        assert!((resp[0].actual_earning_result.unwrap() - 2.18).abs() < 0.01);
    }
}
