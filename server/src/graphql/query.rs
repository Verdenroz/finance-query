//! GraphQL `QueryRoot` — top-level query fields.

use async_graphql::{Context, Object, Result};

use super::{
    error::to_gql_error,
    types::{
        enums::{GqlIndicesRegion, GqlScreener},
        market::{GqlFearAndGreed, GqlMarketSummaryQuote, GqlTrendingQuote},
        news::GqlNews,
        quote::GqlQuote,
    },
};
use crate::AppState;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Fetch data for a single ticker symbol.
    async fn ticker(&self, symbol: String) -> GqlTicker {
        GqlTicker { symbol }
    }

    /// Search for quotes and/or news matching a query string.
    async fn search(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 6)] quotes: u32,
        #[graphql(default = 0)] news: u32,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::search::search(
            &state.cache,
            &query,
            quotes,
            news,
            crate::services::search::SearchFlags {
                logo: true,
                ..Default::default()
            },
            None,
        )
        .await
        .map_err(to_gql_error)
    }

    /// List currently trending tickers.
    async fn trending(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<Vec<GqlTrendingQuote>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::market::get_trending(&state.cache, None)
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
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<Vec<GqlMarketSummaryQuote>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::market::get_market_summary(&state.cache, None)
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
    ) -> Result<Vec<GqlNews>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::news::get_general_news(&state.cache, count as usize)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Predefined stock screener results.
    async fn screener(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "type")] r#type: GqlScreener,
        #[graphql(default = 25)] count: u32,
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        let lib_screener: finance_query::Screener = r#type.into();
        let json = crate::services::market::get_screener(
            &state.cache,
            lib_screener,
            r#type.as_scr_id(),
            count,
        )
        .await
        .map_err(to_gql_error)?;
        Ok(finance_query::ValueFormat::from(format).transform(json))
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
}

/// Entry point for per-symbol queries. Actual data fetching happens in
/// `ComplexObject` resolvers so only requested fields trigger network calls.
#[derive(async_graphql::SimpleObject)]
#[graphql(complex)]
pub struct GqlTicker {
    pub symbol: String,
}

#[async_graphql::ComplexObject]
impl GqlTicker {
    async fn quote(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] logo: bool,
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<super::types::quote::GqlQuote> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::quote::get_quote(&state.cache, &self.symbol, logo)
            .await
            .map_err(to_gql_error)?;
        let json = finance_query::ValueFormat::from(format).transform(json);
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn chart(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlInterval::OneDay")] interval: super::types::enums::GqlInterval,
        #[graphql(default_with = "GqlTimeRange::OneMonth")]
        range: super::types::enums::GqlTimeRange,
        #[graphql(default)] events: bool,
        #[graphql(default)] patterns: bool,
    ) -> Result<super::types::chart::GqlChart> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::chart::get_chart(
            &state.cache,
            &self.symbol,
            interval.into(),
            range.into(),
            events,
            patterns,
        )
        .await
        .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn news(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] count: u32,
    ) -> Result<Vec<super::types::news::GqlNews>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::news::get_news(&state.cache, &self.symbol, count as usize)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn options(&self, ctx: &Context<'_>, date: Option<i64>) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::options::get_options(&state.cache, &self.symbol, date)
            .await
            .map_err(to_gql_error)
    }

    async fn financials(
        &self,
        ctx: &Context<'_>,
        statement: super::types::enums::GqlStatementType,
        #[graphql(default_with = "GqlFrequency::Annual")]
        frequency: super::types::enums::GqlFrequency,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::financials::get_financials(
            &state.cache,
            &self.symbol,
            statement.into(),
            statement.as_str(),
            frequency.into(),
            frequency.as_str(),
        )
        .await
        .map_err(to_gql_error)
    }

    async fn dividends(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: super::types::enums::GqlTimeRange,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::events::get_dividends(
            &state.cache,
            &self.symbol,
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)
    }

    async fn splits(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: super::types::enums::GqlTimeRange,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::events::get_splits(
            &state.cache,
            &self.symbol,
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)
    }

    async fn capital_gains(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: super::types::enums::GqlTimeRange,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::events::get_capital_gains(
            &state.cache,
            &self.symbol,
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)
    }

    async fn indicators(
        &self,
        ctx: &Context<'_>,
        interval: super::types::enums::GqlInterval,
        range: super::types::enums::GqlTimeRange,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::indicators::get_indicators(
            &state.cache,
            &self.symbol,
            interval.into(),
            interval.as_str(),
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)
    }

    async fn risk(
        &self,
        ctx: &Context<'_>,
        interval: super::types::enums::GqlInterval,
        range: super::types::enums::GqlTimeRange,
        benchmark: Option<String>,
    ) -> Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;
        crate::services::risk::get_risk(
            &state.cache,
            &self.symbol,
            interval.into(),
            interval.as_str(),
            range.into(),
            range.as_str(),
            benchmark.as_deref(),
        )
        .await
        .map_err(to_gql_error)
    }
}

// Bring enums into scope for default_with expressions evaluated in the macro context.
use super::types::enums::{GqlFrequency, GqlInterval, GqlTimeRange, GqlValueFormat};
