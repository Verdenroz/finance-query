//! Core per-symbol fields: identity, quote, chart, news, options,
//! recommendations, and financial statements.

use async_graphql::{Context, Object, Result};

use super::{build_gql_options, resolve_gql_lang};
use crate::AppState;
use crate::graphql::error::{exec_gql, from_gql_json, to_gql_error};
use crate::graphql::pagination::{self, Page};
use crate::graphql::types::{
    chart::GqlChart,
    enums::{GqlFrequency, GqlInterval, GqlStatementType, GqlTimeRange, GqlValueFormat},
    financials::{GqlFinancialDataPoint, GqlFinancialLineItem},
    news::GqlNews,
    options::GqlOptions,
    quote::GqlQuote,
    recommendation::GqlRecommendation,
};

pub(super) struct TickerCoreQuery {
    pub(super) symbol: String,
}

#[Object]
impl TickerCoreQuery {
    async fn symbol(&self) -> &str {
        &self.symbol
    }

    async fn quote(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] logo: bool,
        #[graphql(default)] format: GqlValueFormat,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<GqlQuote> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let json =
            crate::services::quote::get_quote(&state.cache, &self.symbol, logo, lang.as_deref())
                .await
                .map_err(to_gql_error)?;
        let json = finance_query::ValueFormat::from(format).transform(json);
        from_gql_json(json)
    }

    // async-graphql requires each field to be an explicit function parameter, preventing a params struct
    #[allow(clippy::too_many_arguments)]
    async fn chart(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlInterval::OneDay")] interval: GqlInterval,
        #[graphql(default_with = "GqlTimeRange::OneMonth")] range: GqlTimeRange,
        #[graphql(
            desc = "Start date as Unix timestamp (seconds). When provided, overrides `range`."
        )]
        start: Option<i64>,
        #[graphql(
            desc = "End date as Unix timestamp (seconds). Defaults to now when `start` is set."
        )]
        end: Option<i64>,
        #[graphql(default)] events: bool,
        #[graphql(default)] patterns: bool,
    ) -> Result<GqlChart> {
        let state = ctx.data::<AppState>()?;
        if start.is_none() && end.is_some() {
            return Err(async_graphql::Error::new(
                "`end` requires `start` to be set",
            ));
        }

        exec_gql(crate::services::chart::get_chart(
            &state.cache,
            &self.symbol,
            interval.into(),
            range.into(),
            start,
            end,
            events,
            patterns,
        ))
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn news(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10, desc = "Overall cap on articles fetched")] count: u32,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
        #[graphql(
            desc = "Max entries per page; omitted = every fetched article (up to `count`) in one page"
        )]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlNews>> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let json = crate::services::news::get_news(
            &state.cache,
            &self.symbol,
            count as usize,
            lang.as_deref(),
        )
        .await
        .map_err(to_gql_error)?;
        let entries: Vec<GqlNews> = from_gql_json(json)?;
        pagination::paginate(&entries, first, after).await
    }

    async fn options(&self, ctx: &Context<'_>, date: Option<i64>) -> Result<GqlOptions> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::options::get_options(&state.cache, &self.symbol, date)
            .await
            .map_err(to_gql_error)?;
        let opts: finance_query::Options = from_gql_json(json)?;
        Ok(build_gql_options(opts))
    }

    /// Similar-stock recommendations for this symbol.
    async fn recommendations(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 5)] limit: u32,
    ) -> Result<GqlRecommendation> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_recommendations(
            &state.cache,
            &self.symbol,
            limit,
        ))
        .await
    }

    async fn financials(
        &self,
        ctx: &Context<'_>,
        statement: GqlStatementType,
        #[graphql(default_with = "GqlFrequency::Annual")] frequency: GqlFrequency,
        #[graphql(
            desc = "Filter to specific metric names (e.g. [\"TotalRevenue\",\"NetIncome\"]). Omitted = all metrics."
        )]
        metrics: Option<Vec<String>>,
    ) -> Result<Vec<GqlFinancialLineItem>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::financials::get_financials(
            &state.cache,
            &self.symbol,
            statement.into(),
            statement.as_str(),
            frequency.into(),
            frequency.as_str(),
        )
        .await
        .map_err(to_gql_error)?;
        let fs: finance_query::FinancialStatement = from_gql_json(json)?;
        let metric_set: Option<std::collections::HashSet<&str>> = metrics
            .as_ref()
            .map(|m| m.iter().map(|s| s.as_str()).collect());
        let items: Vec<GqlFinancialLineItem> = fs
            .statement
            .into_iter()
            .filter(|(metric, _)| {
                metric_set
                    .as_ref()
                    .map(|s| s.contains(metric.as_str()))
                    .unwrap_or(true)
            })
            .map(|(metric, dates)| GqlFinancialLineItem {
                metric,
                values: dates
                    .into_iter()
                    .map(|(date, value)| GqlFinancialDataPoint { date, value })
                    .collect(),
            })
            .collect();
        Ok(items)
    }
}
