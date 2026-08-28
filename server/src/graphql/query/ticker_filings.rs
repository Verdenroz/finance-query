//! Per-symbol filings/disclosure fields routed via `Capability::FILINGS`:
//! congressional trading (FMP, falling back to keyless House PTR
//! disclosures) and fails-to-deliver (FMP, falling back to keyless EDGAR).
//! Distinct from Yahoo's `insiderTransactions`/`secFilings`
//! (`TickerHoldersQuery`/`TickerCoreQuery`) and SEC EDGAR's own
//! submissions/facts fields (`TickerAnalysisQuery`).

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::{from_gql_json, to_gql_error};
use crate::graphql::pagination::{self, Page};
use crate::graphql::types::filings::{
    GqlCongressionalTrade, GqlFailToDeliver, GqlFilingSection, GqlFilingSectionForm, GqlRiskFactor,
};

pub(super) struct TickerFilingsQuery {
    pub(super) symbol: String,
}

#[Object]
impl TickerFilingsQuery {
    /// Congressional trading disclosures for this symbol (FMP when
    /// `FMP_API_KEY` is set, falling back to keyless House PTR disclosures
    /// otherwise).
    async fn congressional_trades(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Max entries per page; omitted = every fetched disclosure in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlCongressionalTrade>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::filings::get_congressional_trades(
            &state.cache,
            &state.providers,
            &self.symbol,
        )
        .await
        .map_err(to_gql_error)?;
        let entries: Vec<GqlCongressionalTrade> = from_gql_json(json)?;
        pagination::paginate(&entries, first, after).await
    }

    /// Fails-to-deliver records for this symbol. Routes through FMP when
    /// `FMP_API_KEY` is set, EDGAR otherwise (keyless).
    async fn fails_to_deliver(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Max entries per page; omitted = every fetched record in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlFailToDeliver>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::filings::get_fails_to_deliver(
            &state.cache,
            &state.providers,
            &self.symbol,
        )
        .await
        .map_err(to_gql_error)?;
        let entries: Vec<GqlFailToDeliver> = from_gql_json(json)?;
        pagination::paginate(&entries, first, after).await
    }

    /// Sectioned text of one filing by accession number. Routes through EDGAR
    /// (best-effort HTML extraction) or Polygon when configured.
    async fn filing_sections(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "SEC accession number, e.g. \"0000320193-24-000123\"")]
        accession_number: String,
        #[graphql(desc = "Which filing form to fetch sectioned text for")]
        form: GqlFilingSectionForm,
    ) -> Result<Vec<GqlFilingSection>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::filings::get_filing_sections(
            &state.providers,
            &self.symbol,
            &accession_number,
            form.into(),
        )
        .await
        .map_err(to_gql_error)?;
        from_gql_json(json)
    }

    /// Risk factors extracted from this symbol's SEC filings. Routes through
    /// EDGAR (best-effort HTML extraction) or Polygon when configured.
    async fn risk_factors(&self, ctx: &Context<'_>) -> Result<Vec<GqlRiskFactor>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::filings::get_risk_factors(
            &state.cache,
            &state.providers,
            &self.symbol,
        )
        .await
        .map_err(to_gql_error)?;
        from_gql_json(json)
    }
}
