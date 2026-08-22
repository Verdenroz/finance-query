use finance_query_server::graphql::FinanceSchema;
use finance_query_server::params::{AnalysisType, HolderType, Quarter};
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    ANALYSIS_TYPE_SPECS, DEFAULT_MCP_PAGE_SIZE, GQL_COMPANY_PROFILE_DEFAULT_FIELDS,
    GQL_COMPANY_PROFILE_VALID_FIELDS, GQL_EARNINGS_SURPRISES_COMPOSITE,
    GQL_EARNINGS_SURPRISES_DEFAULT_FIELDS, GQL_EARNINGS_SURPRISES_VALID_FIELDS,
    GQL_EARNINGS_TRANSCRIPT_DEFAULT_FIELDS, GQL_EARNINGS_TRANSCRIPT_VALID_FIELDS,
    HOLDER_TYPE_SPECS, build_paginated_composite_selection, build_selection_or_default,
    build_type_spec_selection, execute_query, parse_fields, unwrap_ticker_field,
    wrap_nested_connection,
};

fn holder_type_to_field(holder_type: HolderType) -> &'static str {
    match holder_type {
        HolderType::Major => "majorHolders",
        HolderType::Institutional => "institutionalHolders",
        HolderType::MutualFund => "mutualFundHolders",
        HolderType::InsiderTransactions => "insiderTransactions",
        HolderType::InsiderPurchases => "insiderPurchases",
        HolderType::InsiderRoster => "insiderRoster",
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

/// Build the `<gqlField> { ... }` selection for `get_holders`: delegates to
/// the paginated-composite builder when the holder type has a paginated
/// nested list, otherwise falls back to the plain type-spec builder.
#[allow(clippy::too_many_arguments)]
fn build_holders_selection(
    field_list: Option<&[String]>,
    valid_fields: &[&str],
    default_fields: &[&str],
    composite_fields: &[(&str, &str)],
    paginated_field: Option<&str>,
    limit: u32,
    cursor: Option<&str>,
) -> String {
    match paginated_field {
        Some(pf) => {
            let item_selection = composite_fields
                .iter()
                .find(|(name, _)| *name == pf)
                .map(|(_, sel)| *sel)
                .unwrap_or("{ }");
            let fields_csv = field_list.map(|fs| fs.join(","));
            build_paginated_composite_selection(
                fields_csv.as_deref(),
                valid_fields,
                default_fields,
                composite_fields,
                pf,
                item_selection,
                Some(limit),
                cursor,
            )
        }
        None => {
            build_type_spec_selection(field_list, valid_fields, default_fields, composite_fields)
        }
    }
}

pub async fn get_holders(
    schema: &FinanceSchema,
    symbol: String,
    holder_type: HolderType,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_field = holder_type_to_field(holder_type);
    let (_, valid_fields, default_fields, composite_fields) = HOLDER_TYPE_SPECS
        .iter()
        .find(|(n, ..)| *n == gql_field)
        .expect("holder_type_to_field only returns fields present in HOLDER_TYPE_SPECS");

    let field_list = parse_fields(fields);
    let paginated_field = holder_paginated_field(gql_field);
    let selection = build_holders_selection(
        field_list.as_deref(),
        valid_fields,
        default_fields,
        composite_fields,
        paginated_field,
        limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE),
        cursor.as_deref(),
    );

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

fn analysis_type_to_field(analysis_type: AnalysisType) -> &'static str {
    match analysis_type {
        AnalysisType::Recommendations => "recommendationTrend",
        AnalysisType::UpgradesDowngrades => "gradingHistory",
        AnalysisType::EarningsEstimate => "earningsEstimate",
        AnalysisType::EarningsHistory => "earningsHistory",
    }
}

pub async fn get_analysis(
    schema: &FinanceSchema,
    symbol: String,
    analysis_type: AnalysisType,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_field = analysis_type_to_field(analysis_type);
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

pub async fn get_company_profile(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_COMPANY_PROFILE_VALID_FIELDS,
        GQL_COMPANY_PROFILE_DEFAULT_FIELDS,
    );

    let query = format!(
        "query GetCompanyProfile($symbol: String!) {{ ticker(symbol: $symbol) {{ companyProfile {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "companyProfile");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_earnings_surprises(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_EARNINGS_SURPRISES_VALID_FIELDS,
        GQL_EARNINGS_SURPRISES_DEFAULT_FIELDS,
        &[("surprises", GQL_EARNINGS_SURPRISES_COMPOSITE)],
    );

    let query = format!(
        "query GetEarningsSurprises($symbol: String!) {{ ticker(symbol: $symbol) {{ earningsSurprises {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "earningsSurprises");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_earnings_transcript(
    schema: &FinanceSchema,
    symbol: String,
    quarter: Option<Quarter>,
    year: Option<i32>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_EARNINGS_TRANSCRIPT_VALID_FIELDS,
        GQL_EARNINGS_TRANSCRIPT_DEFAULT_FIELDS,
    );
    let quarter_arg = quarter.map(|q| format!("quarter: \"{}\"", q.as_str()));
    let year_arg = year.map(|y| format!("year: {y}"));
    let args: Vec<String> = [quarter_arg, year_arg].into_iter().flatten().collect();
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };

    let query = format!(
        "query GetEarningsTranscript($symbol: String!) {{ ticker(symbol: $symbol) {{ earningsTranscript{args_str} {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "earningsTranscript");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holder_type_to_field_maps_every_variant() {
        assert_eq!(holder_type_to_field(HolderType::Major), "majorHolders");
        assert_eq!(
            holder_type_to_field(HolderType::Institutional),
            "institutionalHolders"
        );
        assert_eq!(
            holder_type_to_field(HolderType::MutualFund),
            "mutualFundHolders"
        );
        assert_eq!(
            holder_type_to_field(HolderType::InsiderTransactions),
            "insiderTransactions"
        );
        assert_eq!(
            holder_type_to_field(HolderType::InsiderPurchases),
            "insiderPurchases"
        );
        assert_eq!(
            holder_type_to_field(HolderType::InsiderRoster),
            "insiderRoster"
        );
    }

    #[test]
    fn holder_paginated_field_maps_paginated_holder_types() {
        assert_eq!(
            holder_paginated_field("institutionalHolders"),
            Some("ownershipList")
        );
        assert_eq!(
            holder_paginated_field("mutualFundHolders"),
            Some("ownershipList")
        );
        assert_eq!(
            holder_paginated_field("insiderTransactions"),
            Some("transactions")
        );
        assert_eq!(holder_paginated_field("insiderRoster"), Some("holders"));
    }

    #[test]
    fn holder_paginated_field_returns_none_for_unpaginated_or_unknown_types() {
        assert_eq!(holder_paginated_field("majorHolders"), None);
        assert_eq!(holder_paginated_field("insiderPurchases"), None);
        assert_eq!(holder_paginated_field("bogusField"), None);
        assert_eq!(holder_paginated_field(""), None);
    }

    #[test]
    fn analysis_type_to_field_maps_every_variant() {
        assert_eq!(
            analysis_type_to_field(AnalysisType::Recommendations),
            "recommendationTrend"
        );
        assert_eq!(
            analysis_type_to_field(AnalysisType::UpgradesDowngrades),
            "gradingHistory"
        );
        assert_eq!(
            analysis_type_to_field(AnalysisType::EarningsEstimate),
            "earningsEstimate"
        );
        assert_eq!(
            analysis_type_to_field(AnalysisType::EarningsHistory),
            "earningsHistory"
        );
    }

    #[test]
    fn build_holders_selection_uses_type_spec_builder_when_not_paginated() {
        let (_, valid_fields, default_fields, composite_fields) = HOLDER_TYPE_SPECS
            .iter()
            .find(|(n, ..)| *n == "majorHolders")
            .unwrap();
        let selection = build_holders_selection(
            None,
            valid_fields,
            default_fields,
            composite_fields,
            None,
            25,
            None,
        );

        for f in *default_fields {
            assert!(selection.contains(f));
        }
        assert!(!selection.contains("first:"));
    }

    #[test]
    fn build_holders_selection_builds_paginated_connection_when_holder_type_is_paginated() {
        let (_, valid_fields, default_fields, composite_fields) = HOLDER_TYPE_SPECS
            .iter()
            .find(|(n, ..)| *n == "institutionalHolders")
            .unwrap();
        let selection = build_holders_selection(
            None,
            valid_fields,
            default_fields,
            composite_fields,
            Some("ownershipList"),
            10,
            None,
        );

        assert!(selection.contains("ownershipList"));
        assert!(selection.contains("first: 10"));
        assert!(selection.contains("edges"));
        assert!(selection.contains("pageInfo"));
    }

    #[test]
    fn build_holders_selection_includes_cursor_when_present() {
        let (_, valid_fields, default_fields, composite_fields) = HOLDER_TYPE_SPECS
            .iter()
            .find(|(n, ..)| *n == "insiderTransactions")
            .unwrap();
        let selection = build_holders_selection(
            None,
            valid_fields,
            default_fields,
            composite_fields,
            Some("transactions"),
            25,
            Some("abc123"),
        );

        assert!(selection.contains("after: \"abc123\""));
    }

    #[test]
    fn build_holders_selection_respects_explicit_field_list() {
        let (_, valid_fields, default_fields, composite_fields) = HOLDER_TYPE_SPECS
            .iter()
            .find(|(n, ..)| *n == "majorHolders")
            .unwrap();
        let requested = vec!["institutionsCount".to_string()];
        let selection = build_holders_selection(
            Some(&requested),
            valid_fields,
            default_fields,
            composite_fields,
            None,
            25,
            None,
        );

        assert_eq!(selection, "{ institutionsCount }");
    }
}
