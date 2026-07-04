//! GraphQL type for industry data (market-wide, on QueryRoot).

use async_graphql::SimpleObject;
use serde::Deserialize;

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlIndustryData {
    pub name: String,
    pub key: String,
    pub symbol: Option<String>,
    pub sector_name: Option<String>,
    pub sector_key: Option<String>,
    pub overview: Option<GqlIndustryOverview>,
    pub performance: Option<GqlIndustryPerformance>,
    pub benchmark: Option<GqlBenchmarkPerformance>,
    pub top_companies: Vec<GqlIndustryCompany>,
    pub top_performing_companies: Vec<GqlPerformingCompany>,
    pub top_growth_companies: Vec<GqlGrowthCompany>,
    pub research_reports: Vec<GqlIndustryResearchReport>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlIndustryOverview {
    pub description: Option<String>,
    pub companies_count: Option<i64>,
    pub market_cap: Option<f64>,
    pub market_weight: Option<f64>,
    pub employee_count: Option<i64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlIndustryPerformance {
    pub day_change_percent: Option<f64>,
    pub ytd_change_percent: Option<f64>,
    pub one_year_change_percent: Option<f64>,
    pub three_year_change_percent: Option<f64>,
    pub five_year_change_percent: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlBenchmarkPerformance {
    pub name: Option<String>,
    pub day_change_percent: Option<f64>,
    pub ytd_change_percent: Option<f64>,
    pub one_year_change_percent: Option<f64>,
    pub three_year_change_percent: Option<f64>,
    pub five_year_change_percent: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlIndustryCompany {
    pub symbol: String,
    pub name: Option<String>,
    pub last_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_weight: Option<f64>,
    pub day_change_percent: Option<f64>,
    pub ytd_return: Option<f64>,
    pub rating: Option<String>,
    pub target_price: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlPerformingCompany {
    pub symbol: String,
    pub name: Option<String>,
    pub last_price: Option<f64>,
    pub ytd_return: Option<f64>,
    pub target_price: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlGrowthCompany {
    pub symbol: String,
    pub name: Option<String>,
    pub last_price: Option<f64>,
    pub ytd_return: Option<f64>,
    pub growth_estimate: Option<f64>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlIndustryResearchReport {
    pub id: Option<String>,
    pub title: Option<String>,
    pub provider: Option<String>,
    pub report_date: Option<String>,
    pub report_type: Option<String>,
    pub investment_rating: Option<String>,
    pub target_price: Option<f64>,
    pub target_price_status: Option<String>,
}
