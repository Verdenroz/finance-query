//! GraphQL types for analysis data (recommendations, upgrades, earnings).

use async_graphql::{Json, SimpleObject};
use serde::Deserialize;

// ── Recommendation Trend ────────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlRecommendationTrend {
    pub trend: Vec<GqlRecommendationPeriod>,
    pub max_age: Option<i64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlRecommendationPeriod {
    pub period: Option<String>,
    pub strong_buy: Option<i32>,
    pub buy: Option<i32>,
    pub hold: Option<i32>,
    pub sell: Option<i32>,
    pub strong_sell: Option<i32>,
}

// ── Upgrade/Downgrade History ───────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlUpgradeDowngradeHistory {
    pub history: Vec<GqlGradeChange>,
    pub max_age: Option<i64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlGradeChange {
    pub epoch_grade_date: Option<i64>,
    pub firm: Option<String>,
    pub from_grade: Option<String>,
    pub to_grade: Option<String>,
    pub action: Option<String>,
    pub prior_price_target: Option<f64>,
    pub current_price_target: Option<f64>,
    pub price_target_action: Option<String>,
}

// ── Earnings Trend (earnings estimate) ──────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlEarningsTrend {
    pub default_methodology: Option<String>,
    pub max_age: Option<i64>,
    pub trend: Vec<GqlEarningsTrendPeriod>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlEarningsTrendPeriod {
    pub end_date: Option<String>,
    pub earnings_estimate: Option<GqlEarningsEstimate>,
    pub revenue_estimate: Option<GqlRevenueEstimate>,
    pub eps_trend: Option<GqlEpsTrend>,
    pub eps_revisions: Option<GqlEpsRevisions>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlEarningsEstimate {
    pub avg: Option<Json<serde_json::Value>>,
    pub low: Option<Json<serde_json::Value>>,
    pub high: Option<Json<serde_json::Value>>,
    pub year_ago_eps: Option<Json<serde_json::Value>>,
    pub number_of_analysts: Option<Json<serde_json::Value>>,
    pub growth: Option<Json<serde_json::Value>>,
    pub earnings_currency: Option<String>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlRevenueEstimate {
    pub avg: Option<Json<serde_json::Value>>,
    pub low: Option<Json<serde_json::Value>>,
    pub high: Option<Json<serde_json::Value>>,
    pub number_of_analysts: Option<Json<serde_json::Value>>,
    pub year_ago_revenue: Option<Json<serde_json::Value>>,
    pub growth: Option<Json<serde_json::Value>>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlEpsTrend {
    pub current: Option<Json<serde_json::Value>>,
    #[serde(rename = "7daysAgo")]
    pub seven_days_ago: Option<Json<serde_json::Value>>,
    #[serde(rename = "30daysAgo")]
    pub thirty_days_ago: Option<Json<serde_json::Value>>,
    #[serde(rename = "60daysAgo")]
    pub sixty_days_ago: Option<Json<serde_json::Value>>,
    #[serde(rename = "90daysAgo")]
    pub ninety_days_ago: Option<Json<serde_json::Value>>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlEpsRevisions {
    pub up_last7days: Option<Json<serde_json::Value>>,
    pub up_last30days: Option<Json<serde_json::Value>>,
    #[serde(rename = "downLast7Days")]
    pub down_last7_days: Option<Json<serde_json::Value>>,
    pub down_last30days: Option<Json<serde_json::Value>>,
    pub down_last90days: Option<Json<serde_json::Value>>,
    pub eps_revisions_currency: Option<String>,
}

// ── Earnings History ────────────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlEarningsHistory {
    pub default_methodology: Option<String>,
    pub history: Vec<GqlEarningsHistoryEntry>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlEarningsHistoryEntry {
    pub max_age: Option<i64>,
    pub quarter: Option<Json<serde_json::Value>>,
    pub period: Option<String>,
    pub currency: Option<String>,
    pub eps_actual: Option<Json<serde_json::Value>>,
    pub eps_estimate: Option<Json<serde_json::Value>>,
    pub eps_difference: Option<Json<serde_json::Value>>,
    pub surprise_percent: Option<Json<serde_json::Value>>,
}
