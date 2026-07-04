//! Single-ticker entry point and batch (multi-symbol) root query fields.

use async_graphql::{Context, Object, Result};

use super::{GqlTicker, build_gql_options, extract_batch_errors, resolve_gql_lang};
use crate::AppState;
use crate::graphql::error::to_gql_error;
use crate::graphql::types::{
    chart::{GqlChart, GqlChartsBatch, GqlSpark, GqlSparkBatch, GqlSymbolChart},
    enums::{GqlFrequency, GqlInterval, GqlStatementType, GqlTimeRange, GqlValueFormat},
    events::{
        GqlCapitalGain, GqlCapitalGainsBatch, GqlDividend, GqlDividendsBatch, GqlSplit,
        GqlSplitsBatch, GqlSymbolCapitalGains, GqlSymbolDividends, GqlSymbolSplits,
    },
    financials::{
        GqlFinancialDataPoint, GqlFinancialLineItem, GqlFinancialsBatch, GqlSymbolFinancials,
    },
    indicators::{GqlIndicatorsBatch, GqlIndicatorsSummary, GqlSymbolIndicators},
    options::GqlOptionsBatch,
    options::GqlSymbolOptions,
    quote::{GqlQuote, GqlQuotesBatch},
    recommendation::{GqlRecommendation, GqlRecommendationsBatch},
};

#[derive(Default)]
pub(super) struct RootBatchQuery;

#[Object]
impl RootBatchQuery {
    /// Fetch data for a single ticker symbol.
    async fn ticker(&self, symbol: String) -> GqlTicker {
        GqlTicker::new(symbol)
    }

    /// Batch quotes: one upstream call for multiple symbols.
    async fn quotes(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default)] logo: bool,
        #[graphql(default)] format: GqlValueFormat,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<GqlQuotesBatch> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::quote::get_quotes(&state.cache, refs, logo, lang.as_deref())
            .await
            .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let quotes_map = json
            .get("quotes")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let lib_format = finance_query::ValueFormat::from(format);
        let mut quotes = Vec::with_capacity(quotes_map.len());
        for (_, v) in quotes_map {
            let v = lib_format.transform(v);
            let q: GqlQuote =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            quotes.push(q);
        }
        Ok(GqlQuotesBatch { quotes, errors })
    }

    /// Batch charts: one upstream call for multiple symbols.
    async fn charts(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default_with = "GqlInterval::OneDay")] interval: GqlInterval,
        #[graphql(default_with = "GqlTimeRange::OneMonth")] range: GqlTimeRange,
        #[graphql(default)] patterns: bool,
    ) -> Result<GqlChartsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::chart::get_batch_charts(
            &state.cache,
            refs,
            interval.into(),
            range.into(),
            patterns,
        )
        .await
        .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let charts_map = json
            .get("charts")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut charts = Vec::with_capacity(charts_map.len());
        for (symbol, v) in charts_map {
            let chart: GqlChart =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            charts.push(GqlSymbolChart { symbol, chart });
        }
        Ok(GqlChartsBatch { charts, errors })
    }

    /// Batch sparkline data: one upstream call for multiple symbols, close
    /// prices only — optimized for lightweight sparkline rendering.
    async fn spark(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default_with = "GqlInterval::OneDay")] interval: GqlInterval,
        #[graphql(default_with = "GqlTimeRange::OneMonth")] range: GqlTimeRange,
    ) -> Result<GqlSparkBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json =
            crate::services::chart::get_spark(&state.cache, refs, interval.into(), range.into())
                .await
                .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let sparks_map = json
            .get("sparks")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut sparks = Vec::with_capacity(sparks_map.len());
        for (_, v) in sparks_map {
            let spark: GqlSpark =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            sparks.push(spark);
        }
        Ok(GqlSparkBatch { sparks, errors })
    }

    /// Batch options chains: one upstream call for multiple symbols.
    async fn options_batch(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        date: Option<i64>,
    ) -> Result<GqlOptionsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::options::get_batch_options(&state.cache, refs, date)
            .await
            .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let options_map = json
            .get("options")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut options = Vec::with_capacity(options_map.len());
        for (symbol, v) in options_map {
            let opts: finance_query::Options =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            options.push(GqlSymbolOptions {
                symbol,
                options: build_gql_options(opts),
            });
        }
        Ok(GqlOptionsBatch { options, errors })
    }

    /// Batch stock splits: one upstream call for multiple symbols.
    async fn splits_batch(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: GqlTimeRange,
    ) -> Result<GqlSplitsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::events::get_batch_splits(
            &state.cache,
            refs,
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let splits_map = json
            .get("splits")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut splits = Vec::with_capacity(splits_map.len());
        for (symbol, v) in splits_map {
            let entries: Vec<GqlSplit> =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            splits.push(GqlSymbolSplits {
                symbol,
                splits: entries,
            });
        }
        Ok(GqlSplitsBatch { splits, errors })
    }

    /// Batch capital gains: one upstream call for multiple symbols.
    async fn capital_gains_batch(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: GqlTimeRange,
    ) -> Result<GqlCapitalGainsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::events::get_batch_capital_gains(
            &state.cache,
            refs,
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let cg_map = json
            .get("capitalGains")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut capital_gains = Vec::with_capacity(cg_map.len());
        for (symbol, v) in cg_map {
            let entries: Vec<GqlCapitalGain> =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            capital_gains.push(GqlSymbolCapitalGains {
                symbol,
                capital_gains: entries,
            });
        }
        Ok(GqlCapitalGainsBatch {
            capital_gains,
            errors,
        })
    }

    /// Batch dividends: one upstream call for multiple symbols.
    async fn dividends_batch(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: GqlTimeRange,
    ) -> Result<GqlDividendsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::events::get_batch_dividends(
            &state.cache,
            refs,
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let dividends_map = json
            .get("dividends")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut dividends = Vec::with_capacity(dividends_map.len());
        for (symbol, v) in dividends_map {
            let entries: Vec<GqlDividend> =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            dividends.push(GqlSymbolDividends {
                symbol,
                dividends: entries,
            });
        }
        Ok(GqlDividendsBatch { dividends, errors })
    }

    /// Batch indicators: one upstream call for multiple symbols.
    async fn indicators_batch(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        interval: GqlInterval,
        range: GqlTimeRange,
    ) -> Result<GqlIndicatorsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::indicators::get_batch_indicators(
            &state.cache,
            refs,
            interval.into(),
            interval.as_str(),
            range.into(),
            range.as_str(),
        )
        .await
        .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let indicators_map = json
            .get("indicators")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut indicators = Vec::with_capacity(indicators_map.len());
        for (symbol, v) in indicators_map {
            let flags: GqlIndicatorsSummary =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            indicators.push(GqlSymbolIndicators {
                symbol,
                indicators: flags,
            });
        }
        Ok(GqlIndicatorsBatch { indicators, errors })
    }

    /// Batch similar-stock recommendations: one upstream call for multiple symbols.
    async fn recommendations_batch(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        #[graphql(default = 5)] limit: u32,
    ) -> Result<GqlRecommendationsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::analysis::get_batch_recommendations(&state.cache, refs, limit)
            .await
            .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let recs_map = json
            .get("recommendations")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut recommendations = Vec::with_capacity(recs_map.len());
        for (_, v) in recs_map {
            let rec: GqlRecommendation =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
            recommendations.push(rec);
        }
        Ok(GqlRecommendationsBatch {
            recommendations,
            errors,
        })
    }

    /// Batch financial statements: one upstream call for multiple symbols.
    async fn financials_batch(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
        statement: GqlStatementType,
        #[graphql(default_with = "GqlFrequency::Annual")] frequency: GqlFrequency,
        #[graphql(
            desc = "Filter to specific metric names (e.g. [\"TotalRevenue\",\"NetIncome\"]). Omitted = all metrics."
        )]
        metrics: Option<Vec<String>>,
    ) -> Result<GqlFinancialsBatch> {
        let state = ctx.data::<AppState>()?;
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let json = crate::services::financials::get_batch_financials(
            &state.cache,
            refs,
            statement.into(),
            statement.as_str(),
            frequency.into(),
            frequency.as_str(),
        )
        .await
        .map_err(to_gql_error)?;

        let errors = extract_batch_errors(&json);
        let metric_set: Option<std::collections::HashSet<&str>> = metrics
            .as_ref()
            .map(|m| m.iter().map(|s| s.as_str()).collect());
        let financials_map = json
            .get("financials")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let mut financials = Vec::with_capacity(financials_map.len());
        for (symbol, v) in financials_map {
            let fs: finance_query::FinancialStatement =
                serde_json::from_value(v).map_err(|e| async_graphql::Error::new(e.to_string()))?;
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
            financials.push(GqlSymbolFinancials {
                symbol,
                statement: items,
            });
        }
        Ok(GqlFinancialsBatch { financials, errors })
    }
}
