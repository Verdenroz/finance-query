//! GraphQL type for sector data (market-wide, on QueryRoot).

use super::formatted::GqlFormattedValue;
use async_graphql::{Json, SimpleObject};
use serde::Deserialize;

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorData {
    pub name: String,
    pub symbol: Option<String>,
    pub key: String,
    pub overview: Option<GqlSectorOverview>,
    pub performance: Option<GqlSectorPerformance>,
    pub benchmark: Option<GqlSectorPerformance>,
    pub benchmark_name: Option<String>,
    pub top_companies: Vec<GqlSectorCompany>,
    pub top_etfs: Vec<GqlSectorETF>,
    pub top_mutual_funds: Vec<GqlSectorMutualFund>,
    pub industries: Vec<GqlSectorIndustry>,
    pub research_reports: Vec<GqlSectorResearchReport>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorOverview {
    pub companies_count: Option<i64>,
    pub market_cap: Option<Json<GqlFormattedValue>>,
    pub description: Option<String>,
    pub industries_count: Option<i64>,
    pub market_weight: Option<Json<GqlFormattedValue>>,
    pub employee_count: Option<Json<GqlFormattedValue>>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorPerformance {
    pub ytd_change_percent: Option<Json<GqlFormattedValue>>,
    pub day_change_percent: Option<Json<GqlFormattedValue>>,
    pub one_year_change_percent: Option<Json<GqlFormattedValue>>,
    pub three_year_change_percent: Option<Json<GqlFormattedValue>>,
    pub five_year_change_percent: Option<Json<GqlFormattedValue>>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorCompany {
    pub symbol: String,
    pub name: Option<String>,
    pub market_cap: Option<Json<GqlFormattedValue>>,
    pub market_weight: Option<Json<GqlFormattedValue>>,
    pub last_price: Option<Json<GqlFormattedValue>>,
    pub target_price: Option<Json<GqlFormattedValue>>,
    pub day_change_percent: Option<Json<GqlFormattedValue>>,
    pub ytd_return: Option<Json<GqlFormattedValue>>,
    pub rating: Option<String>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorETF {
    pub symbol: String,
    pub name: Option<String>,
    pub net_assets: Option<Json<GqlFormattedValue>>,
    pub expense_ratio: Option<Json<GqlFormattedValue>>,
    pub last_price: Option<Json<GqlFormattedValue>>,
    pub ytd_return: Option<Json<GqlFormattedValue>>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorMutualFund {
    pub symbol: String,
    pub name: Option<String>,
    pub net_assets: Option<Json<GqlFormattedValue>>,
    pub expense_ratio: Option<Json<GqlFormattedValue>>,
    pub last_price: Option<Json<GqlFormattedValue>>,
    pub ytd_return: Option<Json<GqlFormattedValue>>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorIndustry {
    pub symbol: Option<String>,
    pub key: Option<String>,
    pub name: String,
    pub market_weight: Option<Json<GqlFormattedValue>>,
    pub day_change_percent: Option<Json<GqlFormattedValue>>,
    pub ytd_return: Option<Json<GqlFormattedValue>>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlSectorResearchReport {
    pub id: String,
    pub headline: Option<String>,
    pub provider: Option<String>,
    pub report_date: Option<String>,
    pub report_title: Option<String>,
    pub report_type: Option<String>,
    pub target_price: Option<f64>,
    pub target_price_status: Option<String>,
    pub investment_rating: Option<String>,
}
