use finance_query::edgar;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{finance_err, invalid_params, ser_err};

fn edgar_guard() -> Result<(), McpError> {
    if std::env::var("EDGAR_EMAIL").is_err() {
        return Err(invalid_params(
            "EDGAR not configured — set the EDGAR_EMAIL environment variable to enable SEC EDGAR tools",
        ));
    }
    Ok(())
}

pub async fn get_edgar_facts(symbol: String) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let cik = edgar::resolve_cik(&symbol).await.map_err(finance_err)?;
    let facts = edgar::company_facts(cik).await.map_err(finance_err)?;
    let json = serde_json::to_string(&facts).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_edgar_submissions(symbol: String) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let cik = edgar::resolve_cik(&symbol).await.map_err(finance_err)?;
    let submissions = edgar::submissions(cik).await.map_err(finance_err)?;
    let json = serde_json::to_string(&submissions).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}

pub async fn get_edgar_search(
    query: String,
    forms: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<CallToolResult, McpError> {
    edgar_guard()?;
    let form_list: Vec<&str> = forms
        .as_deref()
        .map(|s| s.split(',').map(str::trim).collect())
        .unwrap_or_default();
    let forms_ref: Option<&[&str]> = if form_list.is_empty() {
        None
    } else {
        Some(&form_list)
    };
    let results = edgar::search(
        &query,
        forms_ref,
        start_date.as_deref(),
        end_date.as_deref(),
        None,
        None,
    )
    .await
    .map_err(finance_err)?;
    let json = serde_json::to_string(&results).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(json)]))
}
