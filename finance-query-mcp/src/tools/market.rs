use finance_query::{IndicesRegion, Industry, Region, Sector};
use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_FEAR_AND_GREED_DEFAULT_FIELDS, GQL_FEAR_AND_GREED_VALID_FIELDS,
    GQL_INDUSTRY_DEFAULT_FIELDS, GQL_INDUSTRY_VALID_FIELDS, GQL_MARKET_HOURS_VALID_FIELDS,
    GQL_MARKET_SUMMARY_DEFAULT_FIELDS, GQL_MARKET_SUMMARY_VALID_FIELDS, GQL_QUOTE_DEFAULT_FIELDS,
    GQL_QUOTE_VALID_FIELDS, GQL_SECTOR_DEFAULT_FIELDS, GQL_SECTOR_VALID_FIELDS,
    GQL_TRENDING_DEFAULT_FIELDS, GQL_TRENDING_VALID_FIELDS, INDUSTRY_COMPOSITE_FIELDS,
    MARKET_HOURS_COMPOSITE_FIELDS, SECTOR_COMPOSITE_FIELDS, build_selection_or_default,
    build_type_spec_selection, execute_query, parse_fields, unwrap_field,
};

/// Builds the parenthesized `marketSummary(...)` argument list from the
/// optional region/normalized-lang inputs; empty when neither is present
/// (a GraphQL field with no filled args takes no parens at all).
fn market_summary_args(region: Option<Region>, normalized_lang: Option<&str>) -> String {
    let mut args = Vec::new();
    if let Some(r) = region {
        args.push(format!(
            "region: \"{}\"",
            crate::tools::gql::escape_gql_string(r.region())
        ));
    }
    if let Some(l) = normalized_lang {
        args.push(format!("lang: \"{l}\""));
    }
    if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    }
}

pub async fn get_market_summary(
    schema: &FinanceSchema,
    region: Option<Region>,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_MARKET_SUMMARY_VALID_FIELDS,
        GQL_MARKET_SUMMARY_DEFAULT_FIELDS,
    );
    let normalized_lang = crate::lang::normalize(lang.as_deref());
    let args_str = market_summary_args(region, normalized_lang.as_deref());
    let query = format!("query {{ marketSummary{args_str} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "marketSummary");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_fear_and_greed(
    schema: &FinanceSchema,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_FEAR_AND_GREED_VALID_FIELDS,
        GQL_FEAR_AND_GREED_DEFAULT_FIELDS,
    );
    let query = format!("query {{ fearAndGreed {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "fearAndGreed");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_trending(
    schema: &FinanceSchema,
    region: Option<Region>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_TRENDING_VALID_FIELDS,
        GQL_TRENDING_DEFAULT_FIELDS,
    );
    let region_arg = region
        .map(|r| {
            format!(
                "(region: \"{}\")",
                crate::tools::gql::escape_gql_string(r.region())
            )
        })
        .unwrap_or_default();
    let query = format!("query {{ trending{region_arg} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "trending");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

/// Maps an `IndicesRegion` to its `GqlIndicesRegion` enum literal
/// (async-graphql renames enums SCREAMING_SNAKE_CASE).
fn indices_region_to_gql(region: IndicesRegion) -> &'static str {
    match region {
        IndicesRegion::Americas => "AMERICAS",
        IndicesRegion::Europe => "EUROPE",
        IndicesRegion::AsiaPacific => "ASIA_PACIFIC",
        IndicesRegion::MiddleEastAfrica => "MIDDLE_EAST_AFRICA",
        IndicesRegion::Currencies => "CURRENCIES",
    }
}

pub async fn get_indices(
    schema: &FinanceSchema,
    region: Option<IndicesRegion>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let args = region
        .map(|r| format!("(region: {})", indices_region_to_gql(r)))
        .unwrap_or_default();
    let field_list = parse_fields(fields);
    // `indices` returns `Vec<GqlQuote>` — same type as `quote`/`quotes`, so
    // reuse their existing allow-list/defaults rather than duplicating them.
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_QUOTE_VALID_FIELDS,
        GQL_QUOTE_DEFAULT_FIELDS,
    );
    let query = format!("query {{ indices{args} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "indices");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_market_hours(
    schema: &FinanceSchema,
    region: Option<Region>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_MARKET_HOURS_VALID_FIELDS,
        GQL_MARKET_HOURS_VALID_FIELDS,
        MARKET_HOURS_COMPOSITE_FIELDS,
    );
    let region_arg = region
        .map(|r| {
            format!(
                "(region: \"{}\")",
                crate::tools::gql::escape_gql_string(r.region())
            )
        })
        .unwrap_or_default();
    let query = format!("query {{ marketHours{region_arg} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "marketHours");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_sector(
    schema: &FinanceSchema,
    sector: Sector,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_SECTOR_VALID_FIELDS,
        GQL_SECTOR_DEFAULT_FIELDS,
        SECTOR_COMPOSITE_FIELDS,
    );
    let lang_arg = match crate::lang::normalize(lang.as_deref()) {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    let query = format!(
        "query {{ sector(sector: \"{}\"{}) {} }}",
        sector.as_api_path(),
        lang_arg,
        selection
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "sector");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_industry(
    schema: &FinanceSchema,
    industry: Industry,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_INDUSTRY_VALID_FIELDS,
        GQL_INDUSTRY_DEFAULT_FIELDS,
        INDUSTRY_COMPOSITE_FIELDS,
    );
    let lang_arg = match crate::lang::normalize(lang.as_deref()) {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    let query = format!(
        "query {{ industry(industry: \"{}\"{}) {} }}",
        industry.as_slug(),
        lang_arg,
        selection
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "industry");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_summary_args_empty_when_neither_present() {
        assert_eq!(market_summary_args(None, None), "");
    }

    #[test]
    fn market_summary_args_region_only() {
        assert_eq!(
            market_summary_args(Some(Region::UnitedStates), None),
            "(region: \"US\")"
        );
    }

    #[test]
    fn market_summary_args_lang_only() {
        assert_eq!(market_summary_args(None, Some("ja")), "(lang: \"ja\")");
    }

    #[test]
    fn market_summary_args_both_present() {
        assert_eq!(
            market_summary_args(Some(Region::UnitedStates), Some("ja")),
            "(region: \"US\", lang: \"ja\")"
        );
    }

    #[test]
    fn indices_region_to_gql_maps_every_variant() {
        assert_eq!(indices_region_to_gql(IndicesRegion::Americas), "AMERICAS");
        assert_eq!(indices_region_to_gql(IndicesRegion::Europe), "EUROPE");
        assert_eq!(
            indices_region_to_gql(IndicesRegion::AsiaPacific),
            "ASIA_PACIFIC"
        );
        assert_eq!(
            indices_region_to_gql(IndicesRegion::MiddleEastAfrica),
            "MIDDLE_EAST_AFRICA"
        );
        assert_eq!(
            indices_region_to_gql(IndicesRegion::Currencies),
            "CURRENCIES"
        );
    }
}
