//! Benzinga partner data: analyst ratings, insights, bull/bear, consensus, guidance, earnings, news.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::super::build_client;
use super::super::models::PaginatedResponseDTO;

/// Analyst rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystRatingDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Analyst name.
    pub analyst: Option<String>,
    /// Analyst firm.
    #[serde(alias = "analyst_firm")]
    pub firm: Option<String>,
    /// Rating action (e.g., `"Initiates"`, `"Upgrades"`).
    #[serde(alias = "action")]
    pub rating_action: Option<String>,
    /// Rating (e.g., `"Buy"`, `"Hold"`).
    pub rating: Option<String>,
    /// Prior rating.
    pub prior_rating: Option<String>,
    /// Price target.
    #[serde(alias = "target_price")]
    pub price_target: Option<f64>,
    /// Prior price target.
    #[serde(alias = "prior_target_price")]
    pub adjusted_price_target_prior: Option<f64>,
    /// Date.
    pub date: Option<String>,
    /// URL.
    pub url: Option<String>,
}

/// Analyst insight.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystInsightDTO {
    /// Ticker.
    pub ticker: Option<String>,
    /// Analyst.
    pub analyst: Option<String>,
    /// Firm.
    #[serde(alias = "analyst_firm")]
    pub firm: Option<String>,
    /// InsightDTO type.
    pub insight_type: Option<String>,
    /// Rating.
    pub rating: Option<String>,
    /// Rationale.
    #[serde(alias = "rationale")]
    pub insight: Option<String>,
    /// Target price.
    #[serde(alias = "target_price")]
    pub price_target: Option<f64>,
    /// Date.
    pub date: Option<String>,
}

/// Analyst details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalystDetailDTO {
    /// Analyst name.
    #[serde(alias = "analyst_name")]
    pub name: Option<String>,
    /// Firm name.
    #[serde(alias = "firm_name")]
    pub firm: Option<String>,
    /// Analyst ID.
    #[serde(alias = "analyst_id")]
    pub benzinga_id: Option<String>,
    /// Firm ID.
    pub firm_id: Option<String>,
    /// Number of ratings.
    pub ratings_count: Option<u32>,
}

/// Bull/bear summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BullBearDTO {
    /// Ticker.
    pub ticker: Option<String>,
    /// Bull case.
    #[serde(alias = "bull_case")]
    pub bull_case_summary: Option<String>,
    /// Bear case.
    #[serde(alias = "bear_case")]
    pub bear_case_summary: Option<String>,
    /// Date.
    pub date: Option<String>,
}

/// Consensus rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConsensusRatingDTO {
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
pub struct CorporateGuidanceDTO {
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
pub struct EarningsAnnouncementDTO {
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
pub struct FirmDetailDTO {
    /// Firm name.
    pub name: Option<String>,
    /// Firm ID.
    #[serde(alias = "id")]
    pub benzinga_id: Option<String>,
    /// Number of analysts.
    pub analysts_count: Option<u32>,
}

/// Fetch analyst ratings.
pub async fn analyst_ratings(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<AnalystRatingDTO>> {
    let client = build_client()?;
    client.get("/benzinga/v1/ratings", params).await
}

/// Fetch analyst insights.
pub async fn analyst_insights(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<AnalystInsightDTO>> {
    let client = build_client()?;
    client.get("/benzinga/v1/analyst-insights", params).await
}

/// Fetch analyst details.
pub async fn analyst_details(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<AnalystDetailDTO>> {
    let client = build_client()?;
    client.get("/benzinga/v1/analysts", params).await
}

/// Fetch bull/bear summaries.
pub async fn bulls_bears(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<BullBearDTO>> {
    let client = build_client()?;
    client.get("/benzinga/v1/bulls-bears-say", params).await
}

/// Fetch consensus ratings.
pub async fn consensus_ratings(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<ConsensusRatingDTO>> {
    let client = build_client()?;
    let path = format!(
        "/benzinga/v1/consensus-ratings/{}",
        crate::adapters::common::encode_path_segment(ticker)
    );
    client.get(&path, params).await
}

/// Fetch corporate guidance.
pub async fn corporate_guidance(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<CorporateGuidanceDTO>> {
    let client = build_client()?;
    client.get("/benzinga/v1/guidance", params).await
}

/// Fetch earnings announcements.
pub async fn benzinga_earnings(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EarningsAnnouncementDTO>> {
    let client = build_client()?;
    client.get("/benzinga/v1/earnings", params).await
}

/// Fetch firm details.
pub async fn firm_details(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<FirmDetailDTO>> {
    let client = build_client()?;
    client.get("/benzinga/v1/firms", params).await
}
