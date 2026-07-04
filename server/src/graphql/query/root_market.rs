//! Market-wide root query fields: trending, indices, sector/industry data,
//! market summary, Fear & Greed, general news, and RSS/Atom feeds.

use async_graphql::{Context, Object, Result};

use super::resolve_gql_lang;
use crate::AppState;
use crate::graphql::error::to_gql_error;
use crate::graphql::types::{
    enums::{GqlIndicesRegion, GqlValueFormat},
    feeds::GqlFeedEntry,
    industry::GqlIndustryData,
    market::{GqlFearAndGreed, GqlMarketSummaryQuote, GqlTrendingQuote},
    news::GqlNews,
    quote::GqlQuote,
    sector::GqlSectorData,
};

#[derive(Default)]
pub(super) struct RootMarketQuery;

#[Object]
impl RootMarketQuery {
    /// List currently trending tickers.
    async fn trending(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Region code for localization (e.g. \"US\", \"JP\", \"GB\")")]
        region: Option<String>,
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<Vec<GqlTrendingQuote>> {
        let state = ctx.data::<AppState>()?;
        let region = region
            .as_deref()
            .and_then(|s| s.parse::<finance_query::Region>().ok());
        let json = crate::services::market::get_trending(&state.cache, region)
            .await
            .map_err(to_gql_error)?;
        let json = finance_query::ValueFormat::from(format).transform(json);
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Current Fear & Greed Index from alternative.me.
    async fn fear_and_greed(&self, ctx: &Context<'_>) -> Result<GqlFearAndGreed> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::market::get_fear_and_greed(&state.cache)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Market summary: major indices, currencies, and commodities.
    async fn market_summary(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Region code for localization (e.g. \"US\", \"JP\", \"GB\")")]
        region: Option<String>,
        #[graphql(default)] format: GqlValueFormat,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<Vec<GqlMarketSummaryQuote>> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let region = region
            .as_deref()
            .and_then(|s| s.parse::<finance_query::Region>().ok());
        let json =
            crate::services::market::get_market_summary(&state.cache, region, lang.as_deref())
                .await
                .map_err(to_gql_error)?;
        let json = finance_query::ValueFormat::from(format).transform(json);
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// General market news.
    async fn news(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] count: u32,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<Vec<GqlNews>> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let json =
            crate::services::news::get_general_news(&state.cache, count as usize, lang.as_deref())
                .await
                .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// RSS/Atom feed entries aggregated from one or more named financial
    /// publishers (see `finance_query::feeds::FeedSource` for the full list).
    async fn feeds(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            desc = "Source slugs, e.g. [\"bloomberg\", \"marketwatch\"] (default: federal-reserve, sec, marketwatch, bloomberg)"
        )]
        sources: Option<Vec<String>>,
        #[graphql(desc = "SEC form type for the sec-filings source (default: \"10-K\")")]
        form_type: Option<String>,
    ) -> Result<Vec<GqlFeedEntry>> {
        let state = ctx.data::<AppState>()?;
        let source_key = sources
            .as_deref()
            .map(|s| s.join(","))
            .unwrap_or_else(|| "all".to_string());
        let parsed =
            crate::services::feeds::parse_sources(sources.as_deref(), form_type.as_deref())
                .map_err(async_graphql::Error::new)?;
        let json = crate::services::feeds::get_feeds(
            &state.cache,
            &parsed,
            &source_key,
            form_type.as_deref().unwrap_or(""),
        )
        .await
        .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// World market indices, optionally filtered by region.
    async fn indices(
        &self,
        ctx: &Context<'_>,
        region: Option<GqlIndicesRegion>,
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<Vec<GqlQuote>> {
        let state = ctx.data::<AppState>()?;
        let lib_region = region.map(|r| r.into());
        let json = crate::services::market::get_indices(&state.cache, lib_region)
            .await
            .map_err(to_gql_error)?;

        // BatchQuotesResponse has a "quotes" field that is an object keyed by symbol.
        // Flatten it to a Vec<GqlQuote>.
        let quotes_map = json
            .get("quotes")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let lib_format = finance_query::ValueFormat::from(format);
        let mut result = Vec::with_capacity(quotes_map.len());
        for (_, v) in quotes_map {
            let v = lib_format.transform(v);
            let q: GqlQuote =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            result.push(q);
        }
        Ok(result)
    }

    /// Fetch sector-level market data.
    async fn sector(
        &self,
        ctx: &Context<'_>,
        sector: String,
        #[graphql(default)] format: GqlValueFormat,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<GqlSectorData> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let st: finance_query::Sector = sector
            .parse()
            .map_err(|_| async_graphql::Error::new(format!("Invalid sector: {sector}")))?;
        let json = crate::services::market::get_sector(&state.cache, st, &sector, lang.as_deref())
            .await
            .map_err(to_gql_error)?;
        let json = finance_query::ValueFormat::from(format).transform(json);
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Fetch industry-level market data.
    async fn industry(
        &self,
        ctx: &Context<'_>,
        industry: String,
        #[graphql(default)] format: GqlValueFormat,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<GqlIndustryData> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let json = crate::services::market::get_industry(&state.cache, &industry, lang.as_deref())
            .await
            .map_err(to_gql_error)?;
        let json = finance_query::ValueFormat::from(format).transform(json);
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }
}
