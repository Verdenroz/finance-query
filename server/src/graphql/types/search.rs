//! GraphQL types for symbol/news search and type-filtered lookup.

use crate::graphql::pagination::{self, Page};
use async_graphql::{ComplexObject, Json, Result, SimpleObject};
use serde::Deserialize;

/// A quote/symbol result from `search`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlSearchQuote {
    pub symbol: String,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub quote_type: Option<String>,
    pub exchange: Option<String>,
    pub exch_disp: Option<String>,
    pub type_disp: Option<String>,
    pub industry: Option<String>,
    pub industry_disp: Option<String>,
    pub sector: Option<String>,
    pub sector_disp: Option<String>,
    #[serde(rename = "isYahooFinance")]
    pub is_yahoo_finance: Option<bool>,
    pub disp_sec_ind_flag: Option<bool>,
    pub logo_url: Option<String>,
    pub score: Option<f64>,
}

/// A news article result from `search`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlSearchNews {
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub publisher: Option<String>,
    pub link: Option<String>,
    pub provider_publish_time: Option<i64>,
    #[graphql(name = "type")]
    #[serde(rename = "type")]
    pub news_type: Option<String>,
    /// Opaque nested thumbnail object (resolutions/urls) — not curated further.
    pub thumbnail: Option<Json<serde_json::Value>>,
    pub related_tickers: Option<Vec<String>>,
}

/// A research report result from `search`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlResearchReport {
    pub report_headline: Option<String>,
    pub author: Option<String>,
    pub report_date: Option<i64>,
    pub id: Option<String>,
    pub provider: Option<String>,
}

/// Combined search results: quotes, news, and research reports.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(rename_all = "camelCase", default)]
pub struct GqlSearchResults {
    pub count: Option<i32>,
    #[graphql(skip)]
    pub quotes: Vec<GqlSearchQuote>,
    pub news: Vec<GqlSearchNews>,
    pub research_reports: Vec<GqlResearchReport>,
    pub total_time: Option<i64>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlSearchResults {
    /// Quote/symbol results.
    async fn quotes(
        &self,
        #[graphql(desc = "Max quotes to return; omitted = every matching quote in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlSearchQuote>> {
        pagination::paginate(self.quotes.clone(), first, after).await
    }
}

/// A quote/document result from `lookup`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlLookupQuote {
    pub symbol: String,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub quote_type: Option<String>,
    pub exchange: Option<String>,
    pub exch_disp: Option<String>,
    pub type_disp: Option<String>,
    pub industry: Option<String>,
    pub sector: Option<String>,
    pub regular_market_price: Option<f64>,
    pub regular_market_change: Option<f64>,
    pub regular_market_change_percent: Option<f64>,
    pub regular_market_previous_close: Option<f64>,
    pub logo_url: Option<String>,
    pub company_logo_url: Option<String>,
}

/// Type-filtered symbol lookup results.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlLookupResults {
    pub quotes: Vec<GqlLookupQuote>,
    pub start: Option<i32>,
    pub count: Option<i32>,
}
