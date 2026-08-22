//! Analyst estimates, recommendations, earnings surprises, and grade history.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::fundamentals::{EarningsSurprise, GradingAction};

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
pub async fn earnings_surprises(symbol: &str) -> Result<Vec<EarningsSurpriseDTO>> {
    let client = build_client()?;
    client.get("/stable/earnings", &[("symbol", symbol)]).await
}

/// Fetch stock grade history for a symbol.
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

// ============================================================================
// Canonical conversions
// ============================================================================

/// Convert an earnings-surprise record into the canonical [`EarningsSurprise`],
/// deriving `surprise`/`surprise_percent` since FMP doesn't report them directly.
fn to_earnings_surprise(dto: EarningsSurpriseDTO) -> EarningsSurprise {
    let surprise = match (dto.actual_earning_result, dto.estimated_earning) {
        (Some(actual), Some(estimated)) => Some(actual - estimated),
        _ => None,
    };
    let surprise_percent = match (surprise, dto.estimated_earning) {
        (Some(s), Some(estimated)) if estimated != 0.0 => Some(s / estimated.abs() * 100.0),
        _ => None,
    };
    EarningsSurprise {
        symbol: dto.symbol,
        date: dto.date,
        actual_eps: dto.actual_earning_result,
        estimated_eps: dto.estimated_earning,
        surprise,
        surprise_percent,
    }
}

/// Fetch canonical earnings-surprise history for a symbol.
pub async fn fetch_earnings_surprises_response(symbol: &str) -> Result<Vec<EarningsSurprise>> {
    Ok(earnings_surprises(symbol)
        .await?
        .into_iter()
        .map(to_earnings_surprise)
        .collect())
}

fn to_grading_action(dto: StockGradeDTO) -> GradingAction {
    GradingAction {
        symbol: dto.symbol,
        date: dto.date,
        grading_company: dto.grading_company,
        previous_grade: dto.previous_grade,
        new_grade: dto.new_grade,
    }
}

/// Fetch canonical grade-action history for a symbol.
pub async fn fetch_grading_history_response(
    symbol: &str,
    limit: u32,
) -> Result<Vec<GradingAction>> {
    Ok(stock_grade(symbol, limit)
        .await?
        .into_iter()
        .map(to_grading_action)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyst_estimates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/analyst-estimates")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                mockito::Matcher::UrlEncoded("period".into(), "quarter".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "4".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "date": "2024-03-31",
                        "revenueAvg": 90000000000.0,
                        "epsAvg": 1.50,
                        "numAnalystsRevenue": 30,
                        "numAnalystsEps": 28
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<AnalystEstimateDTO> = client
            .get(
                "/stable/analyst-estimates",
                &[("symbol", "AAPL"), ("period", "quarter"), ("limit", "4")],
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
            .mock("GET", "/stable/earnings")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
            ]))
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
        let resp: Vec<EarningsSurpriseDTO> = client
            .get("/stable/earnings", &[("symbol", "AAPL")])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert!((resp[0].actual_earning_result.unwrap() - 2.18).abs() < 0.01);
    }

    #[test]
    fn maps_earnings_surprise_and_derives_surprise_fields() {
        let dto: EarningsSurpriseDTO = serde_json::from_value(serde_json::json!({
            "date": "2024-01-25",
            "symbol": "AAPL",
            "actualEarningResult": 2.18,
            "estimatedEarning": 2.10
        }))
        .unwrap();

        let out = to_earnings_surprise(dto);
        assert_eq!(out.actual_eps, Some(2.18));
        assert_eq!(out.estimated_eps, Some(2.10));
        assert!((out.surprise.unwrap() - 0.08).abs() < 1e-9);
        assert!((out.surprise_percent.unwrap() - 3.8095238095).abs() < 1e-6);
    }

    #[test]
    fn earnings_surprise_missing_estimate_yields_no_derived_fields() {
        let dto: EarningsSurpriseDTO = serde_json::from_value(serde_json::json!({
            "date": "2024-01-25",
            "symbol": "AAPL",
            "actualEarningResult": 2.18
        }))
        .unwrap();

        let out = to_earnings_surprise(dto);
        assert_eq!(out.surprise, None);
        assert_eq!(out.surprise_percent, None);
    }

    #[test]
    fn maps_stock_grade_to_grading_action() {
        let dto: StockGradeDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "date": "2024-01-15",
            "gradingCompany": "Morgan Stanley",
            "previousGrade": "Equal-Weight",
            "newGrade": "Overweight"
        }))
        .unwrap();

        let out = to_grading_action(dto);
        assert_eq!(out.grading_company.as_deref(), Some("Morgan Stanley"));
        assert_eq!(out.previous_grade.as_deref(), Some("Equal-Weight"));
        assert_eq!(out.new_grade.as_deref(), Some("Overweight"));
    }
}
