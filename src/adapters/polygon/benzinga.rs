//! Benzinga partner data: analyst ratings, insights, bull/bear, consensus, guidance, earnings, news.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::PaginatedResponse;

/// Analyst rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystRating {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Analyst name.
    pub analyst: Option<String>,
    /// Analyst firm.
    pub analyst_firm: Option<String>,
    /// Rating action (e.g., `"Initiates"`, `"Upgrades"`).
    pub action: Option<String>,
    /// Rating (e.g., `"Buy"`, `"Hold"`).
    pub rating: Option<String>,
    /// Prior rating.
    pub prior_rating: Option<String>,
    /// Price target.
    pub target_price: Option<f64>,
    /// Prior price target.
    pub prior_target_price: Option<f64>,
    /// Date.
    pub date: Option<String>,
    /// URL.
    pub url: Option<String>,
}

/// Analyst insight.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystInsight {
    /// Ticker.
    pub ticker: Option<String>,
    /// Analyst.
    pub analyst: Option<String>,
    /// Firm.
    pub analyst_firm: Option<String>,
    /// Insight type.
    pub insight_type: Option<String>,
    /// Rating.
    pub rating: Option<String>,
    /// Rationale.
    pub rationale: Option<String>,
    /// Target price.
    pub target_price: Option<f64>,
    /// Date.
    pub date: Option<String>,
}

/// Analyst details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystDetail {
    /// Analyst name.
    pub analyst_name: Option<String>,
    /// Firm name.
    pub firm_name: Option<String>,
    /// Analyst ID.
    pub analyst_id: Option<String>,
    /// Firm ID.
    pub firm_id: Option<String>,
    /// Number of ratings.
    pub ratings_count: Option<u32>,
}

/// Bull/bear summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BullBear {
    /// Ticker.
    pub ticker: Option<String>,
    /// Bull case.
    pub bull_case: Option<String>,
    /// Bear case.
    pub bear_case: Option<String>,
    /// Date.
    pub date: Option<String>,
}

/// Consensus rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConsensusRating {
    /// Ticker.
    pub ticker: Option<String>,
    /// Buy count.
    pub buy: Option<u32>,
    /// Hold count.
    pub hold: Option<u32>,
    /// Sell count.
    pub sell: Option<u32>,
    /// Strong buy count.
    pub strong_buy: Option<u32>,
    /// Strong sell count.
    pub strong_sell: Option<u32>,
    /// Consensus target price.
    pub target_price: Option<f64>,
    /// Target high.
    pub target_high: Option<f64>,
    /// Target low.
    pub target_low: Option<f64>,
}

/// Corporate guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CorporateGuidance {
    /// Ticker.
    pub ticker: Option<String>,
    /// EPS guidance.
    pub eps_guidance: Option<f64>,
    /// Revenue guidance.
    pub revenue_guidance: Option<f64>,
    /// Period.
    pub period: Option<String>,
    /// Date.
    pub date: Option<String>,
}

/// Earnings announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsAnnouncement {
    /// Ticker.
    pub ticker: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Report date.
    pub date: Option<String>,
    /// Reporting quarter.
    pub quarter: Option<String>,
    /// Actual EPS.
    pub eps_actual: Option<f64>,
    /// Estimated EPS.
    pub eps_estimate: Option<f64>,
    /// Actual revenue.
    pub revenue_actual: Option<f64>,
    /// Estimated revenue.
    pub revenue_estimate: Option<f64>,
}

/// Firm details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FirmDetail {
    /// Firm name.
    pub name: Option<String>,
    /// Firm ID.
    pub id: Option<String>,
    /// Number of analysts.
    pub analysts_count: Option<u32>,
}

/// Fetch analyst ratings.
pub async fn analyst_ratings(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<AnalystRating>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/analyst-ratings", params).await
}

/// Fetch analyst insights.
pub async fn analyst_insights(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<AnalystInsight>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/analyst-insights", params).await
}

/// Fetch analyst details.
pub async fn analyst_details(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<AnalystDetail>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/analyst-details", params).await
}

/// Fetch bull/bear summaries.
pub async fn bulls_bears(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<BullBear>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/bulls-bears-say", params).await
}

/// Fetch consensus ratings.
pub async fn consensus_ratings(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<ConsensusRating>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/consensus-ratings", params).await
}

/// Fetch corporate guidance.
pub async fn corporate_guidance(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<CorporateGuidance>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/corporate-guidance", params).await
}

/// Fetch earnings announcements.
pub async fn benzinga_earnings(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<EarningsAnnouncement>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/earnings", params).await
}

/// Fetch firm details.
pub async fn firm_details(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<FirmDetail>> {
    let client = build_client()?;
    client.get("/v2/reference/news/benzinga/firm-details", params).await
}
