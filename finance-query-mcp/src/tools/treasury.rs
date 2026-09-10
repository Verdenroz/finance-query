use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_TREASURY_AUCTION_VALID_FIELDS, GQL_UPCOMING_AUCTION_VALID_FIELDS,
    build_connection_selection, build_selection_or_default, escape_gql_string, execute_query,
    parse_fields, unwrap_field, wrap_connection,
};

/// Build the trailing `first`/`after` arguments every auction query carries.
fn page_args(limit: Option<u32>, cursor: Option<&str>) -> String {
    let mut args = vec![format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE))];
    if let Some(c) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    args.join(", ")
}

#[allow(clippy::too_many_arguments)]
pub async fn get_treasury_auctions(
    schema: &FinanceSchema,
    upcoming: Option<bool>,
    security_type: Option<String>,
    security_term: Option<String>,
    from: Option<String>,
    to: Option<String>,
    count: Option<u32>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    if upcoming.unwrap_or(false) {
        return upcoming_auctions(schema, fields, limit, cursor).await;
    }

    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_TREASURY_AUCTION_VALID_FIELDS,
        GQL_TREASURY_AUCTION_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let page = page_args(limit, cursor.as_deref());
    let query = format!(
        "query TreasuryAuctions($securityType: String, $securityTerm: String, \
         $from: String, $to: String, $count: Int) {{ \
         treasuryAuctions(securityType: $securityType, securityTerm: $securityTerm, \
         from: $from, to: $to, count: $count, {page}) {selection} }}"
    );

    let mut variables = async_graphql::Variables::default();
    variables.insert(
        async_graphql::Name::new("securityType"),
        security_type.into(),
    );
    variables.insert(
        async_graphql::Name::new("securityTerm"),
        security_term.into(),
    );
    variables.insert(async_graphql::Name::new("from"), from.into());
    variables.insert(async_graphql::Name::new("to"), to.into());
    variables.insert(async_graphql::Name::new("count"), count.into());

    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_connection(unwrap_field(json, "treasuryAuctions"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

async fn upcoming_auctions(
    schema: &FinanceSchema,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_UPCOMING_AUCTION_VALID_FIELDS,
        GQL_UPCOMING_AUCTION_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let page = page_args(limit, cursor.as_deref());
    let query = format!("query {{ upcomingAuctions({page}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_connection(unwrap_field(json, "upcomingAuctions"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_args_default_to_the_curated_page_size() {
        assert_eq!(
            page_args(None, None),
            format!("first: {DEFAULT_MCP_PAGE_SIZE}")
        );
    }

    #[test]
    fn page_args_place_the_cursor_after_first() {
        assert_eq!(
            page_args(Some(5), Some("cur1")),
            "first: 5, after: \"cur1\""
        );
    }

    #[test]
    fn page_args_escape_quotes_in_the_cursor() {
        assert!(page_args(None, Some("has\"quote")).contains("after: \"has\\\"quote\""));
    }
}
