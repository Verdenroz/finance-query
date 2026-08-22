use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query::Sector;
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_INDUSTRY_VALID_FIELDS, GQL_SECTOR_VALID_FIELDS, INDUSTRY_COMPOSITE_FIELDS,
        SECTOR_COMPOSITE_FIELDS, unwrap_field,
    },
};
use finance_query_server::lang;
use finance_query_server::params::SectorQuery;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};
use super::support::parse_format;

/// GET /v2/sectors/{sector}
///
/// Query: `format` (raw|pretty|both), `fields` (comma-separated)
pub(crate) async fn get_sector(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(sector): Path<Sector>,
    Query(params): Query<SectorQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let format = parse_format(params.format);
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let gql_format = match format {
        finance_query::ValueFormat::Raw => "RAW",
        finance_query::ValueFormat::Pretty => "PRETTY",
        finance_query::ValueFormat::Both => "BOTH",
    };
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_SECTOR_VALID_FIELDS,
        SECTOR_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetSector($sector: String!, $lang: String) {{ sector(sector: $sector, lang: $lang, format: {gql_format}) {selection} }}"
    );
    info!(
        "Fetching {:?} sector (format={}, fields={:?})",
        sector,
        format.as_str(),
        params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("sector"), sector.as_api_path().into());
    if let Some(l) = &lang {
        vars.insert(Name::new("lang"), l.clone().into());
    }
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "sector"))).into_response()
}

/// GET /v2/industries/{industry}
pub(crate) async fn get_industry(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(industry): Path<finance_query::Industry>,
    Query(params): Query<SectorQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let format = parse_format(params.format);
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let gql_format = match format {
        finance_query::ValueFormat::Raw => "RAW",
        finance_query::ValueFormat::Pretty => "PRETTY",
        finance_query::ValueFormat::Both => "BOTH",
    };
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_INDUSTRY_VALID_FIELDS,
        INDUSTRY_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetIndustry($industry: String!, $lang: String) {{ industry(industry: $industry, lang: $lang, format: {gql_format}) {selection} }}"
    );
    info!(
        "Fetching {:?} industry (fields={:?})",
        industry, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("industry"), industry.as_slug().into());
    if let Some(l) = &lang {
        vars.insert(Name::new("lang"), l.clone().into());
    }
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "industry"))).into_response()
}
