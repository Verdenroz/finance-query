use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, EDGAR_FACTS_COMPOSITE_FIELDS, EDGAR_SUBMISSIONS_COMPOSITE_FIELDS,
    GQL_EDGAR_FACTS_DEFAULT_FIELDS, GQL_EDGAR_FACTS_VALID_FIELDS,
    GQL_EDGAR_SUBMISSIONS_VALID_FIELDS, build_paginated_composite_selection, escape_gql_string,
    execute_query, gql_string_list_literal, parse_fields, unwrap_field, unwrap_ticker_field,
    wrap_nested_connection,
};
fn edgar_guard() -> Result<(), McpError> {
    if std::env::var("EDGAR_EMAIL").is_err() {
        return Err(invalid_params(
            "EDGAR not configured — set the EDGAR_EMAIL environment variable to enable SEC EDGAR tools",
        ));
    }
    Ok(())
}

pub async fn get_edgar_facts(
    schema: &FinanceSchema,
    symbol: String,
    taxonomy: Option<String>,
    concepts: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let field_list = parse_fields(fields);
    let data_points_item_selection = EDGAR_FACTS_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "dataPoints")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ end val fy fp form }");
    let fields_csv = field_list.as_ref().map(|fs| fs.join(","));
    let selection = build_paginated_composite_selection(
        fields_csv.as_deref(),
        GQL_EDGAR_FACTS_VALID_FIELDS,
        GQL_EDGAR_FACTS_DEFAULT_FIELDS,
        EDGAR_FACTS_COMPOSITE_FIELDS,
        "dataPoints",
        data_points_item_selection,
        Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
        cursor.as_deref(),
    );

    let taxonomy_arg = match &taxonomy {
        Some(t) if !t.trim().is_empty() => format!("taxonomy: \"{}\"", escape_gql_string(t)),
        _ => String::new(),
    };
    let concept_list = parse_fields(concepts);
    let concepts_arg = match &concept_list {
        Some(cs) if !cs.is_empty() => format!("concepts: [{}]", gql_string_list_literal(cs)),
        _ => String::new(),
    };
    let args: Vec<&str> = [taxonomy_arg.as_str(), concepts_arg.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };

    let query = format!(
        "query GetEdgarFacts($symbol: String!) {{ ticker(symbol: $symbol) {{ edgarFacts{args_str} {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let mut concepts = unwrap_ticker_field(json, "edgarFacts");
    if let Some(list) = concepts.as_array_mut() {
        for concept in list.iter_mut() {
            *concept = wrap_nested_connection(concept.take(), "dataPoints");
        }
    }
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&concepts).map_err(ser_err)?,
    )]))
}

pub async fn get_edgar_submissions(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let field_list = parse_fields(fields);
    let filings_item_selection = EDGAR_SUBMISSIONS_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "filings")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ accessionNumber filingDate reportDate form size primaryDocument primaryDocDescription }");
    let fields_csv = field_list.as_ref().map(|fs| fs.join(","));
    let selection = build_paginated_composite_selection(
        fields_csv.as_deref(),
        GQL_EDGAR_SUBMISSIONS_VALID_FIELDS,
        GQL_EDGAR_SUBMISSIONS_VALID_FIELDS,
        EDGAR_SUBMISSIONS_COMPOSITE_FIELDS,
        "filings",
        filings_item_selection,
        Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
        cursor.as_deref(),
    );
    let query = format!(
        "query GetEdgarSubmissions($symbol: String!) {{ ticker(symbol: $symbol) {{ edgarSubmissions {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_nested_connection(unwrap_ticker_field(json, "edgarSubmissions"), "filings");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_edgar_search(
    schema: &FinanceSchema,
    query: String,
    forms: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    from: Option<u32>,
    size: Option<u32>,
) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let gql_query = "query GetEdgarSearch($query: String!, $forms: String, $startDate: String, $endDate: String, $from: Int, $size: Int) { edgarSearch(query: $query, forms: $forms, startDate: $startDate, endDate: $endDate, from: $from, size: $size) { totalHits hits { fileDate form adsh displayNames ciks } pageInfo { hasNextPage hasPreviousPage startCursor endCursor } } }";
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("query"), query.into());
    if let Some(f) = forms.filter(|f| !f.trim().is_empty()) {
        variables.insert(async_graphql::Name::new("forms"), f.into());
    }
    if let Some(d) = start_date.filter(|d| !d.is_empty()) {
        variables.insert(async_graphql::Name::new("startDate"), d.into());
    }
    if let Some(d) = end_date.filter(|d| !d.is_empty()) {
        variables.insert(async_graphql::Name::new("endDate"), d.into());
    }
    if let Some(from) = from {
        variables.insert(async_graphql::Name::new("from"), (from as i64).into());
    }
    if let Some(size) = size {
        variables.insert(async_graphql::Name::new("size"), (size as i64).into());
    }
    let json = execute_query(schema, gql_query, variables).await?;
    let data = unwrap_field(json, "edgarSearch");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
