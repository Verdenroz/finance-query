use finance_query::Screener;
use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_LOOKUP_RESULTS_DEFAULT_FIELDS, GQL_LOOKUP_RESULTS_VALID_FIELDS,
    GQL_SCREENER_RESULTS_DEFAULT_FIELDS, GQL_SCREENER_RESULTS_VALID_FIELDS,
    GQL_SEARCH_RESULTS_DEFAULT_FIELDS, GQL_SEARCH_RESULTS_VALID_FIELDS,
    LOOKUP_RESULTS_COMPOSITE_FIELDS, SCREENER_RESULTS_COMPOSITE_FIELDS,
    SEARCH_RESULTS_COMPOSITE_FIELDS, build_paginated_composite_selection,
    build_type_spec_selection, execute_query, parse_fields, unwrap_field, wrap_nested_connection,
};

fn lookup_type_to_gql(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "equity" | "stock" => "EQUITY",
        "etf" => "ETF",
        "mutualfund" | "mutual_fund" | "mutual-fund" => "MUTUAL_FUND",
        "index" => "INDEX",
        "future" => "FUTURE",
        "currency" | "forex" | "fx" => "CURRENCY",
        "crypto" | "cryptocurrency" => "CRYPTOCURRENCY",
        _ => "ALL",
    }
}

pub async fn search(
    schema: &FinanceSchema,
    query: String,
    lang: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let quotes_item_selection = SEARCH_RESULTS_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "quotes")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ symbol }");
    let fields_csv = field_list.as_ref().map(|fs| fs.join(","));
    let selection = build_paginated_composite_selection(
        fields_csv.as_deref(),
        GQL_SEARCH_RESULTS_VALID_FIELDS,
        GQL_SEARCH_RESULTS_DEFAULT_FIELDS,
        SEARCH_RESULTS_COMPOSITE_FIELDS,
        "quotes",
        quotes_item_selection,
        Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
        cursor.as_deref(),
    );
    let lang_arg = match crate::lang::normalize(lang.as_deref()) {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    let gql_query = format!(
        "query {{ search(query: \"{}\"{lang_arg}) {selection} }}",
        crate::tools::gql::escape_gql_string(&query)
    );
    let json = execute_query(schema, &gql_query, async_graphql::Variables::default()).await?;
    let data = wrap_nested_connection(unwrap_field(json, "search"), "quotes");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn screener(
    schema: &FinanceSchema,
    screener_type: String,
    count: Option<u32>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let s = screener_type.parse::<Screener>().map_err(|_| {
        invalid_params(format!(
            "Invalid screener: '{screener_type}'. Valid types: {}",
            Screener::valid_types()
        ))
    })?;
    let gql_type = s.as_scr_id().to_uppercase();
    let n = count.unwrap_or(25);
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_SCREENER_RESULTS_VALID_FIELDS,
        GQL_SCREENER_RESULTS_DEFAULT_FIELDS,
        SCREENER_RESULTS_COMPOSITE_FIELDS,
    );
    let query = format!("query {{ screener(type: {gql_type}, count: {n}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "screener");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_lookup(
    schema: &FinanceSchema,
    query: String,
    query_type: Option<String>,
    lang: Option<String>,
    fields: Option<String>,
    logo: Option<bool>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_LOOKUP_RESULTS_VALID_FIELDS,
        GQL_LOOKUP_RESULTS_DEFAULT_FIELDS,
        LOOKUP_RESULTS_COMPOSITE_FIELDS,
    );
    let gql_type = lookup_type_to_gql(query_type.as_deref().unwrap_or("all"));
    let lang_arg = match crate::lang::normalize(lang.as_deref()) {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    let logo_arg = if logo.unwrap_or(false) {
        ", logo: true".to_string()
    } else {
        String::new()
    };
    let gql_query = format!(
        "query {{ lookup(query: \"{}\", type: {gql_type}{lang_arg}{logo_arg}) {selection} }}",
        crate::tools::gql::escape_gql_string(&query)
    );
    let json = execute_query(schema, &gql_query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "lookup");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
