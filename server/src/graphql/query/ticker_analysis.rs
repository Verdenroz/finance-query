//! Per-symbol analyst and disclosure fields: recommendation trend, grading
//! history, earnings estimates/history, SEC EDGAR facts/submissions, and
//! earnings call transcripts.

use async_graphql::{Context, Object, Result};

use super::resolve_gql_lang;
use crate::AppState;
use crate::graphql::error::{exec_gql, from_gql_json, to_gql_error};
use crate::graphql::types::{
    analysis::{
        GqlEarningsHistory, GqlEarningsTrend, GqlRecommendationTrend, GqlUpgradeDowngradeHistory,
    },
    edgar::{GqlEdgarFiling, GqlEdgarSubmissions, GqlFactConcept, GqlFactDataPoint},
    transcript::GqlTranscriptWithMeta,
};

pub(super) struct TickerAnalysisQuery {
    pub(super) symbol: String,
}

#[Object]
impl TickerAnalysisQuery {
    async fn recommendation_trend(&self, ctx: &Context<'_>) -> Result<GqlRecommendationTrend> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_recommendation_trend(
            &state.cache,
            &self.symbol,
        ))
        .await
    }

    async fn grading_history(&self, ctx: &Context<'_>) -> Result<GqlUpgradeDowngradeHistory> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_grading_history(
            &state.cache,
            &self.symbol,
        ))
        .await
    }

    async fn earnings_estimate(&self, ctx: &Context<'_>) -> Result<GqlEarningsTrend> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_earnings_trend(
            &state.cache,
            &self.symbol,
        ))
        .await
    }

    async fn earnings_history(&self, ctx: &Context<'_>) -> Result<GqlEarningsHistory> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_earnings_history(
            &state.cache,
            &self.symbol,
        ))
        .await
    }

    /// SEC EDGAR company facts (XBRL) for this symbol.
    async fn edgar_facts(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            desc = "XBRL taxonomy (default: us-gaap). Also try ifrs-full or dei.",
            default_with = "\"us-gaap\".to_string()"
        )]
        taxonomy: String,
        #[graphql(
            desc = "Filter to specific concepts. Omitted = curated defaults (headline financials)."
        )]
        concepts: Option<Vec<String>>,
    ) -> Result<Vec<GqlFactConcept>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::edgar::get_facts(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        let facts: finance_query::CompanyFacts = from_gql_json(json)?;
        let taxonomy_facts = facts.facts.get(&taxonomy);
        let default_concepts: &[&str] = &[
            "Revenues",
            "CostOfRevenue",
            "GrossProfit",
            "OperatingIncomeLoss",
            "NetIncomeLoss",
            "EarningsPerShareBasic",
            "EarningsPerShareDiluted",
            "Assets",
            "AssetsCurrent",
            "Liabilities",
            "LiabilitiesCurrent",
            "StockholdersEquity",
            "CashAndCashEquivalentsAtCarryingValue",
            "LongTermDebtNoncurrent",
            "NetCashProvidedByUsedInOperatingActivities",
            "NetCashProvidedByUsedInInvestingActivities",
            "NetCashProvidedByUsedInFinancingActivities",
        ];
        let concept_set: Vec<&str> = match &concepts {
            Some(c) => c.iter().map(|s| s.as_str()).collect(),
            None => default_concepts.to_vec(),
        };
        let mut result = Vec::new();
        let Some(tf) = taxonomy_facts else {
            return Ok(result);
        };
        for concept_name in &concept_set {
            let Some(fc) = tf.0.get(*concept_name) else {
                continue;
            };
            for (unit, data_points) in &fc.units {
                result.push(GqlFactConcept {
                    concept: concept_name.to_string(),
                    label: fc.label.clone(),
                    description: fc.description.clone(),
                    taxonomy: taxonomy.clone(),
                    unit: unit.clone(),
                    data_points: data_points
                        .iter()
                        .map(|dp| GqlFactDataPoint {
                            start: dp.start.clone(),
                            end: dp.end.clone(),
                            val: dp.val,
                            accn: dp.accn.clone(),
                            fy: dp.fy,
                            fp: dp.fp.clone(),
                            form: dp.form.clone(),
                            filed: dp.filed.clone(),
                            frame: dp.frame.clone(),
                        })
                        .collect(),
                });
            }
        }
        Ok(result)
    }

    /// SEC EDGAR filing submissions for this symbol.
    async fn edgar_submissions(&self, ctx: &Context<'_>) -> Result<GqlEdgarSubmissions> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::edgar::get_submissions(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        let submissions: finance_query::EdgarSubmissions = from_gql_json(json)?;
        let filings: Vec<GqlEdgarFiling> = submissions
            .filings
            .as_ref()
            .and_then(|f| f.recent.as_ref())
            .map(|r| {
                r.to_filings()
                    .into_iter()
                    .map(|f| GqlEdgarFiling {
                        accession_number: f.accession_number,
                        filing_date: f.filing_date,
                        report_date: f.report_date,
                        form: f.form,
                        size: f.size as i64,
                        primary_document: f.primary_document,
                        primary_doc_description: f.primary_doc_description,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(GqlEdgarSubmissions {
            cik: submissions.cik,
            name: submissions.name,
            tickers: submissions.tickers,
            exchanges: submissions.exchanges,
            sic: submissions.sic,
            sic_description: submissions.sic_description,
            fiscal_year_end: submissions.fiscal_year_end,
            category: submissions.category,
            ein: submissions.ein,
            entity_type: submissions.entity_type,
            state_of_incorporation: submissions.state_of_incorporation,
            website: submissions.website,
            insider_transaction_for_owner_exists: submissions.insider_transaction_for_owner_exists,
            insider_transaction_for_issuer_exists: submissions
                .insider_transaction_for_issuer_exists,
            filings,
        })
    }

    /// Latest earnings call transcript for this symbol (or a specific quarter/year).
    async fn transcript(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Fiscal quarter (Q1, Q2, Q3, Q4). Omitted = latest.")] quarter: Option<
            String,
        >,
        #[graphql(desc = "Fiscal year. Omitted with quarter = latest.")] year: Option<i32>,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<GqlTranscriptWithMeta> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        exec_gql(crate::services::transcripts::get_transcript(
            &state.cache,
            &self.symbol,
            quarter.as_deref(),
            year,
            lang.as_deref(),
        ))
        .await
    }

    /// Earnings call transcripts for this symbol.
    async fn transcripts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default)] limit: Option<i32>,
        #[graphql(desc = "Target language for translated text fields (BCP 47)")] lang: Option<
            String,
        >,
    ) -> Result<Vec<GqlTranscriptWithMeta>> {
        let state = ctx.data::<AppState>()?;
        let lang = resolve_gql_lang(lang.as_deref());
        exec_gql(crate::services::transcripts::get_transcripts(
            &state.cache,
            &self.symbol,
            limit.map(|l| l as usize),
            lang.as_deref(),
        ))
        .await
    }
}
