use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    EDGAR_FACTS_COMPOSITE_FIELDS, GQL_EDGAR_FACTS_DEFAULT_FIELDS, GQL_EDGAR_FACTS_VALID_FIELDS,
    build_type_spec_selection, escape_gql_string, execute_query, gql_string_list_literal,
    parse_fields, unwrap_field, unwrap_ticker_field,
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
) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_EDGAR_FACTS_VALID_FIELDS,
        GQL_EDGAR_FACTS_DEFAULT_FIELDS,
        EDGAR_FACTS_COMPOSITE_FIELDS,
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
    let data = unwrap_ticker_field(json, "edgarFacts");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_edgar_submissions(
    schema: &FinanceSchema,
    symbol: String,
) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let query = "query GetEdgarSubmissions($symbol: String!) { ticker(symbol: $symbol) { edgarSubmissions { cik name tickers exchanges sic sicDescription fiscalYearEnd category filings { accessionNumber filingDate reportDate form size primaryDocument primaryDocDescription } } } }";
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, query, variables).await?;
    let data = unwrap_ticker_field(json, "edgarSubmissions");
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
) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let gql_query = "query GetEdgarSearch($query: String!, $forms: String, $startDate: String, $endDate: String) { edgarSearch(query: $query, forms: $forms, startDate: $startDate, endDate: $endDate) { totalHits hits { fileDate form adsh displayNames ciks } } }";
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
    let json = execute_query(schema, gql_query, variables).await?;
    let data = unwrap_field(json, "edgarSearch");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
