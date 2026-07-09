//! Discovery root query fields: search, lookup, and screeners.

use async_graphql::{Context, Object, Result};

use super::{resolve_gql_lang, screener_error_to_gql};
use crate::AppState;
use crate::graphql::error::{exec_gql, from_gql_json, to_gql_error};
use crate::graphql::types::{
    enums::{GqlLookupType, GqlScreener, GqlValueFormat},
    screener::{GqlCustomScreenerInput, GqlScreenerResults},
    search::{GqlLookupResults, GqlSearchResults},
};

#[derive(Default)]
pub(super) struct RootDiscoveryQuery;

#[Object]
impl RootDiscoveryQuery {
    /// Search for quotes, news, and research reports matching a query string.
    #[allow(clippy::too_many_arguments)]
    async fn search(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 6)] quotes: u32,
        #[graphql(default = 0)] news: u32,
        #[graphql(default)] fuzzy: bool,
        #[graphql(default = true)] logo: bool,
        #[graphql(default)] research: bool,
        #[graphql(default)] cultural: bool,
        #[graphql(desc = "Region code for localization (e.g. \"US\", \"JP\", \"GB\")")]
        region: Option<String>,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<GqlSearchResults> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let region = region
            .as_deref()
            .and_then(|s| s.parse::<finance_query::Region>().ok());
        exec_gql(crate::services::search::search(
            &state.cache,
            &query,
            quotes,
            news,
            crate::services::search::SearchFlags {
                fuzzy,
                logo,
                research,
                cultural,
            },
            region,
            lang.as_deref(),
        ))
        .await
    }

    /// Type-filtered symbol lookup (equity/ETF/mutual fund/index/future/currency/crypto).
    #[allow(clippy::too_many_arguments)]
    async fn lookup(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(name = "type", default)] lookup_type: GqlLookupType,
        #[graphql(default = 25)] count: u32,
        #[graphql(default)] logo: bool,
        #[graphql(desc = "Region code for localization (e.g. \"US\", \"JP\", \"GB\")")]
        region: Option<String>,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<GqlLookupResults> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let region = region
            .as_deref()
            .and_then(|s| s.parse::<finance_query::Region>().ok());
        exec_gql(crate::services::search::lookup(
            &state.cache,
            &query,
            lookup_type.into(),
            count,
            logo,
            region,
            lang.as_deref(),
        ))
        .await
    }

    /// Predefined stock screener results.
    async fn screener(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "type")] r#type: GqlScreener,
        #[graphql(default = 25)] count: u32,
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<GqlScreenerResults> {
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
        let json = finance_query::ValueFormat::from(format).transform(json);
        from_gql_json(json)
    }

    /// Custom stock/fund screener with flexible filter conditions.
    async fn custom_screener(
        &self,
        _ctx: &Context<'_>,
        input: GqlCustomScreenerInput,
        #[graphql(default)] format: GqlValueFormat,
    ) -> Result<GqlScreenerResults> {
        let quote_type = input
            .quote_type
            .as_deref()
            .and_then(|s| s.parse::<finance_query::QuoteType>().ok())
            .unwrap_or_default();
        let filters: Vec<crate::services::screener::FilterInput> = input
            .filters
            .into_iter()
            .map(|f| crate::services::screener::FilterInput {
                field: f.field,
                operator: f.operator,
                value: f.value.0,
            })
            .collect();

        let results = match quote_type {
            finance_query::QuoteType::MutualFund => {
                crate::services::screener::run_custom_fund_screener(
                    input.size,
                    input.offset,
                    input.sort_field.as_deref(),
                    input.sort_ascending,
                    &filters,
                )
                .await
            }
            _ => {
                crate::services::screener::run_custom_equity_screener(
                    input.size,
                    input.offset,
                    input.sort_field.as_deref(),
                    input.sort_ascending,
                    &filters,
                )
                .await
            }
        }
        .map_err(screener_error_to_gql)?;

        let json =
            serde_json::to_value(&results).map_err(|e| async_graphql::Error::new(e.to_string()))?;
        let json = finance_query::ValueFormat::from(format).transform(json);
        let mut gql_results: GqlScreenerResults = from_gql_json(json)?;
        gql_results.page_info = Some(crate::graphql::pagination::offset_page_info(
            input.offset as usize,
            input.size as usize,
            gql_results.total,
        ));
        Ok(gql_results)
    }
}
