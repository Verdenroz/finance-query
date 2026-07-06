use finance_query::{IndicesRegion, Sector};
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

pub async fn get_market_summary(
    schema: &FinanceSchema,
    region: Option<String>,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_MARKET_SUMMARY_VALID_FIELDS,
        GQL_MARKET_SUMMARY_DEFAULT_FIELDS,
    );
    let mut args = Vec::new();
    if let Some(r) = region.as_deref().filter(|r| !r.is_empty()) {
        args.push(format!(
            "region: \"{}\"",
            crate::tools::gql::escape_gql_string(r)
        ));
    }
    if let Some(l) = crate::lang::normalize(lang.as_deref()) {
        args.push(format!("lang: \"{l}\""));
    }
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
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
    region: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_TRENDING_VALID_FIELDS,
        GQL_TRENDING_DEFAULT_FIELDS,
    );
    let region_arg = region
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| format!("(region: \"{}\")", crate::tools::gql::escape_gql_string(r)))
        .unwrap_or_default();
    let query = format!("query {{ trending{region_arg} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "trending");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_indices(
    schema: &FinanceSchema,
    region: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    // `region` is a `GqlIndicesRegion` enum, spliced as a literal (async-graphql
    // renames enums SCREAMING_SNAKE_CASE — same as `financials.rs`'s statement type).
    let gql_region = region
        .as_deref()
        .and_then(|s| s.parse::<IndicesRegion>().ok())
        .map(|r| match r {
            IndicesRegion::Americas => "AMERICAS",
            IndicesRegion::Europe => "EUROPE",
            IndicesRegion::AsiaPacific => "ASIA_PACIFIC",
            IndicesRegion::MiddleEastAfrica => "MIDDLE_EAST_AFRICA",
            IndicesRegion::Currencies => "CURRENCIES",
        });
    let args = gql_region
        .map(|r| format!("(region: {r})"))
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
    region: Option<String>,
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
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| format!("(region: \"{}\")", crate::tools::gql::escape_gql_string(r)))
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
    sector: String,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    // Validating against the `Sector` enum before use means only an exact,
    // known canonical slug can ever reach the query text — safe to splice.
    let _s: Sector = sector.parse().map_err(|_| {
        crate::error::invalid_params(format!(
            "Invalid sector: '{sector}'. Valid types: {}",
            Sector::valid_types()
        ))
    })?;
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
        sector, lang_arg, selection
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "sector");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_industry(
    schema: &FinanceSchema,
    industry: String,
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
    // `Industry` has no `FromStr` in the library (unlike `Sector`), so unlike
    // get_sector this can't be validated against a known-slug allow-list —
    // escape instead to prevent breaking out of the string literal.
    let query = format!(
        "query {{ industry(industry: \"{}\"{}) {} }}",
        crate::tools::gql::escape_gql_string(&industry),
        lang_arg,
        selection
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "industry");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
