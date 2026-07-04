use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    ANALYSIS_TYPE_SPECS, DEFAULT_MCP_PAGE_SIZE, HOLDER_TYPE_SPECS,
    build_paginated_composite_selection, build_type_spec_selection, execute_query, parse_fields,
    unwrap_ticker_field, wrap_nested_connection,
};

fn holder_type_to_field(ht: &str) -> Option<&'static str> {
    match ht.to_lowercase().replace('-', "").as_str() {
        "major" => Some("majorHolders"),
        "institutional" => Some("institutionalHolders"),
        "mutualfund" => Some("mutualFundHolders"),
        "insidertransactions" => Some("insiderTransactions"),
        "insiderpurchases" => Some("insiderPurchases"),
        "insiderroster" => Some("insiderRoster"),
        _ => None,
    }
}

/// Which nested list field (if any) is paginated for a given holder GraphQL field.
fn holder_paginated_field(gql_field: &str) -> Option<&'static str> {
    match gql_field {
        "institutionalHolders" | "mutualFundHolders" => Some("ownershipList"),
        "insiderTransactions" => Some("transactions"),
        "insiderRoster" => Some("holders"),
        _ => None,
    }
}

pub async fn get_holders(
    schema: &FinanceSchema,
    symbol: String,
    holder_type: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_field = holder_type_to_field(&holder_type)
        .ok_or_else(|| invalid_params(format!("Invalid holder_type '{}'", holder_type)))?;
    let (_, valid_fields, default_fields, composite_fields) = HOLDER_TYPE_SPECS
        .iter()
        .find(|(n, ..)| *n == gql_field)
        .expect("holder_type_to_field only returns fields present in HOLDER_TYPE_SPECS");

    let field_list = parse_fields(fields);
    let paginated_field = holder_paginated_field(gql_field);
    let selection = match paginated_field {
        Some(pf) => {
            let item_selection = composite_fields
                .iter()
                .find(|(name, _)| *name == pf)
                .map(|(_, sel)| *sel)
                .unwrap_or("{ }");
            let fields_csv = field_list.as_ref().map(|fs| fs.join(","));
            build_paginated_composite_selection(
                fields_csv.as_deref(),
                valid_fields,
                default_fields,
                composite_fields,
                pf,
                item_selection,
                Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
                cursor.as_deref(),
            )
        }
        None => build_type_spec_selection(
            field_list.as_deref(),
            valid_fields,
            default_fields,
            composite_fields,
        ),
    };

    let query = format!(
        "query GetHolders($symbol: String!) {{ ticker(symbol: $symbol) {{ {gql_field} {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let mut data = unwrap_ticker_field(json, gql_field);
    if let Some(pf) = paginated_field {
        data = wrap_nested_connection(data, pf);
    }
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

fn analysis_type_to_field(at: &str) -> Option<&'static str> {
    match at.to_lowercase().replace('-', "").as_str() {
        "recommendations" => Some("recommendationTrend"),
        "upgradesdowngrades" => Some("gradingHistory"),
        "earningsestimate" => Some("earningsEstimate"),
        "earningshistory" => Some("earningsHistory"),
        _ => None,
    }
}

pub async fn get_analysis(
    schema: &FinanceSchema,
    symbol: String,
    analysis_type: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_field = analysis_type_to_field(&analysis_type)
        .ok_or_else(|| invalid_params(format!("Invalid analysis_type '{}'", analysis_type)))?;
    let (_, valid_fields, default_fields, composite_fields) = ANALYSIS_TYPE_SPECS
        .iter()
        .find(|(n, ..)| *n == gql_field)
        .expect("analysis_type_to_field only returns fields present in ANALYSIS_TYPE_SPECS");

    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        valid_fields,
        default_fields,
        composite_fields,
    );

    let query = format!(
        "query GetAnalysis($symbol: String!) {{ ticker(symbol: $symbol) {{ {gql_field} {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, gql_field);
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
