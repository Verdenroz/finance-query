//! Misc market-metadata root query fields: crypto, EDGAR lookups, market
//! hours/quote-type/currencies/exchanges, calendar, and FRED/Treasury data.

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::to_gql_error;
use crate::graphql::pagination::{self, Page};
use crate::graphql::types::{
    calendar::GqlCalendarEvent,
    crypto::GqlCoinQuote,
    edgar::{GqlEdgarCik, GqlEdgarSearchHit, GqlEdgarSearchResults},
    enums::GqlTimeRange,
    fred::{GqlMacroSeries, GqlTreasuryYield},
    metadata::{GqlCurrency, GqlExchange, GqlMarketHours, GqlQuoteTypeData},
};

#[derive(Default)]
pub(super) struct RootMetadataQuery;

#[Object]
impl RootMetadataQuery {
    /// Top N cryptocurrency quotes by market cap (CoinGecko).
    async fn crypto_coins(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "\"usd\".to_string()")] vs_currency: String,
        #[graphql(default = 50, desc = "Overall cap on coins fetched")] count: u32,
        #[graphql(
            desc = "Max coins per page; omitted = every fetched coin (up to `count`) in one page"
        )]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlCoinQuote>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::crypto::get_coins(&state.cache, &vs_currency, count as usize)
            .await
            .map_err(to_gql_error)?;
        let coins: Vec<GqlCoinQuote> =
            serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        pagination::paginate(coins, first, after).await
    }

    /// A single cryptocurrency quote by CoinGecko ID (e.g. "bitcoin").
    async fn crypto_coin(
        &self,
        ctx: &Context<'_>,
        id: String,
        #[graphql(default_with = "\"usd\".to_string()")] vs_currency: String,
    ) -> Result<GqlCoinQuote> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::crypto::get_coin(&state.cache, &id, &vs_currency)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Resolve a ticker symbol to its SEC CIK number. Requires `EDGAR_EMAIL`.
    async fn edgar_cik(&self, ctx: &Context<'_>, symbol: String) -> Result<GqlEdgarCik> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::edgar::get_cik(&state.cache, &symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Market open/close hours, optionally for a specific region.
    async fn market_hours(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Region code (e.g. \"US\", \"JP\", \"GB\")")] region: Option<String>,
    ) -> Result<GqlMarketHours> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::metadata::get_hours(&state.cache, region.as_deref())
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Quote type metadata (exchange, timezone, identifiers) for a symbol.
    async fn quote_type(
        &self,
        ctx: &Context<'_>,
        symbol: String,
    ) -> Result<Option<GqlQuoteTypeData>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::metadata::get_quote_type(&state.cache, &symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Currency pairs available from Yahoo Finance.
    async fn currencies(&self, ctx: &Context<'_>) -> Result<Vec<GqlCurrency>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::metadata::get_currencies(&state.cache)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Supported stock exchanges with their symbol suffixes and data providers.
    async fn exchanges(&self, ctx: &Context<'_>) -> Result<Vec<GqlExchange>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::metadata::get_exchanges(&state.cache)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Upcoming financial events (earnings, dividends, options expirations,
    /// and major economic releases) across multiple symbols, merged and
    /// sorted ascending by timestamp.
    async fn calendar(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default_with = "GqlTimeRange::OneMonth")] range: GqlTimeRange,
    ) -> Result<Vec<GqlCalendarEvent>> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::calendar::get_calendar(
            &state.cache,
            refs,
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)?;
        let events: Vec<finance_query::CalendarEvent> =
            serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(events.into_iter().map(GqlCalendarEvent::from).collect())
    }

    /// A FRED economic data series (e.g. "FEDFUNDS", "CPIAUCSL"). Requires `FRED_API_KEY`.
    async fn fred_series(&self, ctx: &Context<'_>, id: String) -> Result<GqlMacroSeries> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::fred::get_series(&state.cache, &id)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// US Treasury yield curve rates for a calendar year (keyless).
    async fn treasury_yields(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Calendar year (default: current year)")] year: Option<i32>,
        #[graphql(desc = "Max rows to return; omitted = every matching row in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlTreasuryYield>> {
        let state = ctx.data::<AppState>()?;
        let year = year.unwrap_or_else(|| {
            chrono::Utc::now()
                .format("%Y")
                .to_string()
                .parse()
                .unwrap_or(2025)
        });
        let json = crate::services::fred::get_treasury_yields(&state.cache, year as u32)
            .await
            .map_err(to_gql_error)?;
        let rows: Vec<GqlTreasuryYield> =
            serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        pagination::paginate(rows, first, after).await
    }

    /// SEC EDGAR full-text search.
    #[allow(clippy::too_many_arguments)]
    async fn edgar_search(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default)] forms: Option<String>,
        #[graphql(default)] start_date: Option<String>,
        #[graphql(default)] end_date: Option<String>,
        #[graphql(desc = "Pagination offset (default: 0)")] from: Option<i32>,
        #[graphql(desc = "Page size (default: 100, max: 100)")] size: Option<i32>,
    ) -> Result<GqlEdgarSearchResults> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::edgar::search_edgar(
            &state.cache,
            &query,
            forms.as_deref(),
            start_date.as_deref(),
            end_date.as_deref(),
            from.map(|v| v.max(0) as usize),
            size.map(|v| v.max(0) as usize).or(Some(100)),
        )
        .await
        .map_err(to_gql_error)?;
        let results: finance_query::EdgarSearchResults =
            serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        let hits: Vec<GqlEdgarSearchHit> = results
            .hits
            .as_ref()
            .map(|h| {
                h.hits
                    .iter()
                    .filter_map(|hit| {
                        hit._source.as_ref().map(|src| GqlEdgarSearchHit {
                            file_date: src.file_date.clone(),
                            form: src.form.clone(),
                            adsh: src.adsh.clone(),
                            display_names: src.display_names.clone(),
                            ciks: src.ciks.clone(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let total_hits = results
            .hits
            .as_ref()
            .and_then(|h| h.total.as_ref())
            .and_then(|t| t.value)
            .map(|v| v as i64);
        let effective_from = from.map(|v| v.max(0) as usize).unwrap_or(0);
        let effective_size = hits.len();
        Ok(GqlEdgarSearchResults {
            total_hits,
            hits,
            page_info: Some(crate::graphql::pagination::offset_page_info(
                effective_from,
                effective_size,
                total_hits,
            )),
        })
    }
}
